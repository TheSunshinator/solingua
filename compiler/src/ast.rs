/// A complete Solingua program
#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

/// A top-level declaration
#[derive(Debug, Clone)]
pub enum Declaration {
    Function(FunctionDeclaration),
    Blueprint(BlueprintDeclaration),
    Container(ContainerDeclaration),
    Singleton(String),
    Value(ValueDeclaration),
}

/// A function declaration
#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: String,
    pub body: Vec<Statement>,
}

/// A value declaration
#[derive(Debug, Clone)]
pub struct ValueDeclaration {
    pub name: String,
    pub type_name: String,
    pub type_generics: Vec<String>,
    pub assigned_value: Expression,
    pub is_mutable: bool,
}

/// A container (data-only structure): `Name -> { parameters(...) computed_values } is container,`
#[derive(Debug, Clone)]
pub struct ContainerDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub computed_values: Vec<ComputedValue>,
}

/// A computed value inside a container
#[derive(Debug, Clone)]
pub struct ComputedValue {
    pub name: String,
    pub type_name: String,
    pub expression: Expression,
}

/// A blueprint (class or interface) declaration
#[derive(Debug, Clone)]
pub struct BlueprintDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub methods: Vec<FunctionDeclaration>,
    pub is_declared: bool,
    pub implements: Option<String>,
    pub generic_params: Vec<String>,
}

/// A function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}

/// A statement in a function body
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ExpressionStatement(Expression),
    ValueDeclaration {
        name: String,
        type_name: String,
        type_argument: Option<String>,
        assigned_value: Expression,
    },
    MutationStatement {
        name: String,
        new_value: Expression,
    },
    ReturnStatement(Expression),
    IfStatement {
        condition: Expression,
        body: Box<Expression>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
}

/// An expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    FunctionCall { name: String, arguments: Vec<Expression> },
    StringLiteral(String),
    StringTemplate { parts: Vec<StringTemplatePart> },
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    ValueReference(String),
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    IfExpression {
        branches: Vec<ConditionBranch>,
        else_branch: Box<Expression>,
    },
    Comparison {
        left: Box<Expression>,
        operator: ComparisonOperator,
        right: Box<Expression>,
    },
    Arithmetic {
        left: Box<Expression>,
        operator: ArithmeticOperator,
        right: Box<Expression>,
    },
    LogicalBinary {
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
    },
    LogicalNot(Box<Expression>),
}

/// A part of a string template
#[derive(Debug, Clone, PartialEq)]
pub enum StringTemplatePart {
    Literal(String),
    Expression(Expression),
}

/// A branch in an if expression: `condition -> result`
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionBranch {
    pub condition: Expression,
    pub result: Expression,
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
}

/// Arithmetic operators
#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}
