//! Bytecode generation from AST

use crate::ast::*;
use bytecode_system::{BytecodeChunk, Opcode, RegisterId, Value as BytecodeValue};
use core_types::{ErrorKind, JsError};
use std::collections::HashMap;

/// Bytecode generator that converts AST to bytecode
pub struct BytecodeGenerator {
    chunk: BytecodeChunk,
    locals: HashMap<String, RegisterId>,
    next_register: u32,
    loop_starts: Vec<usize>,
    loop_exits: Vec<Vec<usize>>,
    last_was_expression: bool,
}

impl BytecodeGenerator {
    /// Create a new bytecode generator
    pub fn new() -> Self {
        Self {
            chunk: BytecodeChunk::new(),
            locals: HashMap::new(),
            next_register: 0,
            loop_starts: Vec::new(),
            loop_exits: Vec::new(),
            last_was_expression: false,
        }
    }

    /// Generate bytecode from AST
    pub fn generate(&mut self, ast: &ASTNode) -> Result<BytecodeChunk, JsError> {
        self.visit_node(ast)?;

        // Ensure there's always a return
        if self.chunk.instructions.is_empty()
            || !matches!(
                self.chunk.instructions.last().map(|i| &i.opcode),
                Some(Opcode::Return)
            )
        {
            // Only load undefined if the last statement wasn't an expression
            // (expression results should be preserved for return)
            if !self.last_was_expression {
                self.chunk.emit(Opcode::LoadUndefined);
            }
            self.chunk.emit(Opcode::Return);
        }

        self.chunk.register_count = self.next_register;
        Ok(self.chunk.clone())
    }

    fn visit_node(&mut self, node: &ASTNode) -> Result<(), JsError> {
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

    fn visit_statement(&mut self, stmt: &Statement) -> Result<(), JsError> {
        // Track whether this is an expression statement for return value handling
        self.last_was_expression = matches!(stmt, Statement::ExpressionStatement { .. });

        match stmt {
            Statement::VariableDeclaration { declarations, .. } => {
                for decl in declarations {
                    let reg = self.allocate_register();

                    if let Some(init) = &decl.init {
                        self.visit_expression(init)?;
                    } else {
                        self.chunk.emit(Opcode::LoadUndefined);
                    }

                    self.chunk.emit(Opcode::StoreLocal(reg));

                    if let Pattern::Identifier(name) = &decl.id {
                        self.locals.insert(name.clone(), reg);
                    }
                }
            }

            Statement::FunctionDeclaration {
                name, params, body, ..
            } => {
                // Create function bytecode
                let mut func_gen = BytecodeGenerator::new();

                // Set up parameters as locals
                for param in params {
                    if let Pattern::Identifier(param_name) = param {
                        let reg = func_gen.allocate_register();
                        func_gen.locals.insert(param_name.clone(), reg);
                    }
                }

                // Generate body
                for stmt in body {
                    func_gen.visit_statement(stmt)?;
                }

                // Ensure return
                if func_gen.chunk.instructions.is_empty()
                    || !matches!(
                        func_gen.chunk.instructions.last().map(|i| &i.opcode),
                        Some(Opcode::Return)
                    )
                {
                    func_gen.chunk.emit(Opcode::LoadUndefined);
                    func_gen.chunk.emit(Opcode::Return);
                }

                func_gen.chunk.register_count = func_gen.next_register;

                // Store function index (simplified - real impl would store function metadata)
                // For now, we just emit CreateClosure with a placeholder index
                let func_idx = self.chunk.constants.len();

                // Create closure and store
                self.chunk.emit(Opcode::CreateClosure(func_idx));
                let reg = self.allocate_register();
                self.chunk.emit(Opcode::StoreLocal(reg));
                self.locals.insert(name.clone(), reg);
            }

            Statement::ClassDeclaration { .. } => {
                // Simplified class compilation
                self.chunk.emit(Opcode::CreateObject);
                // Real implementation would set up prototype chain
            }

            Statement::ExpressionStatement { expression, .. } => {
                self.visit_expression(expression)?;
                // Pop result (discard)
            }

            Statement::ReturnStatement { argument, .. } => {
                if let Some(expr) = argument {
                    self.visit_expression(expr)?;
                } else {
                    self.chunk.emit(Opcode::LoadUndefined);
                }
                self.chunk.emit(Opcode::Return);
            }

            Statement::IfStatement {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.visit_expression(test)?;

                // Jump to else/end if false
                let else_jump = self.chunk.instruction_count();
                self.chunk.emit(Opcode::JumpIfFalse(0)); // Placeholder

                // Consequent
                self.visit_statement(consequent)?;

                if let Some(alt) = alternate {
                    // Jump over else
                    let end_jump = self.chunk.instruction_count();
                    self.chunk.emit(Opcode::Jump(0)); // Placeholder

                    // Patch else jump
                    let else_addr = self.chunk.instruction_count();
                    self.patch_jump(else_jump, else_addr);

                    // Alternate
                    self.visit_statement(alt)?;

                    // Patch end jump
                    let end_addr = self.chunk.instruction_count();
                    self.patch_jump(end_jump, end_addr);
                } else {
                    // Patch else jump
                    let end_addr = self.chunk.instruction_count();
                    self.patch_jump(else_jump, end_addr);
                }
            }

            Statement::WhileStatement { test, body, .. } => {
                let loop_start = self.chunk.instruction_count();
                self.loop_starts.push(loop_start);
                self.loop_exits.push(Vec::new());

                self.visit_expression(test)?;

                let exit_jump = self.chunk.instruction_count();
                self.chunk.emit(Opcode::JumpIfFalse(0)); // Placeholder

                self.visit_statement(body)?;

                // Loop back
                self.chunk.emit(Opcode::Jump(loop_start));

                // Patch exit jump
                let end_addr = self.chunk.instruction_count();
                self.patch_jump(exit_jump, end_addr);

                // Patch any break jumps
                let exits = self.loop_exits.pop().unwrap();
                for exit in exits {
                    self.patch_jump(exit, end_addr);
                }
                self.loop_starts.pop();
            }

            Statement::ForStatement {
                init,
                test,
                update,
                body,
                ..
            } => {
                // Init
                if let Some(init) = init {
                    match init {
                        ForInit::VariableDeclaration { declarations, .. } => {
                            for decl in declarations {
                                let reg = self.allocate_register();
                                if let Some(expr) = &decl.init {
                                    self.visit_expression(expr)?;
                                } else {
                                    self.chunk.emit(Opcode::LoadUndefined);
                                }
                                self.chunk.emit(Opcode::StoreLocal(reg));
                                if let Pattern::Identifier(name) = &decl.id {
                                    self.locals.insert(name.clone(), reg);
                                }
                            }
                        }
                        ForInit::Expression(expr) => {
                            self.visit_expression(expr)?;
                        }
                    }
                }

                let loop_start = self.chunk.instruction_count();
                self.loop_starts.push(loop_start);
                self.loop_exits.push(Vec::new());

                // Test
                let exit_jump = if let Some(test) = test {
                    self.visit_expression(test)?;
                    let j = self.chunk.instruction_count();
                    self.chunk.emit(Opcode::JumpIfFalse(0));
                    Some(j)
                } else {
                    None
                };

                // Body
                self.visit_statement(body)?;

                // Update
                if let Some(update) = update {
                    self.visit_expression(update)?;
                }

                // Loop back
                self.chunk.emit(Opcode::Jump(loop_start));

                // Patch exit
                let end_addr = self.chunk.instruction_count();
                if let Some(j) = exit_jump {
                    self.patch_jump(j, end_addr);
                }

                let exits = self.loop_exits.pop().unwrap();
                for exit in exits {
                    self.patch_jump(exit, end_addr);
                }
                self.loop_starts.pop();
            }

            Statement::BlockStatement { body, .. } => {
                for stmt in body {
                    self.visit_statement(stmt)?;
                }
            }

            Statement::BreakStatement { .. } => {
                let jump = self.chunk.instruction_count();
                self.chunk.emit(Opcode::Jump(0)); // Placeholder
                if let Some(exits) = self.loop_exits.last_mut() {
                    exits.push(jump);
                }
            }

            Statement::ContinueStatement { .. } => {
                if let Some(&start) = self.loop_starts.last() {
                    self.chunk.emit(Opcode::Jump(start));
                }
            }

            Statement::ThrowStatement { argument, .. } => {
                self.visit_expression(argument)?;
                // Would emit throw opcode - simplified for now
                self.chunk.emit(Opcode::Return); // Placeholder behavior
            }

            Statement::TryStatement { block, .. } => {
                // Simplified - just execute try block
                for stmt in block {
                    self.visit_statement(stmt)?;
                }
            }

            Statement::EmptyStatement { .. } => {}
        }
        Ok(())
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), JsError> {
        match expr {
            Expression::Identifier { name, .. } => {
                if let Some(&reg) = self.locals.get(name) {
                    self.chunk.emit(Opcode::LoadLocal(reg));
                } else {
                    self.chunk.emit(Opcode::LoadGlobal(name.clone()));
                }
            }

            Expression::Literal { value, .. } => match value {
                Literal::Number(n) => {
                    let idx = self.chunk.add_constant(BytecodeValue::Number(*n));
                    self.chunk.emit(Opcode::LoadConstant(idx));
                }
                Literal::String(s) => {
                    let idx = self.chunk.add_constant(BytecodeValue::String(s.clone()));
                    self.chunk.emit(Opcode::LoadConstant(idx));
                }
                Literal::Boolean(true) => {
                    self.chunk.emit(Opcode::LoadTrue);
                }
                Literal::Boolean(false) => {
                    self.chunk.emit(Opcode::LoadFalse);
                }
                Literal::Null => {
                    self.chunk.emit(Opcode::LoadNull);
                }
                Literal::Undefined => {
                    self.chunk.emit(Opcode::LoadUndefined);
                }
            },

            Expression::BinaryExpression {
                left,
                operator,
                right,
                ..
            } => {
                self.visit_expression(left)?;
                self.visit_expression(right)?;

                let op = match operator {
                    BinaryOperator::Add => Opcode::Add,
                    BinaryOperator::Sub => Opcode::Sub,
                    BinaryOperator::Mul => Opcode::Mul,
                    BinaryOperator::Div => Opcode::Div,
                    BinaryOperator::Mod => Opcode::Mod,
                    BinaryOperator::Eq => Opcode::Equal,
                    BinaryOperator::NotEq => Opcode::NotEqual,
                    BinaryOperator::StrictEq => Opcode::StrictEqual,
                    BinaryOperator::StrictNotEq => Opcode::StrictNotEqual,
                    BinaryOperator::Lt => Opcode::LessThan,
                    BinaryOperator::LtEq => Opcode::LessThanEqual,
                    BinaryOperator::Gt => Opcode::GreaterThan,
                    BinaryOperator::GtEq => Opcode::GreaterThanEqual,
                    _ => {
                        return Err(JsError {
                            kind: ErrorKind::InternalError,
                            message: format!("Unsupported binary operator: {:?}", operator),
                            stack: vec![],
                            source_position: None,
                        })
                    }
                };
                self.chunk.emit(op);
            }

            Expression::UnaryExpression {
                operator, argument, ..
            } => {
                self.visit_expression(argument)?;

                match operator {
                    UnaryOperator::Minus => {
                        self.chunk.emit(Opcode::Neg);
                    }
                    UnaryOperator::Not => {
                        // Logical NOT - simplified
                        self.chunk.emit(Opcode::LoadFalse);
                        self.chunk.emit(Opcode::Equal);
                    }
                    _ => {}
                }
            }

            Expression::UpdateExpression {
                operator,
                argument,
                prefix,
                ..
            } => {
                // Simplified update expression
                self.visit_expression(argument)?;

                let one_idx = self.chunk.add_constant(BytecodeValue::Number(1.0));
                self.chunk.emit(Opcode::LoadConstant(one_idx));

                match operator {
                    UpdateOperator::Increment => {
                        self.chunk.emit(Opcode::Add);
                    }
                    UpdateOperator::Decrement => {
                        self.chunk.emit(Opcode::Sub);
                    }
                }

                // Store back (simplified - doesn't handle all cases)
                if let Expression::Identifier { name, .. } = argument.as_ref() {
                    if let Some(&reg) = self.locals.get(name) {
                        if !prefix {
                            // Return old value for postfix
                            self.visit_expression(argument)?;
                        }
                        self.chunk.emit(Opcode::StoreLocal(reg));
                    }
                }
            }

            Expression::LogicalExpression {
                left,
                operator,
                right,
                ..
            } => {
                self.visit_expression(left)?;

                match operator {
                    LogicalOperator::And => {
                        let skip = self.chunk.instruction_count();
                        self.chunk.emit(Opcode::JumpIfFalse(0));
                        self.visit_expression(right)?;
                        let end = self.chunk.instruction_count();
                        self.patch_jump(skip, end);
                    }
                    LogicalOperator::Or => {
                        let skip = self.chunk.instruction_count();
                        self.chunk.emit(Opcode::JumpIfTrue(0));
                        self.visit_expression(right)?;
                        let end = self.chunk.instruction_count();
                        self.patch_jump(skip, end);
                    }
                    LogicalOperator::NullishCoalesce => {
                        // Simplified - treat as OR for now
                        let skip = self.chunk.instruction_count();
                        self.chunk.emit(Opcode::JumpIfTrue(0));
                        self.visit_expression(right)?;
                        let end = self.chunk.instruction_count();
                        self.patch_jump(skip, end);
                    }
                }
            }

            Expression::AssignmentExpression {
                left,
                operator: _,
                right,
                ..
            } => {
                self.visit_expression(right)?;

                match left {
                    AssignmentTarget::Identifier(name) => {
                        if let Some(&reg) = self.locals.get(name) {
                            self.chunk.emit(Opcode::StoreLocal(reg));
                        } else {
                            self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                        }
                    }
                    AssignmentTarget::Member(member_expr) => {
                        if let Expression::MemberExpression {
                            object,
                            property,
                            computed,
                            ..
                        } = member_expr.as_ref()
                        {
                            self.visit_expression(object)?;
                            if !computed {
                                if let Expression::Identifier { name, .. } = property.as_ref() {
                                    self.chunk.emit(Opcode::StoreProperty(name.clone()));
                                }
                            }
                        }
                    }
                    AssignmentTarget::Pattern(_) => {
                        // Destructuring - simplified
                    }
                }
            }

            Expression::ConditionalExpression {
                test,
                consequent,
                alternate,
                ..
            } => {
                self.visit_expression(test)?;
                let else_jump = self.chunk.instruction_count();
                self.chunk.emit(Opcode::JumpIfFalse(0));

                self.visit_expression(consequent)?;
                let end_jump = self.chunk.instruction_count();
                self.chunk.emit(Opcode::Jump(0));

                let else_addr = self.chunk.instruction_count();
                self.patch_jump(else_jump, else_addr);

                self.visit_expression(alternate)?;

                let end_addr = self.chunk.instruction_count();
                self.patch_jump(end_jump, end_addr);
            }

            Expression::CallExpression {
                callee, arguments, ..
            } => {
                // Push arguments
                for arg in arguments {
                    self.visit_expression(arg)?;
                }

                // Push callee
                self.visit_expression(callee)?;

                // Call
                self.chunk.emit(Opcode::Call(arguments.len() as u8));
            }

            Expression::MemberExpression {
                object, property, ..
            } => {
                self.visit_expression(object)?;

                if let Expression::Identifier { name, .. } = property.as_ref() {
                    self.chunk.emit(Opcode::LoadProperty(name.clone()));
                }
            }

            Expression::NewExpression {
                callee, arguments, ..
            } => {
                // Simplified new - just call
                for arg in arguments {
                    self.visit_expression(arg)?;
                }
                self.visit_expression(callee)?;
                self.chunk.emit(Opcode::Call(arguments.len() as u8));
            }

            Expression::ArrayExpression { elements, .. } => {
                self.chunk.emit(Opcode::CreateObject); // Simplified
                for elem in elements {
                    if let Some(el) = elem {
                        match el {
                            ArrayElement::Expression(e) => {
                                self.visit_expression(e)?;
                            }
                            ArrayElement::Spread(e) => {
                                self.visit_expression(e)?;
                            }
                        }
                    }
                }
            }

            Expression::ObjectExpression { properties, .. } => {
                self.chunk.emit(Opcode::CreateObject);

                for prop in properties {
                    if let ObjectProperty::Property { key, value, .. } = prop {
                        if let PropertyKey::Identifier(name) = key {
                            self.visit_expression(value)?;
                            self.chunk.emit(Opcode::StoreProperty(name.clone()));
                        }
                    }
                }
            }

            Expression::ArrowFunctionExpression { params, body, .. } => {
                let mut func_gen = BytecodeGenerator::new();

                for param in params {
                    if let Pattern::Identifier(name) = param {
                        let reg = func_gen.allocate_register();
                        func_gen.locals.insert(name.clone(), reg);
                    }
                }

                match body {
                    ArrowFunctionBody::Expression(expr) => {
                        func_gen.visit_expression(expr)?;
                        func_gen.chunk.emit(Opcode::Return);
                    }
                    ArrowFunctionBody::Block(stmts) => {
                        for stmt in stmts {
                            func_gen.visit_statement(stmt)?;
                        }
                        if func_gen.chunk.instructions.is_empty()
                            || !matches!(
                                func_gen.chunk.instructions.last().map(|i| &i.opcode),
                                Some(Opcode::Return)
                            )
                        {
                            func_gen.chunk.emit(Opcode::LoadUndefined);
                            func_gen.chunk.emit(Opcode::Return);
                        }
                    }
                }

                func_gen.chunk.register_count = func_gen.next_register;

                // Store function (simplified - use placeholder index)
                let func_idx = self.chunk.constants.len();

                self.chunk.emit(Opcode::CreateClosure(func_idx));
            }

            Expression::FunctionExpression {
                name, params, body, ..
            } => {
                let mut func_gen = BytecodeGenerator::new();

                if let Some(n) = name {
                    let reg = func_gen.allocate_register();
                    func_gen.locals.insert(n.clone(), reg);
                }

                for param in params {
                    if let Pattern::Identifier(param_name) = param {
                        let reg = func_gen.allocate_register();
                        func_gen.locals.insert(param_name.clone(), reg);
                    }
                }

                for stmt in body {
                    func_gen.visit_statement(stmt)?;
                }

                if func_gen.chunk.instructions.is_empty()
                    || !matches!(
                        func_gen.chunk.instructions.last().map(|i| &i.opcode),
                        Some(Opcode::Return)
                    )
                {
                    func_gen.chunk.emit(Opcode::LoadUndefined);
                    func_gen.chunk.emit(Opcode::Return);
                }

                func_gen.chunk.register_count = func_gen.next_register;

                // Store function (simplified - use placeholder index)
                let func_idx = self.chunk.constants.len();

                self.chunk.emit(Opcode::CreateClosure(func_idx));
            }

            Expression::ThisExpression { .. } => {
                self.chunk.emit(Opcode::LoadGlobal("this".to_string()));
            }

            Expression::SuperExpression { .. } => {
                self.chunk.emit(Opcode::LoadGlobal("super".to_string()));
            }

            Expression::AwaitExpression { argument, .. } => {
                self.visit_expression(argument)?;
                // Await is runtime behavior - just evaluate the expression for now
            }

            Expression::YieldExpression { argument, .. } => {
                if let Some(arg) = argument {
                    self.visit_expression(arg)?;
                } else {
                    self.chunk.emit(Opcode::LoadUndefined);
                }
            }

            Expression::TemplateLiteral { quasis, .. } => {
                // Simplified - just load the first quasi
                if let Some(quasi) = quasis.first() {
                    let idx = self
                        .chunk
                        .add_constant(BytecodeValue::String(quasi.cooked.clone()));
                    self.chunk.emit(Opcode::LoadConstant(idx));
                } else {
                    let idx = self
                        .chunk
                        .add_constant(BytecodeValue::String(String::new()));
                    self.chunk.emit(Opcode::LoadConstant(idx));
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
        }
        Ok(())
    }

    fn allocate_register(&mut self) -> RegisterId {
        let reg = RegisterId(self.next_register);
        self.next_register += 1;
        reg
    }

    fn patch_jump(&mut self, jump_idx: usize, target: usize) {
        if let Some(inst) = self.chunk.instructions.get_mut(jump_idx) {
            match &mut inst.opcode {
                Opcode::Jump(ref mut addr)
                | Opcode::JumpIfTrue(ref mut addr)
                | Opcode::JumpIfFalse(ref mut addr) => {
                    *addr = target;
                }
                _ => {}
            }
        }
    }
}

impl Default for BytecodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_generator_creation() {
        let gen = BytecodeGenerator::new();
        assert_eq!(gen.next_register, 0);
    }

    #[test]
    fn test_generate_empty_program() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![]);
        let chunk = gen.generate(&ast).unwrap();
        assert!(!chunk.instructions.is_empty());
    }

    #[test]
    fn test_generate_number_literal() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::Literal {
                value: Literal::Number(42.0),
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_generate_variable_declaration() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::VariableDeclaration {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier("x".to_string()),
                init: Some(Expression::Literal {
                    value: Literal::Number(42.0),
                    position: None,
                }),
            }],
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();
        assert!(chunk.register_count > 0);
    }

    #[test]
    fn test_generate_binary_expression() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::BinaryExpression {
                left: Box::new(Expression::Literal {
                    value: Literal::Number(1.0),
                    position: None,
                }),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Literal {
                    value: Literal::Number(2.0),
                    position: None,
                }),
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();
        let has_add = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::Add));
        assert!(has_add);
    }
}
