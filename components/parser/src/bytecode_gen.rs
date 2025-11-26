//! Bytecode generation from AST

use crate::ast::*;
use bytecode_system::{
    BytecodeChunk, Opcode, RegisterId, UpvalueDescriptor, Value as BytecodeValue,
};
use core_types::{ErrorKind, JsError};
use std::collections::HashMap;

/// Result of resolving a variable name
#[derive(Debug, Clone)]
enum VarResolution {
    /// Variable is a local in the current scope
    Local(RegisterId),
    /// Variable is captured from an outer scope
    Upvalue(u32),
    /// Variable is a global
    Global,
}

/// Bytecode generator that converts AST to bytecode
pub struct BytecodeGenerator {
    chunk: BytecodeChunk,
    locals: HashMap<String, RegisterId>,
    next_register: u32,
    loop_starts: Vec<usize>,
    loop_exits: Vec<Vec<usize>>,
    last_was_expression: bool,

    // For closure support
    /// Parent scope (for nested functions)
    enclosing: Option<Box<BytecodeGenerator>>,
    /// Captured variables from outer scopes
    upvalues: Vec<UpvalueDescriptor>,

    // For nested function registration
    /// Nested function bytecode chunks collected during compilation
    nested_functions: Vec<BytecodeChunk>,
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
            enclosing: None,
            upvalues: Vec::new(),
            nested_functions: Vec::new(),
        }
    }

    /// Create a new bytecode generator with an enclosing scope
    fn with_enclosing(enclosing: Box<BytecodeGenerator>) -> Self {
        Self {
            chunk: BytecodeChunk::new(),
            locals: HashMap::new(),
            next_register: 0,
            loop_starts: Vec::new(),
            loop_exits: Vec::new(),
            last_was_expression: false,
            enclosing: Some(enclosing),
            upvalues: Vec::new(),
            nested_functions: Vec::new(),
        }
    }

    /// Resolve a variable name to its location (local, upvalue, or global)
    fn resolve_variable(&mut self, name: &str) -> VarResolution {
        // Check local scope first
        if let Some(&reg) = self.locals.get(name) {
            return VarResolution::Local(reg);
        }

        // Check enclosing scopes (for closures)
        if let Some(ref mut enclosing) = self.enclosing {
            match enclosing.resolve_variable(name) {
                VarResolution::Local(reg) => {
                    // Capture from parent - it's a local in parent scope
                    let upvalue_idx = self.add_upvalue(UpvalueDescriptor {
                        is_local: true,
                        index: reg.0,
                    });
                    return VarResolution::Upvalue(upvalue_idx);
                }
                VarResolution::Upvalue(idx) => {
                    // Capture from grandparent+ - it's already an upvalue in parent
                    let upvalue_idx = self.add_upvalue(UpvalueDescriptor {
                        is_local: false,
                        index: idx,
                    });
                    return VarResolution::Upvalue(upvalue_idx);
                }
                VarResolution::Global => {}
            }
        }

        VarResolution::Global
    }

    /// Add an upvalue descriptor and return its index
    fn add_upvalue(&mut self, descriptor: UpvalueDescriptor) -> u32 {
        // Check if already captured
        for (i, uv) in self.upvalues.iter().enumerate() {
            if uv == &descriptor {
                return i as u32;
            }
        }
        // Add new upvalue
        let idx = self.upvalues.len() as u32;
        self.upvalues.push(descriptor);
        idx
    }

    /// Get the captured upvalues for this function
    pub fn get_upvalues(&self) -> Vec<UpvalueDescriptor> {
        self.upvalues.clone()
    }

    /// Get the nested functions collected during compilation
    ///
    /// Returns all function bytecode chunks that were compiled as nested functions.
    /// These should be registered with the VM before executing the main bytecode.
    pub fn take_nested_functions(&mut self) -> Vec<BytecodeChunk> {
        std::mem::take(&mut self.nested_functions)
    }

    /// Get a reference to nested functions without consuming them
    pub fn nested_functions(&self) -> &[BytecodeChunk] {
        &self.nested_functions
    }

    /// Adjust closure indices in bytecode to account for function registry offset
    ///
    /// When a function contains nested functions, the nested functions get indices
    /// 0, 1, 2... during compilation. But when we merge them into the parent's
    /// nested_functions list, they'll be at different indices. This method adjusts
    /// the CreateClosure and CreateAsyncFunction opcodes to use the correct indices.
    fn adjust_closure_indices(chunk: &mut BytecodeChunk, base_idx: usize) {
        for inst in &mut chunk.instructions {
            match &mut inst.opcode {
                Opcode::CreateClosure(idx, _) => {
                    *idx = *idx + base_idx;
                }
                Opcode::CreateAsyncFunction(idx, _) => {
                    *idx = *idx + base_idx;
                }
                _ => {}
            }
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

        // Transfer nested functions to the chunk
        // This allows the VM to access them when executing closures
        let mut result_chunk = self.chunk.clone();
        result_chunk.nested_functions = self.nested_functions.clone();

        Ok(result_chunk)
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
                // Create function bytecode with enclosing scope for closure support
                // We need to temporarily take ownership of self to pass it as enclosing
                let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                let mut func_gen = BytecodeGenerator::with_enclosing(Box::new(current_gen));

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

                // Get the upvalues captured by this function
                let upvalues = func_gen.get_upvalues();

                // Get the compiled function bytecode
                let mut func_bytecode = func_gen.chunk.clone();

                // Collect any nested functions from the inner function
                let inner_nested = func_gen.take_nested_functions();

                // Restore the outer generator from the enclosing scope
                *self = *func_gen.enclosing.take().unwrap();

                // Add the compiled function to our nested functions list
                let func_idx = self.nested_functions.len();

                // Adjust indices in the function's bytecode for nested functions
                // The inner functions will be placed after this function in the list
                let inner_base_idx = func_idx + 1;
                Self::adjust_closure_indices(&mut func_bytecode, inner_base_idx);

                self.nested_functions.push(func_bytecode);

                // Also include any nested functions from the inner function
                // IMPORTANT: We need to adjust indices in these nested functions too!
                // Their closure indices were relative to their parent's nested_functions list,
                // but now they need to be relative to our nested_functions list.
                let mut adjusted_inner_nested = inner_nested;
                for nested_chunk in &mut adjusted_inner_nested {
                    Self::adjust_closure_indices(nested_chunk, inner_base_idx);
                }
                self.nested_functions.extend(adjusted_inner_nested);

                // Create closure with upvalue descriptors (func_idx is now a proper function registry index)
                self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));

                // Store the function: if at top level (no enclosing), store as global
                // Otherwise store as local
                if self.enclosing.is_none() {
                    // Top-level function declaration - store as global so other functions can access it
                    self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                } else {
                    // Nested function - store in local register
                    let reg = self.allocate_register();
                    self.chunk.emit(Opcode::StoreLocal(reg));
                    self.locals.insert(name.clone(), reg);
                }
            }

            Statement::ClassDeclaration { name, body, .. } => {
                // A class declaration creates a constructor function bound to the class name
                // Find the constructor method in the class body
                let constructor_method = body.iter().find_map(|element| {
                    if let ClassElement::MethodDefinition {
                        kind: MethodKind::Constructor,
                        value,
                        ..
                    } = element
                    {
                        Some(value)
                    } else {
                        None
                    }
                });

                if let Some(constructor_expr) = constructor_method {
                    // Extract parameters and body from the constructor function expression
                    if let Expression::FunctionExpression {
                        params,
                        body: func_body,
                        ..
                    } = constructor_expr
                    {
                        // Create function bytecode with enclosing scope for closure support
                        let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                        let mut func_gen =
                            BytecodeGenerator::with_enclosing(Box::new(current_gen));

                        // Set up parameters as locals
                        for param in params {
                            if let Pattern::Identifier(param_name) = param {
                                let reg = func_gen.allocate_register();
                                func_gen.locals.insert(param_name.clone(), reg);
                            }
                        }

                        // Generate constructor body
                        for stmt in func_body {
                            func_gen.visit_statement(stmt)?;
                        }

                        // Ensure return (constructor returns 'this' implicitly, but we handle that in VM)
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

                        // Get the upvalues captured by this function
                        let upvalues = func_gen.get_upvalues();

                        // Get the compiled function bytecode
                        let mut func_bytecode = func_gen.chunk.clone();

                        // Collect any nested functions from the inner function
                        let inner_nested = func_gen.take_nested_functions();

                        // Restore the outer generator from the enclosing scope
                        *self = *func_gen.enclosing.take().unwrap();

                        // Add the compiled function to our nested functions list
                        let func_idx = self.nested_functions.len();

                        // Adjust indices in the function's bytecode for nested functions
                        let inner_base_idx = func_idx + 1;
                        Self::adjust_closure_indices(&mut func_bytecode, inner_base_idx);

                        self.nested_functions.push(func_bytecode);

                        // Also include any nested functions from the inner function
                        let mut adjusted_inner_nested = inner_nested;
                        for nested_chunk in &mut adjusted_inner_nested {
                            Self::adjust_closure_indices(nested_chunk, inner_base_idx);
                        }
                        self.nested_functions.extend(adjusted_inner_nested);

                        // Create closure with upvalue descriptors
                        self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));

                        // Store the constructor function with the class name as a global
                        self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                    } else {
                        // Constructor is not a function expression (shouldn't happen with valid parsing)
                        // Fallback: create empty constructor
                        let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                        let mut func_gen =
                            BytecodeGenerator::with_enclosing(Box::new(current_gen));
                        func_gen.chunk.emit(Opcode::LoadUndefined);
                        func_gen.chunk.emit(Opcode::Return);
                        func_gen.chunk.register_count = func_gen.next_register;
                        let upvalues = func_gen.get_upvalues();
                        let func_bytecode = func_gen.chunk.clone();
                        *self = *func_gen.enclosing.take().unwrap();
                        let func_idx = self.nested_functions.len();
                        self.nested_functions.push(func_bytecode);
                        self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));
                        self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                    }
                } else {
                    // No explicit constructor - create a default empty constructor
                    let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                    let mut func_gen = BytecodeGenerator::with_enclosing(Box::new(current_gen));
                    func_gen.chunk.emit(Opcode::LoadUndefined);
                    func_gen.chunk.emit(Opcode::Return);
                    func_gen.chunk.register_count = func_gen.next_register;
                    let upvalues = func_gen.get_upvalues();
                    let func_bytecode = func_gen.chunk.clone();
                    *self = *func_gen.enclosing.take().unwrap();
                    let func_idx = self.nested_functions.len();
                    self.nested_functions.push(func_bytecode);
                    self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));
                    self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                }
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
                self.chunk.emit(Opcode::Throw);
            }

            Statement::TryStatement {
                block,
                handler,
                finalizer,
                ..
            } => {
                // Emit PushTry with placeholder for catch offset
                let push_try_idx = self.chunk.instruction_count();
                self.chunk.emit(Opcode::PushTry(0)); // Will patch later

                // Execute try block
                for stmt in block {
                    self.visit_statement(stmt)?;
                }

                // Pop the try handler (no exception occurred)
                self.chunk.emit(Opcode::PopTry);

                // Jump over catch block (normal execution path)
                let jump_over_catch_idx = self.chunk.instruction_count();
                self.chunk.emit(Opcode::Jump(0)); // Will patch later

                // Patch PushTry to point to catch block start
                let catch_start = self.chunk.instruction_count();
                self.patch_jump(push_try_idx, catch_start);

                // Handle catch block
                if let Some(catch_clause) = handler {
                    // Bind exception value to parameter (exception is on stack)
                    if let Some(Pattern::Identifier(param_name)) = &catch_clause.param {
                        let reg = self.allocate_register();
                        self.locals.insert(param_name.clone(), reg);
                        self.chunk.emit(Opcode::StoreLocal(reg));
                    } else {
                        // No parameter or non-identifier pattern - discard exception
                        self.chunk.emit(Opcode::Pop);
                    }

                    // Execute catch block body
                    for stmt in &catch_clause.body {
                        self.visit_statement(stmt)?;
                    }
                } else {
                    // No catch block - just pop the exception value
                    self.chunk.emit(Opcode::Pop);
                }

                // Patch jump to skip over catch block
                let after_catch = self.chunk.instruction_count();
                self.patch_jump(jump_over_catch_idx, after_catch);

                // Handle finally block (if present)
                if let Some(finally_block) = finalizer {
                    for stmt in finally_block {
                        self.visit_statement(stmt)?;
                    }
                }
            }

            Statement::EmptyStatement { .. } => {}
        }
        Ok(())
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), JsError> {
        match expr {
            Expression::Identifier { name, .. } => {
                match self.resolve_variable(name) {
                    VarResolution::Local(reg) => {
                        self.chunk.emit(Opcode::LoadLocal(reg));
                    }
                    VarResolution::Upvalue(idx) => {
                        self.chunk.emit(Opcode::LoadUpvalue(idx));
                    }
                    VarResolution::Global => {
                        self.chunk.emit(Opcode::LoadGlobal(name.clone()));
                    }
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
                Literal::BigInt(_s) => {
                    // TODO: Implement BigInt support in bytecode
                    unimplemented!("BigInt literals not yet supported in bytecode")
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
                        // Logical NOT - invert truthiness
                        self.chunk.emit(Opcode::Not);
                    }
                    UnaryOperator::Typeof => {
                        // typeof operator - returns type as string
                        self.chunk.emit(Opcode::Typeof);
                    }
                    UnaryOperator::Void => {
                        // void operator - discard value and push undefined
                        self.chunk.emit(Opcode::Void);
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
                // Handle update expressions (++i, i++, --i, i--)
                if let Expression::Identifier { name, .. } = argument.as_ref() {
                    if let Some(&reg) = self.locals.get(name) {
                        if *prefix {
                            // Prefix: ++i or --i
                            // 1. Load current value
                            // 2. Add/Sub 1
                            // 3. Store back
                            // 4. Result on stack is new value
                            self.visit_expression(argument)?;
                            let one_idx = self.chunk.add_constant(BytecodeValue::Number(1.0));
                            self.chunk.emit(Opcode::LoadConstant(one_idx));
                            match operator {
                                UpdateOperator::Increment => self.chunk.emit(Opcode::Add),
                                UpdateOperator::Decrement => self.chunk.emit(Opcode::Sub),
                            }
                            // Duplicate the new value so we have one for storage and one for return
                            self.chunk.emit(Opcode::Dup);
                            self.chunk.emit(Opcode::StoreLocal(reg));
                        } else {
                            // Postfix: i++ or i--
                            // 1. Load current value (for return)
                            // 2. Load current value again (for computation)
                            // 3. Add/Sub 1
                            // 4. Store new value back
                            // 5. Result on stack is old value
                            self.visit_expression(argument)?; // old value (for return)
                            self.visit_expression(argument)?; // old value (for computation)
                            let one_idx = self.chunk.add_constant(BytecodeValue::Number(1.0));
                            self.chunk.emit(Opcode::LoadConstant(one_idx));
                            match operator {
                                UpdateOperator::Increment => self.chunk.emit(Opcode::Add),
                                UpdateOperator::Decrement => self.chunk.emit(Opcode::Sub),
                            }
                            // Stack now has: [old_value, new_value]
                            // Store new value back to register
                            self.chunk.emit(Opcode::StoreLocal(reg));
                            // Stack now has: [old_value] - this is the return value
                        }
                    } else {
                        // Global variable - fallback to simpler approach
                        self.visit_expression(argument)?;
                        let one_idx = self.chunk.add_constant(BytecodeValue::Number(1.0));
                        self.chunk.emit(Opcode::LoadConstant(one_idx));
                        match operator {
                            UpdateOperator::Increment => self.chunk.emit(Opcode::Add),
                            UpdateOperator::Decrement => self.chunk.emit(Opcode::Sub),
                        }
                        self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                    }
                } else {
                    // Non-identifier argument (e.g., obj.prop++) - not fully implemented
                    self.visit_expression(argument)?;
                    let one_idx = self.chunk.add_constant(BytecodeValue::Number(1.0));
                    self.chunk.emit(Opcode::LoadConstant(one_idx));
                    match operator {
                        UpdateOperator::Increment => self.chunk.emit(Opcode::Add),
                        UpdateOperator::Decrement => self.chunk.emit(Opcode::Sub),
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
                match left {
                    AssignmentTarget::Identifier(name) => {
                        self.visit_expression(right)?;
                        match self.resolve_variable(name) {
                            VarResolution::Local(reg) => {
                                self.chunk.emit(Opcode::StoreLocal(reg));
                            }
                            VarResolution::Upvalue(idx) => {
                                self.chunk.emit(Opcode::StoreUpvalue(idx));
                            }
                            VarResolution::Global => {
                                self.chunk.emit(Opcode::StoreGlobal(name.clone()));
                            }
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
                            if *computed {
                                // Computed assignment: obj[key] = value
                                // SetIndex expects stack: [obj, key, value]
                                self.visit_expression(object)?;
                                self.visit_expression(property)?;
                                self.visit_expression(right)?;
                                self.chunk.emit(Opcode::SetIndex);
                            } else {
                                // Static assignment: obj.prop = value
                                // StoreProperty expects stack: [obj, value]
                                self.visit_expression(object)?;
                                self.visit_expression(right)?;
                                if let Expression::Identifier { name, .. } = property.as_ref() {
                                    self.chunk.emit(Opcode::StoreProperty(name.clone()));
                                }
                            }
                        }
                    }
                    AssignmentTarget::Pattern(_) => {
                        // Destructuring - simplified
                        self.visit_expression(right)?;
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
                // Check if this is a method call (callee is MemberExpression)
                if let Expression::MemberExpression {
                    object,
                    property,
                    computed,
                    ..
                } = callee.as_ref()
                {
                    // Method call: obj.method(args) - need to bind 'this' to obj
                    // Stack should be: [obj, method, arg1, arg2, ...]
                    self.visit_expression(object)?;

                    // Duplicate object so we have it for both property access and 'this'
                    self.chunk.emit(Opcode::Dup);

                    // Get the method
                    if *computed {
                        self.visit_expression(property)?;
                        self.chunk.emit(Opcode::GetIndex);
                    } else if let Expression::Identifier { name, .. } = property.as_ref() {
                        self.chunk.emit(Opcode::LoadProperty(name.clone()));
                    }

                    // Push arguments
                    for arg in arguments {
                        self.visit_expression(arg)?;
                    }

                    // CallMethod expects stack: [obj (this), method, arg1, arg2, ...]
                    // argc includes the arguments only (not 'this' or method)
                    self.chunk.emit(Opcode::CallMethod(arguments.len() as u8));
                } else {
                    // Regular function call
                    // Push callee first (it goes underneath the arguments on stack)
                    self.visit_expression(callee)?;

                    // Push arguments (they go on top of callee)
                    for arg in arguments {
                        self.visit_expression(arg)?;
                    }

                    // Call - dispatcher expects stack: [callee, arg1, arg2, ...]
                    self.chunk.emit(Opcode::Call(arguments.len() as u8));
                }
            }

            Expression::MemberExpression {
                object,
                property,
                computed,
                ..
            } => {
                self.visit_expression(object)?;

                if *computed {
                    // Computed access: obj[expr] - use GetIndex
                    self.visit_expression(property)?;
                    self.chunk.emit(Opcode::GetIndex);
                } else {
                    // Static access: obj.prop - use LoadProperty
                    if let Expression::Identifier { name, .. } = property.as_ref() {
                        self.chunk.emit(Opcode::LoadProperty(name.clone()));
                    }
                }
            }

            Expression::NewExpression {
                callee, arguments, ..
            } => {
                // Push constructor first (it goes underneath the arguments on stack)
                self.visit_expression(callee)?;
                for arg in arguments {
                    self.visit_expression(arg)?;
                }
                // Use CallNew to properly create new instance
                self.chunk.emit(Opcode::CallNew(arguments.len() as u8));
            }

            Expression::ArrayExpression { elements, .. } => {
                // Push all elements onto the stack first
                let mut element_count = 0;
                for elem in elements {
                    if let Some(el) = elem {
                        match el {
                            ArrayElement::Expression(e) => {
                                self.visit_expression(e)?;
                                element_count += 1;
                            }
                            ArrayElement::Spread(e) => {
                                // For spread, we push the array to be spread
                                // The VM should handle expanding it
                                self.visit_expression(e)?;
                                element_count += 1;
                            }
                        }
                    } else {
                        // Hole in array - push undefined
                        self.chunk.emit(Opcode::LoadUndefined);
                        element_count += 1;
                    }
                }
                // Create the array with the specified number of elements
                self.chunk.emit(Opcode::CreateArray(element_count));
            }

            Expression::ObjectExpression { properties, .. } => {
                self.chunk.emit(Opcode::CreateObject);

                for prop in properties {
                    if let ObjectProperty::Property { key, value, .. } = prop {
                        if let PropertyKey::Identifier(name) = key {
                            // Duplicate the object so StoreProperty doesn't consume it
                            self.chunk.emit(Opcode::Dup);
                            self.visit_expression(value)?;
                            self.chunk.emit(Opcode::StoreProperty(name.clone()));
                        }
                    }
                }
                // Object remains on stack after all properties are set
            }

            Expression::ArrowFunctionExpression { params, body, .. } => {
                // Create function bytecode with enclosing scope for closure support
                let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                let mut func_gen = BytecodeGenerator::with_enclosing(Box::new(current_gen));

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

                // Get the upvalues captured by this function
                let upvalues = func_gen.get_upvalues();

                // Get the compiled function bytecode
                let mut func_bytecode = func_gen.chunk.clone();

                // Collect any nested functions from the inner function
                let inner_nested = func_gen.take_nested_functions();

                // Restore the outer generator
                *self = *func_gen.enclosing.take().unwrap();

                // Add the compiled function to our nested functions list
                let func_idx = self.nested_functions.len();

                // Adjust indices in the function's bytecode for nested functions
                let inner_base_idx = func_idx + 1;
                Self::adjust_closure_indices(&mut func_bytecode, inner_base_idx);

                self.nested_functions.push(func_bytecode);

                // Also include any nested functions from the inner function
                // Adjust indices in these nested functions too
                let mut adjusted_inner_nested = inner_nested;
                for nested_chunk in &mut adjusted_inner_nested {
                    Self::adjust_closure_indices(nested_chunk, inner_base_idx);
                }
                self.nested_functions.extend(adjusted_inner_nested);

                self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));
            }

            Expression::FunctionExpression {
                name, params, body, ..
            } => {
                // Create function bytecode with enclosing scope for closure support
                let current_gen = std::mem::replace(self, BytecodeGenerator::new());
                let mut func_gen = BytecodeGenerator::with_enclosing(Box::new(current_gen));

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

                // Get the upvalues captured by this function
                let upvalues = func_gen.get_upvalues();

                // Get the compiled function bytecode
                let mut func_bytecode = func_gen.chunk.clone();

                // Collect any nested functions from the inner function
                let inner_nested = func_gen.take_nested_functions();

                // Restore the outer generator
                *self = *func_gen.enclosing.take().unwrap();

                // Add the compiled function to our nested functions list
                let func_idx = self.nested_functions.len();

                // Adjust indices in the function's bytecode for nested functions
                let inner_base_idx = func_idx + 1;
                Self::adjust_closure_indices(&mut func_bytecode, inner_base_idx);

                self.nested_functions.push(func_bytecode);

                // Also include any nested functions from the inner function
                // Adjust indices in these nested functions too
                let mut adjusted_inner_nested = inner_nested;
                for nested_chunk in &mut adjusted_inner_nested {
                    Self::adjust_closure_indices(nested_chunk, inner_base_idx);
                }
                self.nested_functions.extend(adjusted_inner_nested);

                self.chunk.emit(Opcode::CreateClosure(func_idx, upvalues));
            }

            Expression::ThisExpression { .. } => {
                self.chunk.emit(Opcode::LoadGlobal("this".to_string()));
            }

            Expression::SuperExpression { .. } => {
                self.chunk.emit(Opcode::LoadGlobal("super".to_string()));
            }

            Expression::AwaitExpression { argument, .. } => {
                self.visit_expression(argument)?;
                // Emit Await opcode to suspend execution until promise resolves
                self.chunk.emit(Opcode::Await);
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
                | Opcode::JumpIfFalse(ref mut addr)
                | Opcode::PushTry(ref mut addr)
                | Opcode::PushFinally(ref mut addr) => {
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

    #[test]
    fn test_function_declaration_collects_nested() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::FunctionDeclaration {
            name: "add".to_string(),
            params: vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ],
            body: vec![Statement::ReturnStatement {
                argument: Some(Expression::BinaryExpression {
                    left: Box::new(Expression::Identifier {
                        name: "a".to_string(),
                        position: None,
                    }),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier {
                        name: "b".to_string(),
                        position: None,
                    }),
                    position: None,
                }),
                position: None,
            }],
            is_async: false,
            is_generator: false,
            position: None,
        }]);

        let _chunk = gen.generate(&ast).unwrap();

        // Verify that nested functions were collected
        let nested = gen.nested_functions();
        assert_eq!(nested.len(), 1, "Expected 1 nested function");

        // The nested function should have bytecode
        assert!(!nested[0].instructions.is_empty());
    }

    #[test]
    fn test_multiple_function_declarations() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![
            Statement::FunctionDeclaration {
                name: "first".to_string(),
                params: vec![],
                body: vec![Statement::ReturnStatement {
                    argument: Some(Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    }),
                    position: None,
                }],
                is_async: false,
                is_generator: false,
                position: None,
            },
            Statement::FunctionDeclaration {
                name: "second".to_string(),
                params: vec![],
                body: vec![Statement::ReturnStatement {
                    argument: Some(Expression::Literal {
                        value: Literal::Number(2.0),
                        position: None,
                    }),
                    position: None,
                }],
                is_async: false,
                is_generator: false,
                position: None,
            },
        ]);

        let _chunk = gen.generate(&ast).unwrap();
        let nested = gen.nested_functions();
        assert_eq!(nested.len(), 2, "Expected 2 nested functions");
    }

    #[test]
    fn test_arrow_function_collects_nested() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::VariableDeclaration {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier("square".to_string()),
                init: Some(Expression::ArrowFunctionExpression {
                    params: vec![Pattern::Identifier("x".to_string())],
                    body: ArrowFunctionBody::Expression(Box::new(Expression::BinaryExpression {
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: None,
                        }),
                        operator: BinaryOperator::Mul,
                        right: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: None,
                        }),
                        position: None,
                    })),
                    is_async: false,
                    position: None,
                }),
            }],
            position: None,
        }]);

        let _chunk = gen.generate(&ast).unwrap();
        let nested = gen.nested_functions();
        assert_eq!(nested.len(), 1, "Expected 1 nested function from arrow");
    }

    #[test]
    fn test_function_expression_collects_nested() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::VariableDeclaration {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier("multiply".to_string()),
                init: Some(Expression::FunctionExpression {
                    name: None,
                    params: vec![
                        Pattern::Identifier("x".to_string()),
                        Pattern::Identifier("y".to_string()),
                    ],
                    body: vec![Statement::ReturnStatement {
                        argument: Some(Expression::BinaryExpression {
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: None,
                            }),
                            operator: BinaryOperator::Mul,
                            right: Box::new(Expression::Identifier {
                                name: "y".to_string(),
                                position: None,
                            }),
                            position: None,
                        }),
                        position: None,
                    }],
                    is_async: false,
                    is_generator: false,
                    position: None,
                }),
            }],
            position: None,
        }]);

        let _chunk = gen.generate(&ast).unwrap();
        let nested = gen.nested_functions();
        assert_eq!(
            nested.len(),
            1,
            "Expected 1 nested function from function expression"
        );
    }

    #[test]
    fn test_take_nested_functions() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::FunctionDeclaration {
            name: "test".to_string(),
            params: vec![],
            body: vec![Statement::ReturnStatement {
                argument: Some(Expression::Literal {
                    value: Literal::Number(42.0),
                    position: None,
                }),
                position: None,
            }],
            is_async: false,
            is_generator: false,
            position: None,
        }]);

        let _chunk = gen.generate(&ast).unwrap();

        // take_nested_functions should consume the nested functions
        let nested = gen.take_nested_functions();
        assert_eq!(nested.len(), 1);

        // After taking, nested_functions should be empty
        assert_eq!(gen.nested_functions().len(), 0);
    }

    #[test]
    fn test_array_literal_generates_create_array() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::ArrayExpression {
                elements: vec![
                    Some(ArrayElement::Expression(Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    })),
                    Some(ArrayElement::Expression(Expression::Literal {
                        value: Literal::Number(2.0),
                        position: None,
                    })),
                    Some(ArrayElement::Expression(Expression::Literal {
                        value: Literal::Number(3.0),
                        position: None,
                    })),
                ],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should have CreateArray(3) opcode
        let has_create_array = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CreateArray(3))
        });
        assert!(has_create_array, "Expected CreateArray(3) opcode for array literal [1, 2, 3]");
    }

    #[test]
    fn test_empty_array_generates_create_array_zero() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::ArrayExpression {
                elements: vec![],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_create_array = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CreateArray(0))
        });
        assert!(has_create_array, "Expected CreateArray(0) for empty array");
    }

    #[test]
    fn test_array_with_holes_generates_create_array() {
        let mut gen = BytecodeGenerator::new();
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::ArrayExpression {
                elements: vec![
                    Some(ArrayElement::Expression(Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    })),
                    None, // hole
                    Some(ArrayElement::Expression(Expression::Literal {
                        value: Literal::Number(3.0),
                        position: None,
                    })),
                ],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should have LoadUndefined for the hole
        let has_load_undefined = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::LoadUndefined)
        });
        assert!(has_load_undefined, "Expected LoadUndefined for array hole");

        // Should have CreateArray(3)
        let has_create_array = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CreateArray(3))
        });
        assert!(has_create_array, "Expected CreateArray(3) for array with hole");
    }

    #[test]
    fn test_computed_member_access_generates_get_index() {
        let mut gen = BytecodeGenerator::new();
        // arr[0]
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::MemberExpression {
                object: Box::new(Expression::Identifier {
                    name: "arr".to_string(),
                    position: None,
                }),
                property: Box::new(Expression::Literal {
                    value: Literal::Number(0.0),
                    position: None,
                }),
                computed: true,
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_get_index = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::GetIndex)
        });
        assert!(has_get_index, "Expected GetIndex opcode for arr[0]");
    }

    #[test]
    fn test_computed_member_access_with_string_key() {
        let mut gen = BytecodeGenerator::new();
        // obj["key"]
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::MemberExpression {
                object: Box::new(Expression::Identifier {
                    name: "obj".to_string(),
                    position: None,
                }),
                property: Box::new(Expression::Literal {
                    value: Literal::String("key".to_string()),
                    position: None,
                }),
                computed: true,
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_get_index = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::GetIndex)
        });
        assert!(has_get_index, "Expected GetIndex opcode for obj[\"key\"]");
    }

    #[test]
    fn test_static_member_access_generates_load_property() {
        let mut gen = BytecodeGenerator::new();
        // obj.prop
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::MemberExpression {
                object: Box::new(Expression::Identifier {
                    name: "obj".to_string(),
                    position: None,
                }),
                property: Box::new(Expression::Identifier {
                    name: "prop".to_string(),
                    position: None,
                }),
                computed: false,
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_load_property = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::LoadProperty(ref s) if s == "prop")
        });
        assert!(has_load_property, "Expected LoadProperty(\"prop\") for obj.prop");
    }

    #[test]
    fn test_computed_assignment_generates_set_index() {
        let mut gen = BytecodeGenerator::new();
        // arr[0] = value
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::AssignmentExpression {
                left: AssignmentTarget::Member(Box::new(Expression::MemberExpression {
                    object: Box::new(Expression::Identifier {
                        name: "arr".to_string(),
                        position: None,
                    }),
                    property: Box::new(Expression::Literal {
                        value: Literal::Number(0.0),
                        position: None,
                    }),
                    computed: true,
                    optional: false,
                    position: None,
                })),
                operator: AssignmentOperator::Assign,
                right: Box::new(Expression::Literal {
                    value: Literal::Number(42.0),
                    position: None,
                }),
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_set_index = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::SetIndex)
        });
        assert!(has_set_index, "Expected SetIndex opcode for arr[0] = value");
    }

    #[test]
    fn test_computed_string_assignment_generates_set_index() {
        let mut gen = BytecodeGenerator::new();
        // obj["key"] = value
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::AssignmentExpression {
                left: AssignmentTarget::Member(Box::new(Expression::MemberExpression {
                    object: Box::new(Expression::Identifier {
                        name: "obj".to_string(),
                        position: None,
                    }),
                    property: Box::new(Expression::Literal {
                        value: Literal::String("key".to_string()),
                        position: None,
                    }),
                    computed: true,
                    optional: false,
                    position: None,
                })),
                operator: AssignmentOperator::Assign,
                right: Box::new(Expression::Literal {
                    value: Literal::String("value".to_string()),
                    position: None,
                }),
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_set_index = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::SetIndex)
        });
        assert!(has_set_index, "Expected SetIndex opcode for obj[\"key\"] = value");
    }

    #[test]
    fn test_method_call_generates_call_method() {
        let mut gen = BytecodeGenerator::new();
        // obj.method(arg1, arg2)
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::CallExpression {
                callee: Box::new(Expression::MemberExpression {
                    object: Box::new(Expression::Identifier {
                        name: "obj".to_string(),
                        position: None,
                    }),
                    property: Box::new(Expression::Identifier {
                        name: "method".to_string(),
                        position: None,
                    }),
                    computed: false,
                    optional: false,
                    position: None,
                }),
                arguments: vec![
                    Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    },
                    Expression::Literal {
                        value: Literal::Number(2.0),
                        position: None,
                    },
                ],
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should have Dup (to preserve 'this')
        let has_dup = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::Dup)
        });
        assert!(has_dup, "Expected Dup opcode for method call (preserve 'this')");

        // Should have CallMethod(2)
        let has_call_method = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallMethod(2))
        });
        assert!(has_call_method, "Expected CallMethod(2) opcode for obj.method(arg1, arg2)");
    }

    #[test]
    fn test_method_call_no_args_generates_call_method() {
        let mut gen = BytecodeGenerator::new();
        // obj.method()
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::CallExpression {
                callee: Box::new(Expression::MemberExpression {
                    object: Box::new(Expression::Identifier {
                        name: "obj".to_string(),
                        position: None,
                    }),
                    property: Box::new(Expression::Identifier {
                        name: "method".to_string(),
                        position: None,
                    }),
                    computed: false,
                    optional: false,
                    position: None,
                }),
                arguments: vec![],
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_call_method = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallMethod(0))
        });
        assert!(has_call_method, "Expected CallMethod(0) opcode for obj.method()");
    }

    #[test]
    fn test_computed_method_call_generates_call_method() {
        let mut gen = BytecodeGenerator::new();
        // obj["method"](arg)
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::CallExpression {
                callee: Box::new(Expression::MemberExpression {
                    object: Box::new(Expression::Identifier {
                        name: "obj".to_string(),
                        position: None,
                    }),
                    property: Box::new(Expression::Literal {
                        value: Literal::String("method".to_string()),
                        position: None,
                    }),
                    computed: true,
                    optional: false,
                    position: None,
                }),
                arguments: vec![Expression::Literal {
                    value: Literal::Number(42.0),
                    position: None,
                }],
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should use GetIndex for computed property access
        let has_get_index = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::GetIndex)
        });
        assert!(has_get_index, "Expected GetIndex for computed method access");

        let has_call_method = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallMethod(1))
        });
        assert!(has_call_method, "Expected CallMethod(1) for obj[\"method\"](arg)");
    }

    #[test]
    fn test_regular_function_call_generates_call() {
        let mut gen = BytecodeGenerator::new();
        // func(arg1, arg2)
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::CallExpression {
                callee: Box::new(Expression::Identifier {
                    name: "func".to_string(),
                    position: None,
                }),
                arguments: vec![
                    Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    },
                    Expression::Literal {
                        value: Literal::Number(2.0),
                        position: None,
                    },
                ],
                optional: false,
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should NOT have CallMethod
        let has_call_method = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallMethod(_))
        });
        assert!(!has_call_method, "Regular function call should NOT use CallMethod");

        // Should have Call(2)
        let has_call = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::Call(2))
        });
        assert!(has_call, "Expected Call(2) opcode for func(arg1, arg2)");
    }

    #[test]
    fn test_constructor_call_generates_call_new() {
        let mut gen = BytecodeGenerator::new();
        // new Foo(arg1, arg2)
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::NewExpression {
                callee: Box::new(Expression::Identifier {
                    name: "Foo".to_string(),
                    position: None,
                }),
                arguments: vec![
                    Expression::Literal {
                        value: Literal::Number(1.0),
                        position: None,
                    },
                    Expression::Literal {
                        value: Literal::Number(2.0),
                        position: None,
                    },
                ],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_call_new = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallNew(2))
        });
        assert!(has_call_new, "Expected CallNew(2) opcode for new Foo(arg1, arg2)");
    }

    #[test]
    fn test_constructor_call_no_args_generates_call_new() {
        let mut gen = BytecodeGenerator::new();
        // new Foo()
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::NewExpression {
                callee: Box::new(Expression::Identifier {
                    name: "Foo".to_string(),
                    position: None,
                }),
                arguments: vec![],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        let has_call_new = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::CallNew(0))
        });
        assert!(has_call_new, "Expected CallNew(0) opcode for new Foo()");
    }

    #[test]
    fn test_constructor_call_does_not_use_regular_call() {
        let mut gen = BytecodeGenerator::new();
        // new Foo()
        let ast = ASTNode::Program(vec![Statement::ExpressionStatement {
            expression: Expression::NewExpression {
                callee: Box::new(Expression::Identifier {
                    name: "Foo".to_string(),
                    position: None,
                }),
                arguments: vec![],
                position: None,
            },
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should NOT have regular Call
        let has_regular_call = chunk.instructions.iter().any(|i| {
            matches!(i.opcode, Opcode::Call(_))
        });
        assert!(!has_regular_call, "Constructor call should NOT use regular Call opcode");
    }

    #[test]
    fn test_postfix_increment_stores_new_value() {
        let mut gen = BytecodeGenerator::new();
        // let i = 0; i++;
        let ast = ASTNode::Program(vec![
            Statement::VariableDeclaration {
                kind: VariableKind::Let,
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier("i".to_string()),
                    init: Some(Expression::Literal {
                        value: Literal::Number(0.0),
                        position: None,
                    }),
                }],
                position: None,
            },
            Statement::ExpressionStatement {
                expression: Expression::UpdateExpression {
                    operator: UpdateOperator::Increment,
                    argument: Box::new(Expression::Identifier {
                        name: "i".to_string(),
                        position: None,
                    }),
                    prefix: false,
                    position: None,
                },
                position: None,
            },
        ]);

        let chunk = gen.generate(&ast).unwrap();

        // Count StoreLocal operations - there should be at least 2:
        // 1. Initial variable assignment (i = 0)
        // 2. Storing incremented value back (i = i + 1)
        let store_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i.opcode, Opcode::StoreLocal(_)))
            .count();

        // Should have at least 2 StoreLocal operations
        assert!(
            store_count >= 2,
            "Expected at least 2 StoreLocal operations (init + increment), got {}",
            store_count
        );

        // Should have Add operation for increment
        let has_add = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::Add));
        assert!(has_add, "Expected Add opcode for increment");

        // Check that we have Load operations before Add (to compute i + 1)
        let has_load_constant = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::LoadConstant(_)));
        assert!(has_load_constant, "Expected LoadConstant(1) for increment");
    }

    #[test]
    fn test_for_loop_bytecode_structure() {
        let mut gen = BytecodeGenerator::new();
        // for (let i = 0; i < 3; i++) { }
        let ast = ASTNode::Program(vec![Statement::ForStatement {
            init: Some(ForInit::VariableDeclaration {
                kind: VariableKind::Let,
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier("i".to_string()),
                    init: Some(Expression::Literal {
                        value: Literal::Number(0.0),
                        position: None,
                    }),
                }],
            }),
            test: Some(Expression::BinaryExpression {
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: None,
                }),
                operator: BinaryOperator::Lt,
                right: Box::new(Expression::Literal {
                    value: Literal::Number(3.0),
                    position: None,
                }),
                position: None,
            }),
            update: Some(Expression::UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: None,
                }),
                prefix: false,
                position: None,
            }),
            body: Box::new(Statement::BlockStatement {
                body: vec![],
                position: None,
            }),
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // For loop must have:
        // 1. StoreLocal for init (i = 0)
        // 2. LoadLocal + LoadConstant + LessThan for test (i < 3)
        // 3. JumpIfFalse to exit
        // 4. Body (empty)
        // 5. LoadLocal + LoadConstant + Add + StoreLocal for update (i++)
        // 6. Jump back to test

        // Check for Jump (loop back)
        let jump_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i.opcode, Opcode::Jump(_)))
            .count();
        assert!(jump_count >= 1, "For loop must have at least one Jump (loop back)");

        // Check for JumpIfFalse (exit condition)
        let has_jump_if_false = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::JumpIfFalse(_)));
        assert!(has_jump_if_false, "For loop must have JumpIfFalse for exit condition");

        // Check for LessThan (test condition)
        let has_less_than = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::LessThan));
        assert!(has_less_than, "For loop test should have LessThan comparison");

        // Check for Add (increment)
        let has_add = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::Add));
        assert!(has_add, "For loop update (i++) should have Add operation");

        // CRITICAL: Check that StoreLocal appears AFTER Add for the increment
        // This ensures the new value is stored back, not the old value
        let mut add_idx = None;
        let mut store_after_add = false;

        for (idx, inst) in chunk.instructions.iter().enumerate() {
            if matches!(inst.opcode, Opcode::Add) {
                add_idx = Some(idx);
            }
            if let Some(ai) = add_idx {
                if idx > ai && matches!(inst.opcode, Opcode::StoreLocal(_)) {
                    store_after_add = true;
                    break;
                }
            }
        }

        assert!(
            store_after_add,
            "CRITICAL: StoreLocal must appear AFTER Add to store incremented value. This is the root cause of infinite loops!"
        );
    }

    #[test]
    fn test_prefix_increment_stores_new_value() {
        let mut gen = BytecodeGenerator::new();
        // let i = 0; ++i;
        let ast = ASTNode::Program(vec![
            Statement::VariableDeclaration {
                kind: VariableKind::Let,
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier("i".to_string()),
                    init: Some(Expression::Literal {
                        value: Literal::Number(0.0),
                        position: None,
                    }),
                }],
                position: None,
            },
            Statement::ExpressionStatement {
                expression: Expression::UpdateExpression {
                    operator: UpdateOperator::Increment,
                    argument: Box::new(Expression::Identifier {
                        name: "i".to_string(),
                        position: None,
                    }),
                    prefix: true,
                    position: None,
                },
                position: None,
            },
        ]);

        let chunk = gen.generate(&ast).unwrap();

        // For prefix increment (++i):
        // 1. Load i
        // 2. Load 1
        // 3. Add
        // 4. Store back to i (new value)
        // Result on stack: new value

        let store_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i.opcode, Opcode::StoreLocal(_)))
            .count();

        assert!(
            store_count >= 2,
            "Expected at least 2 StoreLocal (init + increment), got {}",
            store_count
        );

        // Check Add appears before StoreLocal (for the increment, not init)
        let mut saw_add = false;
        let mut store_after_add = false;
        for inst in &chunk.instructions {
            if matches!(inst.opcode, Opcode::Add) {
                saw_add = true;
            }
            if saw_add && matches!(inst.opcode, Opcode::StoreLocal(_)) {
                store_after_add = true;
                break;
            }
        }
        assert!(store_after_add, "Prefix increment must Store after Add");
    }

    #[test]
    fn debug_postfix_increment_bytecode() {
        let mut gen = BytecodeGenerator::new();
        // let i = 0; i++;
        let ast = ASTNode::Program(vec![
            Statement::VariableDeclaration {
                kind: VariableKind::Let,
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier("i".to_string()),
                    init: Some(Expression::Literal {
                        value: Literal::Number(0.0),
                        position: None,
                    }),
                }],
                position: None,
            },
            Statement::ExpressionStatement {
                expression: Expression::UpdateExpression {
                    operator: UpdateOperator::Increment,
                    argument: Box::new(Expression::Identifier {
                        name: "i".to_string(),
                        position: None,
                    }),
                    prefix: false,
                    position: None,
                },
                position: None,
            },
        ]);

        let chunk = gen.generate(&ast).unwrap();

        println!("\nBYTECODE FOR 'let i = 0; i++':");
        for (idx, inst) in chunk.instructions.iter().enumerate() {
            println!("{:3}: {:?}", idx, inst.opcode);
        }
        println!();

        // Verify: The correct sequence should be:
        // 0: LoadConstant(0) - load 0.0
        // 1: StoreLocal(r0) - i = 0
        // 2: LoadLocal(r0) - load i (old value)
        // 3: LoadConstant(1) - load 1.0
        // 4: Add - compute i + 1
        // 5: LoadLocal(r0) - WRONG! This loads old value again
        // 6: StoreLocal(r0) - stores old value back, NOT the result!
        // 7: Return

        // The bug: after computing i + 1, it loads i again and stores THAT
        // The incremented value is never stored back!
    }

    #[test]
    fn test_class_declaration_with_constructor() {
        let mut gen = BytecodeGenerator::new();
        // class Counter { constructor(start) { this.count = start; } }
        let ast = ASTNode::Program(vec![Statement::ClassDeclaration {
            name: "Counter".to_string(),
            super_class: None,
            body: vec![ClassElement::MethodDefinition {
                key: "constructor".to_string(),
                kind: MethodKind::Constructor,
                value: Expression::FunctionExpression {
                    name: None,
                    params: vec![Pattern::Identifier("start".to_string())],
                    body: vec![Statement::ExpressionStatement {
                        expression: Expression::AssignmentExpression {
                            left: AssignmentTarget::Member(Box::new(Expression::MemberExpression {
                                object: Box::new(Expression::ThisExpression { position: None }),
                                property: Box::new(Expression::Identifier {
                                    name: "count".to_string(),
                                    position: None,
                                }),
                                computed: false,
                                optional: false,
                                position: None,
                            })),
                            operator: AssignmentOperator::Assign,
                            right: Box::new(Expression::Identifier {
                                name: "start".to_string(),
                                position: None,
                            }),
                            position: None,
                        },
                        position: None,
                    }],
                    is_async: false,
                    is_generator: false,
                    position: None,
                },
                is_static: false,
            }],
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should have CreateClosure (for the constructor)
        let has_create_closure = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::CreateClosure(_, _)));
        assert!(
            has_create_closure,
            "Class declaration should create a closure for the constructor"
        );

        // Should have StoreGlobal("Counter") to bind the class name
        let has_store_global = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::StoreGlobal(ref s) if s == "Counter"));
        assert!(
            has_store_global,
            "Class declaration should store constructor as global with class name"
        );

        // Should have exactly one nested function (the constructor)
        let nested = gen.nested_functions();
        assert_eq!(
            nested.len(),
            1,
            "Expected 1 nested function (the constructor)"
        );

        // The nested function (constructor) should have bytecode
        assert!(
            !nested[0].instructions.is_empty(),
            "Constructor should have bytecode"
        );
    }

    #[test]
    fn test_class_declaration_without_constructor() {
        let mut gen = BytecodeGenerator::new();
        // class Empty { }
        let ast = ASTNode::Program(vec![Statement::ClassDeclaration {
            name: "Empty".to_string(),
            super_class: None,
            body: vec![],
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        // Should still have CreateClosure (for default constructor)
        let has_create_closure = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::CreateClosure(_, _)));
        assert!(
            has_create_closure,
            "Class without constructor should create default constructor"
        );

        // Should have StoreGlobal("Empty")
        let has_store_global = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::StoreGlobal(ref s) if s == "Empty"));
        assert!(
            has_store_global,
            "Class without constructor should still bind class name globally"
        );

        // Should have exactly one nested function (default constructor)
        let nested = gen.nested_functions();
        assert_eq!(
            nested.len(),
            1,
            "Expected 1 nested function (default constructor)"
        );
    }

    #[test]
    fn test_class_instantiation_generates_call_new() {
        let mut gen = BytecodeGenerator::new();
        // class Foo { constructor(x) { this.x = x; } }
        // let f = new Foo(5);
        let ast = ASTNode::Program(vec![
            Statement::ClassDeclaration {
                name: "Foo".to_string(),
                super_class: None,
                body: vec![ClassElement::MethodDefinition {
                    key: "constructor".to_string(),
                    kind: MethodKind::Constructor,
                    value: Expression::FunctionExpression {
                        name: None,
                        params: vec![Pattern::Identifier("x".to_string())],
                        body: vec![Statement::ExpressionStatement {
                            expression: Expression::AssignmentExpression {
                                left: AssignmentTarget::Member(Box::new(
                                    Expression::MemberExpression {
                                        object: Box::new(Expression::ThisExpression {
                                            position: None,
                                        }),
                                        property: Box::new(Expression::Identifier {
                                            name: "x".to_string(),
                                            position: None,
                                        }),
                                        computed: false,
                                        optional: false,
                                        position: None,
                                    },
                                )),
                                operator: AssignmentOperator::Assign,
                                right: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: None,
                                }),
                                position: None,
                            },
                            position: None,
                        }],
                        is_async: false,
                        is_generator: false,
                        position: None,
                    },
                    is_static: false,
                }],
                position: None,
            },
            Statement::VariableDeclaration {
                kind: VariableKind::Let,
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier("f".to_string()),
                    init: Some(Expression::NewExpression {
                        callee: Box::new(Expression::Identifier {
                            name: "Foo".to_string(),
                            position: None,
                        }),
                        arguments: vec![Expression::Literal {
                            value: Literal::Number(5.0),
                            position: None,
                        }],
                        position: None,
                    }),
                }],
                position: None,
            },
        ]);

        let chunk = gen.generate(&ast).unwrap();

        // Should have LoadGlobal("Foo") to load the constructor
        let has_load_global = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::LoadGlobal(ref s) if s == "Foo"));
        assert!(
            has_load_global,
            "new Foo() should LoadGlobal to get the constructor"
        );

        // Should have CallNew(1) to call constructor with 1 argument
        let has_call_new = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::CallNew(1)));
        assert!(has_call_new, "new Foo(5) should use CallNew(1)");

        // The sequence should be:
        // 1. CreateClosure (constructor)
        // 2. StoreGlobal("Foo")
        // 3. LoadGlobal("Foo")
        // 4. LoadConstant(5)
        // 5. CallNew(1)
        // 6. StoreLocal (for 'f')
    }

    #[test]
    fn test_class_constructor_bytecode_has_parameter() {
        let mut gen = BytecodeGenerator::new();
        // class Foo { constructor(x) { this.x = x; } }
        let ast = ASTNode::Program(vec![Statement::ClassDeclaration {
            name: "Foo".to_string(),
            super_class: None,
            body: vec![ClassElement::MethodDefinition {
                key: "constructor".to_string(),
                kind: MethodKind::Constructor,
                value: Expression::FunctionExpression {
                    name: None,
                    params: vec![Pattern::Identifier("x".to_string())],
                    body: vec![Statement::ExpressionStatement {
                        expression: Expression::AssignmentExpression {
                            left: AssignmentTarget::Member(Box::new(Expression::MemberExpression {
                                object: Box::new(Expression::ThisExpression { position: None }),
                                property: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: None,
                                }),
                                computed: false,
                                optional: false,
                                position: None,
                            })),
                            operator: AssignmentOperator::Assign,
                            right: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: None,
                            }),
                            position: None,
                        },
                        position: None,
                    }],
                    is_async: false,
                    is_generator: false,
                    position: None,
                },
                is_static: false,
            }],
            position: None,
        }]);

        let _chunk = gen.generate(&ast).unwrap();
        let nested = gen.nested_functions();
        assert_eq!(nested.len(), 1);

        // The constructor should have at least 1 register (for parameter 'x')
        assert!(
            nested[0].register_count >= 1,
            "Constructor should have at least 1 register for parameter"
        );

        // Constructor bytecode should contain operations for 'this.x = x'
        // This includes: LoadGlobal("this"), LoadLocal (for x), StoreProperty("x")
        let has_load_global_this = nested[0]
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::LoadGlobal(ref s) if s == "this"));
        assert!(
            has_load_global_this,
            "Constructor body should reference 'this'"
        );

        let has_store_property = nested[0]
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::StoreProperty(ref s) if s == "x"));
        assert!(
            has_store_property,
            "Constructor should store property 'x' on this"
        );
    }

    #[test]
    fn debug_class_declaration_bytecode() {
        let mut gen = BytecodeGenerator::new();
        // class Foo { constructor(x) { this.x = x; } }
        let ast = ASTNode::Program(vec![Statement::ClassDeclaration {
            name: "Foo".to_string(),
            super_class: None,
            body: vec![ClassElement::MethodDefinition {
                key: "constructor".to_string(),
                kind: MethodKind::Constructor,
                value: Expression::FunctionExpression {
                    name: None,
                    params: vec![Pattern::Identifier("x".to_string())],
                    body: vec![Statement::ExpressionStatement {
                        expression: Expression::AssignmentExpression {
                            left: AssignmentTarget::Member(Box::new(Expression::MemberExpression {
                                object: Box::new(Expression::ThisExpression { position: None }),
                                property: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: None,
                                }),
                                computed: false,
                                optional: false,
                                position: None,
                            })),
                            operator: AssignmentOperator::Assign,
                            right: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: None,
                            }),
                            position: None,
                        },
                        position: None,
                    }],
                    is_async: false,
                    is_generator: false,
                    position: None,
                },
                is_static: false,
            }],
            position: None,
        }]);

        let chunk = gen.generate(&ast).unwrap();

        println!("\nBYTECODE FOR 'class Foo {{ constructor(x) {{ this.x = x; }} }}':");
        println!("Main bytecode:");
        for (idx, inst) in chunk.instructions.iter().enumerate() {
            println!("{:3}: {:?}", idx, inst.opcode);
        }

        let nested = gen.nested_functions();
        println!("\nConstructor bytecode ({} nested functions):", nested.len());
        if !nested.is_empty() {
            for (idx, inst) in nested[0].instructions.iter().enumerate() {
                println!("{:3}: {:?}", idx, inst.opcode);
            }
        }
        println!();

        // Verify the fix: main bytecode should have CreateClosure + StoreGlobal("Foo")
        assert!(chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::CreateClosure(_, _))));
        assert!(chunk.instructions.iter().any(|i| matches!(i.opcode, Opcode::StoreGlobal(ref s) if s == "Foo")));
    }

    #[test]
    fn test_parsed_new_expression_uses_call_new() {
        // This test verifies the fix for the bug where `new Foo(5)` was generating
        // Call opcode instead of CallNew. The issue was that parse_new_expression()
        // was parsing the callee with parse_left_hand_side_expression(), which would
        // consume the (5) as a CallExpression instead of leaving it for NewExpression.
        use crate::Parser;

        let source = "new Foo(5);";
        let mut parser = Parser::new(source);
        let ast = parser.parse().expect("Failed to parse 'new Foo(5)'");

        let mut gen = BytecodeGenerator::new();
        let chunk = gen.generate(&ast).expect("Failed to generate bytecode");

        // Must have CallNew(1) for the constructor call
        let has_call_new = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::CallNew(1)));
        assert!(
            has_call_new,
            "new Foo(5) should generate CallNew(1), not Call(1)"
        );

        // Must NOT have regular Call for this expression
        let has_regular_call = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::Call(1)));
        assert!(
            !has_regular_call,
            "new Foo(5) should not generate Call(1)"
        );

        // Verify the sequence: LoadGlobal("Foo"), LoadConstant, CallNew(1)
        let has_load_global = chunk
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, Opcode::LoadGlobal(ref s) if s == "Foo"));
        assert!(
            has_load_global,
            "Should LoadGlobal('Foo') to get constructor"
        );
    }
}





