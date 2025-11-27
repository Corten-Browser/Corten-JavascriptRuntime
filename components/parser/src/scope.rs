//! Scope analysis for JavaScript AST

use crate::ast::*;
use core_types::JsError;
use std::collections::{HashMap, HashSet};

/// Scope information for a program
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    /// All scopes in the program
    pub scopes: Vec<Scope>,
    /// Variable bindings
    pub bindings: HashMap<String, VariableBinding>,
}

/// A single scope
#[derive(Debug, Clone)]
pub struct Scope {
    /// Scope ID
    pub id: usize,
    /// Parent scope ID
    pub parent: Option<usize>,
    /// Variables declared in this scope
    pub variables: HashSet<String>,
    /// Variables that need to be heap-allocated (used in closures)
    pub heap_variables: HashSet<String>,
    /// Is function scope
    pub is_function: bool,
}

/// Variable binding information
#[derive(Debug, Clone)]
pub struct VariableBinding {
    /// Scope where variable is declared
    pub scope_id: usize,
    /// Is captured by closure
    pub is_captured: bool,
    /// Declaration kind
    pub kind: VariableKind,
}

/// Scope analyzer for JavaScript AST
pub struct ScopeAnalyzer {
    scopes: Vec<Scope>,
    current_scope: usize,
    bindings: HashMap<String, VariableBinding>,
    references: Vec<(String, usize)>, // (name, scope_id)
}

impl ScopeAnalyzer {
    /// Create a new scope analyzer
    pub fn new() -> Self {
        let global_scope = Scope {
            id: 0,
            parent: None,
            variables: HashSet::new(),
            heap_variables: HashSet::new(),
            is_function: false,
        };

        Self {
            scopes: vec![global_scope],
            current_scope: 0,
            bindings: HashMap::new(),
            references: Vec::new(),
        }
    }

    /// Analyze the AST and return scope information
    pub fn analyze(&self, ast: &mut ASTNode) -> Result<ScopeInfo, JsError> {
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.visit_node(ast)?;
        analyzer.resolve_references();

        Ok(ScopeInfo {
            scopes: analyzer.scopes,
            bindings: analyzer.bindings,
        })
    }

    fn visit_node(&mut self, node: &mut ASTNode) -> Result<(), JsError> {
        match node {
            ASTNode::Program(statements) => {
                for stmt in statements {
                    self.visit_statement(stmt)?;
                }
            }
            ASTNode::Statement(stmt) => {
                self.visit_statement(stmt)?;
            }
            ASTNode::Expression(expr) => {
                self.visit_expression(expr)?;
            }
        }
        Ok(())
    }

    fn visit_statement(&mut self, stmt: &mut Statement) -> Result<(), JsError> {
        match stmt {
            Statement::VariableDeclaration {
                kind, declarations, ..
            } => {
                for decl in declarations {
                    self.declare_pattern(&decl.id, *kind)?;
                    if let Some(init) = &mut decl.init {
                        self.visit_expression(init)?;
                    }
                }
            }

            Statement::FunctionDeclaration {
                name, params, body, ..
            } => {
                // Declare function name in current scope
                self.declare_variable(name, VariableKind::Var)?;

                // Create new function scope
                let func_scope = self.enter_scope(true);

                // Declare parameters
                for param in params {
                    self.declare_pattern(param, VariableKind::Let)?;
                }

                // Visit body
                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                self.exit_scope(func_scope);
            }

            Statement::ClassDeclaration {
                name,
                super_class,
                body,
                ..
            } => {
                self.declare_variable(name, VariableKind::Let)?;

                if let Some(expr) = super_class {
                    self.visit_expression(expr)?;
                }

                for element in body {
                    match element {
                        ClassElement::MethodDefinition { value, .. } => {
                            self.visit_expression(value)?;
                        }
                        ClassElement::PropertyDefinition { value, .. } => {
                            if let Some(expr) = value {
                                self.visit_expression(expr)?;
                            }
                        }
                    }
                }
            }

            Statement::ExpressionStatement { expression, .. } => {
                self.visit_expression(expression)?;
            }

            Statement::ReturnStatement { argument, .. } => {
                if let Some(expr) = argument {
                    self.visit_expression(expr)?;
                }
            }

            Statement::IfStatement {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.visit_expression(test)?;
                self.visit_statement(consequent)?;
                if let Some(alt) = alternate {
                    self.visit_statement(alt)?;
                }
            }

            Statement::WhileStatement { test, body, .. } => {
                self.visit_expression(test)?;
                self.visit_statement(body)?;
            }

            Statement::ForStatement {
                init,
                test,
                update,
                body,
                ..
            } => {
                let loop_scope = self.enter_scope(false);

                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration { kind, declarations } => {
                            for decl in declarations {
                                self.declare_pattern(&decl.id, *kind)?;
                                if let Some(expr) = &mut decl.init {
                                    self.visit_expression(expr)?;
                                }
                            }
                        }
                        ForInit::Expression(expr) => {
                            self.visit_expression(expr)?;
                        }
                    }
                }

                if let Some(test) = test {
                    self.visit_expression(test)?;
                }

                if let Some(update) = update {
                    self.visit_expression(update)?;
                }

                self.visit_statement(body)?;

                self.exit_scope(loop_scope);
            }

            Statement::ForInStatement {
                left,
                right,
                body,
                ..
            } => {
                let loop_scope = self.enter_scope(false);

                match left {
                    ForInOfLeft::VariableDeclaration { kind, id } => {
                        self.declare_pattern(id, *kind)?;
                    }
                    ForInOfLeft::Pattern(pattern) => {
                        self.visit_pattern_refs(pattern)?;
                    }
                    ForInOfLeft::Expression(expr) => {
                        self.visit_expression(expr)?;
                    }
                }

                self.visit_expression(right)?;
                self.visit_statement(body)?;

                self.exit_scope(loop_scope);
            }

            Statement::ForOfStatement {
                left,
                right,
                body,
                ..
            } => {
                let loop_scope = self.enter_scope(false);

                match left {
                    ForInOfLeft::VariableDeclaration { kind, id } => {
                        self.declare_pattern(id, *kind)?;
                    }
                    ForInOfLeft::Pattern(pattern) => {
                        self.visit_pattern_refs(pattern)?;
                    }
                    ForInOfLeft::Expression(expr) => {
                        self.visit_expression(expr)?;
                    }
                }

                self.visit_expression(right)?;
                self.visit_statement(body)?;

                self.exit_scope(loop_scope);
            }

            Statement::BlockStatement { body, .. } => {
                let block_scope = self.enter_scope(false);
                for stmt in body {
                    self.visit_statement(stmt)?;
                }
                self.exit_scope(block_scope);
            }

            Statement::ThrowStatement { argument, .. } => {
                self.visit_expression(argument)?;
            }

            Statement::TryStatement {
                block,
                handler,
                finalizer,
                ..
            } => {
                let try_scope = self.enter_scope(false);
                for stmt in block {
                    self.visit_statement(stmt)?;
                }
                self.exit_scope(try_scope);

                if let Some(catch) = handler {
                    let catch_scope = self.enter_scope(false);
                    if let Some(param) = &catch.param {
                        self.declare_pattern(param, VariableKind::Let)?;
                    }
                    for stmt in &mut catch.body.clone() {
                        self.visit_statement(stmt)?;
                    }
                    self.exit_scope(catch_scope);
                }

                if let Some(fin) = finalizer {
                    let finally_scope = self.enter_scope(false);
                    for stmt in &mut fin.clone() {
                        self.visit_statement(stmt)?;
                    }
                    self.exit_scope(finally_scope);
                }
            }

            Statement::EmptyStatement { .. }
            | Statement::BreakStatement { .. }
            | Statement::ContinueStatement { .. }
            | Statement::DebuggerStatement { .. } => {}

            Statement::DoWhileStatement { body, test, .. } => {
                let loop_scope = self.enter_scope(false);
                self.visit_statement(body)?;
                self.visit_expression(test)?;
                self.exit_scope(loop_scope);
            }

            Statement::SwitchStatement {
                discriminant,
                cases,
                ..
            } => {
                self.visit_expression(discriminant)?;
                let switch_scope = self.enter_scope(false);
                for case in cases {
                    if let Some(test) = &mut case.test {
                        self.visit_expression(test)?;
                    }
                    for stmt in &mut case.consequent {
                        self.visit_statement(stmt)?;
                    }
                }
                self.exit_scope(switch_scope);
            }

            Statement::WithStatement { object, body, .. } => {
                self.visit_expression(object)?;
                let with_scope = self.enter_scope(false);
                self.visit_statement(body)?;
                self.exit_scope(with_scope);
            }

            Statement::LabeledStatement { body, .. } => {
                self.visit_statement(body)?;
            }
        }
        Ok(())
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), JsError> {
        match expr {
            Expression::Identifier { name, .. } => {
                self.reference_variable(name);
            }

            Expression::BinaryExpression { left, right, .. } => {
                self.visit_expression(left)?;
                self.visit_expression(right)?;
            }

            Expression::UnaryExpression { argument, .. } => {
                self.visit_expression(argument)?;
            }

            Expression::UpdateExpression { argument, .. } => {
                self.visit_expression(argument)?;
            }

            Expression::LogicalExpression { left, right, .. } => {
                self.visit_expression(left)?;
                self.visit_expression(right)?;
            }

            Expression::AssignmentExpression { left, right, .. } => {
                match left {
                    AssignmentTarget::Identifier(name) => {
                        self.reference_variable(name);
                    }
                    AssignmentTarget::Member(expr) => {
                        self.visit_expression(expr)?;
                    }
                    AssignmentTarget::Pattern(pattern) => {
                        self.visit_pattern_refs(pattern)?;
                    }
                }
                self.visit_expression(right)?;
            }

            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.visit_expression(test)?;
                self.visit_expression(consequent)?;
                self.visit_expression(alternate)?;
            }

            Expression::CallExpression {
                callee, arguments, ..
            } => {
                self.visit_expression(callee)?;
                for arg in arguments {
                    self.visit_expression(arg)?;
                }
            }

            Expression::MemberExpression {
                object, property, ..
            } => {
                self.visit_expression(object)?;
                self.visit_expression(property)?;
            }

            Expression::NewExpression {
                callee, arguments, ..
            } => {
                self.visit_expression(callee)?;
                for arg in arguments {
                    self.visit_expression(arg)?;
                }
            }

            Expression::MetaProperty { .. } => {
                // MetaProperty (new.target, import.meta) has no sub-expressions to analyze
            }

            Expression::ArrayExpression { elements, .. } => {
                for elem in elements {
                    if let Some(el) = elem {
                        match el {
                            ArrayElement::Expression(e) => self.visit_expression(e)?,
                            ArrayElement::Spread(e) => self.visit_expression(e)?,
                        }
                    }
                }
            }

            Expression::ObjectExpression { properties, .. } => {
                for prop in properties {
                    match prop {
                        ObjectProperty::Property { value, .. } => {
                            self.visit_expression(value)?;
                        }
                        ObjectProperty::SpreadElement(expr) => {
                            self.visit_expression(expr)?;
                        }
                    }
                }
            }

            Expression::ArrowFunctionExpression { params, body, .. } => {
                let func_scope = self.enter_scope(true);

                for param in params {
                    self.declare_pattern(param, VariableKind::Let)?;
                }

                match body {
                    ArrowFunctionBody::Expression(expr) => {
                        self.visit_expression(expr)?;
                    }
                    ArrowFunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            self.visit_statement(stmt)?;
                        }
                    }
                }

                self.exit_scope(func_scope);
            }

            Expression::FunctionExpression {
                name, params, body, ..
            } => {
                let func_scope = self.enter_scope(true);

                if let Some(n) = name {
                    self.declare_variable(n, VariableKind::Let)?;
                }

                for param in params {
                    self.declare_pattern(param, VariableKind::Let)?;
                }

                for stmt in body {
                    self.visit_statement(stmt)?;
                }

                self.exit_scope(func_scope);
            }

            Expression::AwaitExpression { argument, .. } => {
                self.visit_expression(argument)?;
            }

            Expression::YieldExpression { argument, .. } => {
                if let Some(arg) = argument {
                    self.visit_expression(arg)?;
                }
            }

            Expression::TemplateLiteral { expressions, .. } => {
                for expr in expressions {
                    self.visit_expression(expr)?;
                }
            }

            Expression::SpreadElement { argument, .. } => {
                self.visit_expression(argument)?;
            }

            Expression::SequenceExpression { expressions, .. } => {
                for expr in expressions {
                    self.visit_expression(expr)?;
                }
            }

            Expression::Literal { .. }
            | Expression::ThisExpression { .. }
            | Expression::SuperExpression { .. } => {}
        }
        Ok(())
    }

    fn visit_pattern_refs(&mut self, pattern: &Pattern) -> Result<(), JsError> {
        match pattern {
            Pattern::Identifier(name) => {
                self.reference_variable(name);
            }
            Pattern::ObjectPattern(props) => {
                for prop in props {
                    self.visit_pattern_refs(&prop.value)?;
                }
            }
            Pattern::ArrayPattern(elems) => {
                for elem in elems {
                    if let Some(p) = elem {
                        self.visit_pattern_refs(p)?;
                    }
                }
            }
            Pattern::AssignmentPattern { left, right } => {
                self.visit_pattern_refs(left)?;
                let mut right_clone = (**right).clone();
                self.visit_expression(&mut right_clone)?;
            }
            Pattern::RestElement(p) => {
                self.visit_pattern_refs(p)?;
            }
        }
        Ok(())
    }

    fn declare_pattern(&mut self, pattern: &Pattern, kind: VariableKind) -> Result<(), JsError> {
        match pattern {
            Pattern::Identifier(name) => {
                self.declare_variable(name, kind)?;
            }
            Pattern::ObjectPattern(props) => {
                for prop in props {
                    self.declare_pattern(&prop.value, kind)?;
                }
            }
            Pattern::ArrayPattern(elems) => {
                for elem in elems {
                    if let Some(p) = elem {
                        self.declare_pattern(p, kind)?;
                    }
                }
            }
            Pattern::AssignmentPattern { left, .. } => {
                self.declare_pattern(left, kind)?;
            }
            Pattern::RestElement(p) => {
                self.declare_pattern(p, kind)?;
            }
        }
        Ok(())
    }

    fn declare_variable(&mut self, name: &str, kind: VariableKind) -> Result<(), JsError> {
        let scope = &mut self.scopes[self.current_scope];
        scope.variables.insert(name.to_string());

        self.bindings.insert(
            name.to_string(),
            VariableBinding {
                scope_id: self.current_scope,
                is_captured: false,
                kind,
            },
        );

        Ok(())
    }

    fn reference_variable(&mut self, name: &str) {
        self.references.push((name.to_string(), self.current_scope));
    }

    fn resolve_references(&mut self) {
        let refs = self.references.clone();
        for (name, ref_scope) in refs {
            let should_mark = if let Some(binding) = self.bindings.get(&name) {
                self.crosses_function_boundary(binding.scope_id, ref_scope)
            } else {
                false
            };

            if should_mark {
                if let Some(binding) = self.bindings.get_mut(&name) {
                    let scope_id = binding.scope_id;
                    binding.is_captured = true;
                    self.scopes[scope_id].heap_variables.insert(name.clone());
                }
            }
        }
    }

    fn crosses_function_boundary(&self, decl_scope: usize, ref_scope: usize) -> bool {
        if decl_scope == ref_scope {
            return false;
        }

        let mut current = ref_scope;
        while let Some(parent) = self.scopes[current].parent {
            if parent == decl_scope {
                return false;
            }
            if self.scopes[current].is_function {
                return true;
            }
            current = parent;
        }

        false
    }

    fn enter_scope(&mut self, is_function: bool) -> usize {
        let new_scope = Scope {
            id: self.scopes.len(),
            parent: Some(self.current_scope),
            variables: HashSet::new(),
            heap_variables: HashSet::new(),
            is_function,
        };
        let id = new_scope.id;
        self.scopes.push(new_scope);
        self.current_scope = id;
        id
    }

    fn exit_scope(&mut self, _scope_id: usize) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }
}

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_analyzer_creation() {
        let analyzer = ScopeAnalyzer::new();
        assert_eq!(analyzer.scopes.len(), 1);
        assert_eq!(analyzer.current_scope, 0);
    }

    #[test]
    fn test_analyze_empty_program() {
        let analyzer = ScopeAnalyzer::new();
        let mut ast = ASTNode::Program(vec![]);
        let info = analyzer.analyze(&mut ast).unwrap();
        assert!(!info.scopes.is_empty());
    }

    #[test]
    fn test_analyze_variable_declaration() {
        let analyzer = ScopeAnalyzer::new();
        let mut ast = ASTNode::Program(vec![Statement::VariableDeclaration {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier("x".to_string()),
                init: None,
            }],
            position: None,
        }]);

        let info = analyzer.analyze(&mut ast).unwrap();
        assert!(info.bindings.contains_key("x"));
    }

    #[test]
    fn test_analyze_function_creates_scope() {
        let analyzer = ScopeAnalyzer::new();
        let mut ast = ASTNode::Program(vec![Statement::FunctionDeclaration {
            name: "foo".to_string(),
            params: vec![],
            body: vec![],
            is_async: false,
            is_generator: false,
            position: None,
        }]);

        let info = analyzer.analyze(&mut ast).unwrap();
        assert!(info.scopes.len() > 1);
    }
}
