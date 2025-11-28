//! Abstract Syntax Tree node definitions

use core_types::SourcePosition;

/// AST node representing JavaScript program elements
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    /// Complete program
    Program(Vec<Statement>),
    /// Single statement
    Statement(Statement),
    /// Single expression
    Expression(Expression),
}

/// JavaScript statements
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable declaration (let, const, var)
    VariableDeclaration {
        /// Declaration kind (let, const, var)
        kind: VariableKind,
        /// List of declarators
        declarations: Vec<VariableDeclarator>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Function declaration
    FunctionDeclaration {
        /// Function name
        name: String,
        /// Parameter names
        params: Vec<Pattern>,
        /// Function body
        body: Vec<Statement>,
        /// Is async function
        is_async: bool,
        /// Is generator function
        is_generator: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Class declaration
    ClassDeclaration {
        /// Class name
        name: String,
        /// Superclass expression
        super_class: Option<Box<Expression>>,
        /// Class body
        body: Vec<ClassElement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Expression statement
    ExpressionStatement {
        /// The expression
        expression: Expression,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Return statement
    ReturnStatement {
        /// Return value
        argument: Option<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// If statement
    IfStatement {
        /// Condition
        test: Expression,
        /// Consequent block
        consequent: Box<Statement>,
        /// Alternate block
        alternate: Option<Box<Statement>>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// While loop
    WhileStatement {
        /// Loop condition
        test: Expression,
        /// Loop body
        body: Box<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// For loop
    ForStatement {
        /// Initialization
        init: Option<ForInit>,
        /// Condition
        test: Option<Expression>,
        /// Update expression
        update: Option<Expression>,
        /// Loop body
        body: Box<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// For...in loop
    ForInStatement {
        /// Left side (variable or pattern)
        left: ForInOfLeft,
        /// Object to iterate over
        right: Expression,
        /// Loop body
        body: Box<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// For...of loop
    ForOfStatement {
        /// Left side (variable or pattern)
        left: ForInOfLeft,
        /// Iterable to iterate over
        right: Expression,
        /// Loop body
        body: Box<Statement>,
        /// Is await for-of
        r#await: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Block statement
    BlockStatement {
        /// Block body
        body: Vec<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Empty statement
    EmptyStatement {
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Break statement
    BreakStatement {
        /// Optional label
        label: Option<String>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Continue statement
    ContinueStatement {
        /// Optional label
        label: Option<String>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Throw statement
    ThrowStatement {
        /// Exception to throw
        argument: Expression,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Try statement
    TryStatement {
        /// Try block
        block: Vec<Statement>,
        /// Catch clause
        handler: Option<CatchClause>,
        /// Finally block
        finalizer: Option<Vec<Statement>>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Do-while loop
    DoWhileStatement {
        /// Loop body
        body: Box<Statement>,
        /// Loop condition
        test: Expression,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Switch statement
    SwitchStatement {
        /// Discriminant expression
        discriminant: Expression,
        /// Case clauses
        cases: Vec<SwitchCase>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// With statement
    WithStatement {
        /// Object expression
        object: Expression,
        /// Body statement
        body: Box<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Debugger statement
    DebuggerStatement {
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Labeled statement
    LabeledStatement {
        /// Label name
        label: String,
        /// Body statement
        body: Box<Statement>,
        /// Source location
        position: Option<SourcePosition>,
    },
}

/// Switch case clause
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// Test expression (None for default case)
    pub test: Option<Expression>,
    /// Consequent statements
    pub consequent: Vec<Statement>,
}

/// JavaScript expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Identifier reference
    Identifier {
        /// Variable name
        name: String,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Literal value
    Literal {
        /// Literal value
        value: Literal,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Parenthesized expression - tracks that an expression was wrapped in parentheses
    /// This is important for distinguishing between `({x} = y)` (valid destructuring assignment)
    /// and `({x}) = y` (invalid - parenthesized expression cannot be assignment target)
    ParenthesizedExpression {
        /// The inner expression
        expression: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Binary operation
    BinaryExpression {
        /// Left operand
        left: Box<Expression>,
        /// Operator
        operator: BinaryOperator,
        /// Right operand
        right: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Unary operation
    UnaryExpression {
        /// Operator
        operator: UnaryOperator,
        /// Operand
        argument: Box<Expression>,
        /// Is prefix operator
        prefix: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Update expression (++, --)
    UpdateExpression {
        /// Operator
        operator: UpdateOperator,
        /// Operand
        argument: Box<Expression>,
        /// Is prefix operator
        prefix: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Logical expression (&&, ||, ??)
    LogicalExpression {
        /// Left operand
        left: Box<Expression>,
        /// Operator
        operator: LogicalOperator,
        /// Right operand
        right: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Assignment expression
    AssignmentExpression {
        /// Left-hand side
        left: AssignmentTarget,
        /// Operator
        operator: AssignmentOperator,
        /// Right-hand side
        right: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Conditional expression (ternary)
    ConditionalExpression {
        /// Condition
        test: Box<Expression>,
        /// Consequent
        consequent: Box<Expression>,
        /// Alternate
        alternate: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Function call
    CallExpression {
        /// Function being called
        callee: Box<Expression>,
        /// Arguments
        arguments: Vec<Expression>,
        /// Optional call (e.g., foo?.())
        optional: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Member access (obj.prop or obj[prop])
    MemberExpression {
        /// Object
        object: Box<Expression>,
        /// Property
        property: Box<Expression>,
        /// Is computed (bracket notation)
        computed: bool,
        /// Optional access (e.g., obj?.prop)
        optional: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// New expression
    NewExpression {
        /// Constructor
        callee: Box<Expression>,
        /// Arguments
        arguments: Vec<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Meta property (new.target, import.meta)
    MetaProperty {
        /// Meta (e.g., "new" or "import")
        meta: String,
        /// Property (e.g., "target" or "meta")
        property: String,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Array literal
    ArrayExpression {
        /// Elements
        elements: Vec<Option<ArrayElement>>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Object literal
    ObjectExpression {
        /// Properties
        properties: Vec<ObjectProperty>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Arrow function
    ArrowFunctionExpression {
        /// Parameters
        params: Vec<Pattern>,
        /// Body (expression or block)
        body: ArrowFunctionBody,
        /// Is async
        is_async: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Function expression
    FunctionExpression {
        /// Optional name
        name: Option<String>,
        /// Parameters
        params: Vec<Pattern>,
        /// Body
        body: Vec<Statement>,
        /// Is async
        is_async: bool,
        /// Is generator
        is_generator: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// This expression
    ThisExpression {
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Super expression
    SuperExpression {
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Await expression
    AwaitExpression {
        /// Argument
        argument: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Yield expression
    YieldExpression {
        /// Argument
        argument: Option<Box<Expression>>,
        /// Is delegated (yield*)
        delegate: bool,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Class expression
    ClassExpression {
        /// Class name (optional for expressions)
        name: Option<String>,
        /// Superclass
        super_class: Option<Box<Expression>>,
        /// Class body
        body: Vec<ClassElement>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Template literal
    TemplateLiteral {
        /// Quasis (string parts)
        quasis: Vec<TemplateElement>,
        /// Expressions (interpolated parts)
        expressions: Vec<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Spread element
    SpreadElement {
        /// Argument to spread
        argument: Box<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },

    /// Sequence expression (comma-separated)
    SequenceExpression {
        /// Expressions
        expressions: Vec<Expression>,
        /// Source location
        position: Option<SourcePosition>,
    },
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableKind {
    /// let declaration
    Let,
    /// const declaration
    Const,
    /// var declaration
    Var,
}

/// Variable declarator
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarator {
    /// Pattern (identifier or destructuring)
    pub id: Pattern,
    /// Initial value
    pub init: Option<Expression>,
}

/// Pattern for variable binding
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Simple identifier
    Identifier(String),
    /// Object destructuring
    ObjectPattern(Vec<ObjectPatternProperty>),
    /// Array destructuring
    ArrayPattern(Vec<Option<Pattern>>),
    /// Assignment pattern (with default value)
    AssignmentPattern {
        /// Left side
        left: Box<Pattern>,
        /// Default value
        right: Box<Expression>,
    },
    /// Rest element (...rest)
    RestElement(Box<Pattern>),
    /// Member expression target (for destructuring assignment, not parameters)
    /// Example: [obj.prop] = [1] or [arr[0]] = [1]
    MemberExpression(Box<Expression>),
}

/// Pattern key type - for object patterns
#[derive(Debug, Clone, PartialEq)]
pub enum PatternKey {
    /// Literal key (identifier, string, or number)
    Literal(String),
    /// Computed key (expression in brackets: [expr])
    Computed(Expression),
}

/// Object pattern property
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPatternProperty {
    /// Key (can be literal or computed)
    pub key: PatternKey,
    /// Value pattern
    pub value: Pattern,
    /// Is shorthand (e.g., { a } instead of { a: a })
    pub shorthand: bool,
}

/// Literal value
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Number
    Number(f64),
    /// BigInt (stored as string to preserve exact value)
    BigInt(String),
    /// String
    String(String),
    /// Boolean
    Boolean(bool),
    /// Null
    Null,
    /// Undefined
    Undefined,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Modulo
    Mod,
    /// Exponentiation
    Exp,
    /// Equality
    Eq,
    /// Inequality
    NotEq,
    /// Strict equality
    StrictEq,
    /// Strict inequality
    StrictNotEq,
    /// Less than
    Lt,
    /// Less than or equal
    LtEq,
    /// Greater than
    Gt,
    /// Greater than or equal
    GtEq,
    /// Bitwise AND
    BitwiseAnd,
    /// Bitwise OR
    BitwiseOr,
    /// Bitwise XOR
    BitwiseXor,
    /// Left shift
    LeftShift,
    /// Right shift
    RightShift,
    /// Unsigned right shift
    UnsignedRightShift,
    /// Instanceof
    Instanceof,
    /// In
    In,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    /// Negate
    Minus,
    /// Plus (type coercion)
    Plus,
    /// Logical NOT
    Not,
    /// Bitwise NOT
    BitwiseNot,
    /// Typeof
    Typeof,
    /// Void
    Void,
    /// Delete
    Delete,
}

/// Update operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdateOperator {
    /// Increment
    Increment,
    /// Decrement
    Decrement,
}

/// Logical operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOperator {
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Nullish coalescing
    NullishCoalesce,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentOperator {
    /// Simple assignment (=)
    Assign,
    /// Addition assignment (+=)
    AddAssign,
    /// Subtraction assignment (-=)
    SubAssign,
    /// Multiplication assignment (*=)
    MulAssign,
    /// Division assignment (/=)
    DivAssign,
    /// Modulo assignment (%=)
    ModAssign,
    /// Exponentiation assignment (**=)
    ExpAssign,
    /// Bitwise AND assignment (&=)
    BitAndAssign,
    /// Bitwise OR assignment (|=)
    BitOrAssign,
    /// Bitwise XOR assignment (^=)
    BitXorAssign,
    /// Left shift assignment (<<=)
    LeftShiftAssign,
    /// Right shift assignment (>>=)
    RightShiftAssign,
    /// Unsigned right shift assignment (>>>=)
    UnsignedRightShiftAssign,
    /// Logical AND assignment (&&=)
    LogicalAndAssign,
    /// Logical OR assignment (||=)
    LogicalOrAssign,
    /// Nullish coalescing assignment (??=)
    NullishCoalesceAssign,
}

/// Assignment target
#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    /// Simple identifier
    Identifier(String),
    /// Member expression
    Member(Box<Expression>),
    /// Destructuring pattern
    Pattern(Pattern),
}

/// For loop initialization
#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    /// Variable declaration
    VariableDeclaration {
        /// Kind
        kind: VariableKind,
        /// Declarations
        declarations: Vec<VariableDeclarator>,
    },
    /// Expression
    Expression(Expression),
}

/// Left side of for-in/for-of loop
#[derive(Debug, Clone, PartialEq)]
pub enum ForInOfLeft {
    /// Variable declaration (let x, const x, var x)
    VariableDeclaration {
        /// Kind
        kind: VariableKind,
        /// Binding pattern
        id: Pattern,
    },
    /// Existing variable or pattern
    Pattern(Pattern),
    /// Left-hand side expression (e.g., member expression like x.y or x[0])
    Expression(Expression),
}

/// Catch clause
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    /// Parameter
    pub param: Option<Pattern>,
    /// Body
    pub body: Vec<Statement>,
}

/// Class element
#[derive(Debug, Clone, PartialEq)]
pub enum ClassElement {
    /// Method definition
    MethodDefinition {
        /// Method name
        key: PropertyKey,
        /// Method kind
        kind: MethodKind,
        /// Value (function expression)
        value: Expression,
        /// Is static
        is_static: bool,
        /// Is private (#name)
        is_private: bool,
        /// Is computed (e.g., [expr])
        computed: bool,
    },
    /// Property definition
    PropertyDefinition {
        /// Property key
        key: PropertyKey,
        /// Initial value
        value: Option<Expression>,
        /// Is static
        is_static: bool,
        /// Is private (#name)
        is_private: bool,
        /// Is computed (e.g., [expr])
        computed: bool,
    },
}

/// Method kind
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MethodKind {
    /// Regular method
    Method,
    /// Getter
    Get,
    /// Setter
    Set,
    /// Constructor
    Constructor,
}

/// Array element (for spread support)
#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElement {
    /// Normal element
    Expression(Expression),
    /// Spread element
    Spread(Expression),
}

/// Object property
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectProperty {
    /// Property with key and value
    Property {
        /// Key
        key: PropertyKey,
        /// Value
        value: Expression,
        /// Is shorthand
        shorthand: bool,
        /// Is computed
        computed: bool,
    },
    /// Spread property
    SpreadElement(Expression),
}

/// Property key
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    /// Identifier key
    Identifier(String),
    /// String literal key
    String(String),
    /// Number literal key
    Number(f64),
    /// Computed key
    Computed(Expression),
}

/// Arrow function body
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowFunctionBody {
    /// Expression body
    Expression(Box<Expression>),
    /// Block body
    Block(Vec<Statement>),
}

/// Template literal element
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateElement {
    /// Raw string value
    pub raw: String,
    /// Cooked string value
    pub cooked: String,
    /// Is tail element
    pub tail: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_program() {
        let node = ASTNode::Program(vec![]);
        assert!(matches!(node, ASTNode::Program(_)));
    }

    #[test]
    fn test_variable_declaration() {
        let decl = Statement::VariableDeclaration {
            kind: VariableKind::Let,
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier("x".to_string()),
                init: Some(Expression::Literal {
                    value: Literal::Number(42.0),
                    position: None,
                }),
            }],
            position: None,
        };
        assert!(matches!(decl, Statement::VariableDeclaration { .. }));
    }

    #[test]
    fn test_binary_expression() {
        let expr = Expression::BinaryExpression {
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
        };
        assert!(matches!(expr, Expression::BinaryExpression { .. }));
    }

    #[test]
    fn test_arrow_function() {
        let expr = Expression::ArrowFunctionExpression {
            params: vec![Pattern::Identifier("x".to_string())],
            body: ArrowFunctionBody::Expression(Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: None,
            })),
            is_async: false,
            position: None,
        };
        assert!(matches!(expr, Expression::ArrowFunctionExpression { .. }));
    }
}
