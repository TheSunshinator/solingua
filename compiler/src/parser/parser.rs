use crate::ast::{
    ArithmeticOperator, ComparisonOperator, ConditionBranch,
    Expression, LogicalOperator, Program, Statement, StringTemplatePart,
    BlueprintDeclaration, Declaration, FunctionDeclaration, Parameter,
};
use crate::lexer::{Span, SpannedToken, Token};
use crate::lexer::keyword::Keyword;
use crate::lexer::symbol::{Symbol, Bound, Arithmetic, Logical};
use crate::lexer::comparison::ComparisonOperator as LexerComparison;
use crate::lexer::literal::{Literal, StringLiteral, StringPart};
use crate::util::Trivalent;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    position: usize,
}

// === Binding powers for Pratt parsing ===
//
// Each infix operator returns (left_bp, right_bp).
// Left-associative:  right_bp = left_bp + 1
// Right-associative: right_bp = left_bp
// Non-associative:   right_bp = left_bp + 1 (and we don't loop)
//
// Precedence (low to high):
//   1/2  — ∧, ∨         (logical, left-assoc, same precedence per spec)
//   3/4  — comparisons   (non-associative)
//   5/6  — +, -          (additive, left-assoc)
//   7/8  — ×, /          (multiplicative, left-assoc)
//   11   — . (postfix)   (member access / method call)

fn infix_binding_power(token: &Token) -> Option<(u8, u8)> {
    match token {
        // Logical ∧ / ∨ — same precedence, left-associative
        Token::Symbol(Symbol::Logical(Logical::And)) |
        Token::Symbol(Symbol::Logical(Logical::Or)) => Some((1, 2)),

        // Comparisons — non-associative (left_bp = 3, right_bp = 4)
        Token::ComparisonOperator(_) => Some((3, 4)),

        // Additive — left-associative
        Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)) |
        Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)) => Some((5, 6)),

        // Multiplicative — left-associative
        Token::Symbol(Symbol::Arithmetic(Arithmetic::Times)) |
        Token::Symbol(Symbol::Arithmetic(Arithmetic::Divided)) => Some((7, 8)),

        // Postfix dot — member access / method call
        Token::Symbol(Symbol::Arrow) => Some((11, 12)),

        _ => None,
    }
}

fn prefix_binding_power(token: &Token) -> Option<u8> {
    match token {
        // ¬ (logical not) — tighter than ∧/∨ but looser than comparison
        Token::Symbol(Symbol::Logical(Logical::Not)) => Some(9),
        // Unary minus — same precedence as multiplicative
        Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)) => Some(9),
        _ => None,
    }
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            declarations.push(self.parse_declaration());
        }

        Program { declarations }
    }

    // === Declarations ===

    fn parse_declaration(&mut self) -> Declaration {
        self.expect_token(&Token::Keyword(Keyword::Let));
        let construct = self.advance().clone();

        match construct {
            Token::Symbol(Symbol::Function) => {
                let decl = self.parse_function_declaration();
                Declaration::Function(decl)
            }
            Token::Keyword(Keyword::Type) => {
                let decl = self.parse_type_declaration();
                Declaration::Blueprint(decl)
            }
            Token::Keyword(Keyword::Singleton) => {
                let name = self.expect_identifier();
                self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));
                let _labels = self.parse_label_declaration();
                // Skip instance section if present
                if self.check(&Token::Keyword(Keyword::Instance)) {
                    self.advance();
                }
                self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
                Declaration::Singleton(name)
            }
            other => {
                let span = self.current_span();
                panic!("{}:{}: Expected construct type (ƒ, type, singleton, ...), got {:?}",
                    span.line, span.column, other);
            }
        }
    }

    fn parse_function_declaration(&mut self) -> FunctionDeclaration {
        let name = self.expect_identifier();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        let labels = self.parse_label_declaration();

        let mut return_type = "nothing".to_string();
        for ld in &labels {
            if let super::label::Label::Return(Trivalent::Some(rt)) = &ld.label {
                return_type = rt.clone();
            }
        }

        // parameters section (bare keyword, no braces)
        let mut parameters = Vec::new();
        if self.check(&Token::Keyword(Keyword::Parameters)) {
            self.advance();
            // Parse parameter declarations until we hit 'body' or closing brace
            while self.check(&Token::Keyword(Keyword::Let)) {
                parameters.push(self.parse_parameter());
            }
        }

        // body section (bare keyword, no braces)
        let mut body = Vec::new();
        if self.check(&Token::Keyword(Keyword::Body)) {
            self.advance();
            // Parse statements until the closing brace of the function
            while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
                body.push(self.parse_statement());
                if self.check(&Token::Symbol(Symbol::Semicolon)) {
                    self.advance();
                }
            }
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        FunctionDeclaration { name, parameters, return_type, body }
    }

    fn parse_type_declaration(&mut self) -> BlueprintDeclaration {
        let name = self.expect_identifier();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        let labels = self.parse_label_declaration();

        // Extract label info
        let mut implements: Option<String> = None;
        let mut is_declared = false;
        let mut generic_params: Vec<String> = Vec::new();
        for ld in &labels {
            match &ld.label {
                super::label::Label::Return(Trivalent::Some(parent)) => {
                    implements = Some(parent.clone());
                }
                super::label::Label::Implementation(Trivalent::Some(impl_type)) => {
                    is_declared = impl_type == "none";
                }
                super::label::Label::Generics(Trivalent::Some(generics)) => {
                    generic_params = generics.clone();
                }
                _ => {}
            }
        }

        // parameters section
        let mut parameters = Vec::new();
        if self.check(&Token::Keyword(Keyword::Parameters)) {
            self.advance();
            while self.check(&Token::Keyword(Keyword::Let)) {
                // Peek ahead to distinguish `let value` from `let 𝑓`
                if self.is_next(&Token::Keyword(Keyword::Value)) {
                    parameters.push(self.parse_parameter());
                } else {
                    break;
                }
            }
        }

        // instance section
        let mut methods = Vec::new();
        if self.check(&Token::Keyword(Keyword::Instance)) {
            self.advance();
            while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing)))
                && !self.check(&Token::Keyword(Keyword::Project))
            {
                if self.check(&Token::Keyword(Keyword::Let)) {
                    if self.is_next(&Token::Symbol(Symbol::Function)) {
                        // let 𝑓 methodName { ... }
                        self.expect_token(&Token::Keyword(Keyword::Let));
                        self.expect_token(&Token::Symbol(Symbol::Function));
                        methods.push(self.parse_function_declaration());
                    } else if self.is_next(&Token::Keyword(Keyword::Value)) {
                        // let value name { ... } — instance value, parse as parameter
                        parameters.push(self.parse_parameter());
                    } else {
                        let span = self.current_span();
                        panic!("{}:{}: Expected 𝑓 or value after let in instance block",
                            span.line, span.column);
                    }
                } else {
                    // Loose statement in instance block (e.g. printLine(...))
                    // Skip for now — parse and discard
                    self.parse_statement();
                    if self.check(&Token::Symbol(Symbol::Semicolon)) {
                        self.advance();
                    }
                }
            }
        }

        // project section — nested types and other project-scoped declarations
        let mut nested_types = Vec::new();
        if self.check(&Token::Keyword(Keyword::Project)) {
            self.advance();
            while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
                self.expect_token(&Token::Keyword(Keyword::Let));
                if self.check(&Token::Keyword(Keyword::Type)) {
                    self.advance();
                    nested_types.push(self.parse_type_declaration());
                } else {
                    let span = self.current_span();
                    panic!("{}:{}: Expected type after let in project block",
                        span.line, span.column);
                }
            }
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        BlueprintDeclaration {
            name,
            parameters,
            methods,
            nested_types,
            is_declared,
            implements,
            generic_params,
        }
    }

    fn parse_parameter(&mut self) -> Parameter {
        self.expect_token(&Token::Keyword(Keyword::Let));
        self.expect_token(&Token::Keyword(Keyword::Value));
        let name = self.expect_identifier();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        let labels = self.parse_label_declaration();

        let mut type_name = "Unknown".to_string();
        for ld in &labels {
            if let super::label::Label::Return(Trivalent::Some(t)) = &ld.label {
                type_name = t.clone();
            }
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        Parameter { name, type_name }
    }

    // === Statements ===

    pub(crate) fn parse_statement(&mut self) -> Statement {
        if self.check(&Token::Keyword(Keyword::Return)) {
            return self.parse_return_statement();
        }

        if self.check(&Token::Keyword(Keyword::While)) {
            return self.parse_while_loop();
        }

        if self.check(&Token::Keyword(Keyword::If))
            && !self.is_next(&Token::Symbol(Symbol::Brace(Bound::Opening)))
        {
            return self.parse_if_statement();
        }

        if self.is_mutation_statement() {
            return self.parse_mutation_statement();
        }

        if self.is_local_declaration() {
            return self.parse_local_declaration();
        }

        let expression = self.parse_expression();
        Statement::ExpressionStatement(expression)
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::If));
        let condition = self.parse_expression();
        self.expect_token(&Token::Keyword(Keyword::Then));
        let body = self.parse_expression();
        Statement::IfStatement {
            condition,
            body: Box::new(body),
        }
    }

    fn parse_while_loop(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::While));
        let condition = self.parse_expression();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));
        let mut body = Vec::new();
        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            body.push(self.parse_statement());
            if self.check(&Token::Symbol(Symbol::Semicolon)) {
                self.advance();
            }
        }
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
        Statement::WhileLoop { condition, body }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::Return));
        let expression = self.parse_expression();
        Statement::ReturnStatement(expression)
    }

    fn is_mutation_statement(&self) -> bool {
        if self.current_is_name() {
            if self.position + 1 < self.tokens.len() {
                return self.tokens[self.position + 1].token == Token::Keyword(Keyword::Becomes);
            }
        }
        false
    }

    fn parse_mutation_statement(&mut self) -> Statement {
        let name = self.expect_identifier();
        self.expect_token(&Token::Keyword(Keyword::Becomes));
        let new_value = self.parse_expression();
        Statement::MutationStatement { name, new_value }
    }

    fn is_local_declaration(&self) -> bool {
        self.check(&Token::Keyword(Keyword::Let))
    }

    fn parse_local_declaration(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::Let));
        self.expect_token(&Token::Keyword(Keyword::Value));
        let name = self.expect_identifier();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        // Parse labels
        let labels = self.parse_label_declaration();

        let mut type_name = "Unknown".to_string();
        let type_argument: Option<String> = None;
        for ld in &labels {
            if let super::label::Label::Return(Trivalent::Some(t)) = &ld.label {
                type_name = t.clone();
            }
        }

        // Parse initially expression
        let mut assigned_value: Option<Expression> = None;
        if self.check(&Token::Keyword(Keyword::Initially)) {
            self.advance();
            assigned_value = Some(self.parse_expression());
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        Statement::ValueDeclaration {
            name,
            type_name,
            type_argument,
            assigned_value: assigned_value.unwrap_or(Expression::IntegerLiteral(0)),
        }
    }

    // === Expressions (Pratt parser) ===

    pub(crate) fn parse_expression(&mut self) -> Expression {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> Expression {
        // --- Prefix / atom ---
        let mut lhs = if let Some(prefix_bp) = prefix_binding_power(self.current()) {
            let op_token = self.advance().clone();
            let rhs = self.parse_expression_bp(prefix_bp);
            match op_token {
                Token::Symbol(Symbol::Logical(Logical::Not)) => {
                    Expression::LogicalNot(Box::new(rhs))
                }
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)) => {
                    // Unary minus: -x → 0 - x
                    Expression::Arithmetic {
                        left: Box::new(Expression::IntegerLiteral(0)),
                        operator: ArithmeticOperator::Subtract,
                        right: Box::new(rhs),
                    }
                }
                _ => unreachable!(),
            }
        } else {
            self.parse_primary_expression()
        };

        // --- Infix / postfix loop ---
        loop {
            let current = self.current().clone();

            let Some((left_bp, right_bp)) = infix_binding_power(&current) else {
                break;
            };

            if left_bp < min_bp {
                break;
            }

            // Postfix dot: member access / method call
            if current == Token::Symbol(Symbol::Arrow) {
                self.advance();
                let member = self.expect_identifier();

                if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Opening))) {
                    self.advance();
                    let arguments = self.parse_arguments();
                    self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
                    lhs = Expression::MethodCall {
                        object: Box::new(lhs),
                        method: member,
                        arguments,
                    };
                } else {
                    lhs = Expression::MemberAccess {
                        object: Box::new(lhs),
                        member,
                    };
                }
                continue;
            }

            // Consume the operator
            self.advance();
            let rhs = self.parse_expression_bp(right_bp);

            lhs = match &current {
                // Logical
                Token::Symbol(Symbol::Logical(Logical::And)) => Expression::LogicalBinary {
                    left: Box::new(lhs),
                    operator: LogicalOperator::And,
                    right: Box::new(rhs),
                },
                Token::Symbol(Symbol::Logical(Logical::Or)) => Expression::LogicalBinary {
                    left: Box::new(lhs),
                    operator: LogicalOperator::Or,
                    right: Box::new(rhs),
                },

                // Comparison
                Token::ComparisonOperator(op) => {
                    let operator = match op {
                        LexerComparison { checks_equality: true, negated: true, checks_smaller_than: true } => ComparisonOperator::GreaterThan,
                        LexerComparison { checks_equality: false, negated: true, checks_smaller_than: true } => ComparisonOperator::GreaterThanOrEqual,
                        LexerComparison { checks_equality: false, negated: false, checks_smaller_than: true } => ComparisonOperator::LessThan,
                        LexerComparison { checks_equality: true, negated: false, checks_smaller_than: true } => ComparisonOperator::LessThanOrEqual,
                        LexerComparison { checks_equality: true, negated: false, checks_smaller_than: false } => ComparisonOperator::Equal,
                        LexerComparison { checks_equality: true, negated: true, checks_smaller_than: false } => ComparisonOperator::NotEqual,
                        _ => {
                            let span = self.current_span();
                            panic!("{}:{}: Unknown comparison operator", span.line, span.column);
                        }
                    };
                    Expression::Comparison {
                        left: Box::new(lhs),
                        operator,
                        right: Box::new(rhs),
                    }
                }

                // Arithmetic
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)) => Expression::Arithmetic {
                    left: Box::new(lhs),
                    operator: ArithmeticOperator::Add,
                    right: Box::new(rhs),
                },
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)) => Expression::Arithmetic {
                    left: Box::new(lhs),
                    operator: ArithmeticOperator::Subtract,
                    right: Box::new(rhs),
                },
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Times)) => Expression::Arithmetic {
                    left: Box::new(lhs),
                    operator: ArithmeticOperator::Multiply,
                    right: Box::new(rhs),
                },
                Token::Symbol(Symbol::Arithmetic(Arithmetic::Divided)) => Expression::Arithmetic {
                    left: Box::new(lhs),
                    operator: ArithmeticOperator::Divide,
                    right: Box::new(rhs),
                },

                _ => unreachable!(),
            };
        }

        lhs
    }

    fn parse_primary_expression(&mut self) -> Expression {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Opening))) {
                    self.advance();
                    let arguments = self.parse_arguments();
                    self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
                    Expression::FunctionCall { name, arguments }
                } else {
                    Expression::ValueReference(name)
                }
            }
            Token::Literal(Literal::String(StringLiteral::Plain(value))) => {
                self.advance();
                Expression::StringLiteral(value)
            }
            Token::Literal(Literal::String(StringLiteral::Template(parts))) => {
                self.advance();
                Expression::StringTemplate {
                    parts: parts.into_iter().map(|p| match p {
                        StringPart::Text(s) => StringTemplatePart::Literal(s),
                        StringPart::Interpolation(s) => {
                            let mut sub_parser = Parser::new(
                                crate::lexer::Lexer::new(&s).tokenize(),
                            );
                            let expr = sub_parser.parse_expression();
                            StringTemplatePart::Expression(expr)
                        }
                    }).collect(),
                }
            }
            Token::Literal(Literal::Integer(value)) => {
                self.advance();
                Expression::IntegerLiteral(value)
            }
            Token::Literal(Literal::Float(value)) => {
                self.advance();
                Expression::FloatLiteral(value)
            }
            Token::Literal(Literal::Boolean(value)) => {
                self.advance();
                Expression::BooleanLiteral(value)
            }
            Token::Keyword(Keyword::If) => {
                self.parse_if_expression()
            }
            Token::Symbol(Symbol::Parentheses(Bound::Opening)) => {
                self.advance();
                let expression = self.parse_expression_bp(0);
                self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
                expression
            }
            other => {
                let span = self.current_span();
                panic!("{}:{}: Unexpected token in expression: {:?}", span.line, span.column, other);
            }
        }
    }

    fn parse_if_expression(&mut self) -> Expression {
        self.expect_token(&Token::Keyword(Keyword::If));

        // Inline form: `if condition then result else result`
        if !self.check(&Token::Symbol(Symbol::Brace(Bound::Opening))) {
            let condition = self.parse_expression();
            self.expect_token(&Token::Keyword(Keyword::Then));
            let then_value = self.parse_expression();

            self.expect_token(&Token::Keyword(Keyword::Else));
            let else_value = self.parse_expression();

            return Expression::IfExpression {
                branches: vec![ConditionBranch { condition, result: then_value }],
                else_branch: Box::new(else_value),
            };
        }

        // Braced form: `if { condition then result ... else then result }`
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        let mut branches = Vec::new();
        let mut else_branch = None;

        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            if self.check(&Token::Keyword(Keyword::Else)) {
                self.advance();
                self.expect_token(&Token::Keyword(Keyword::Then));
                let result = self.parse_expression();
                else_branch = Some(result);
            } else {
                let condition = self.parse_expression();
                self.expect_token(&Token::Keyword(Keyword::Then));
                let result = self.parse_primary_expression();
                branches.push(ConditionBranch { condition, result });
            }
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
        let else_branch = else_branch.expect("If expression must have an else branch");

        Expression::IfExpression {
            branches,
            else_branch: Box::new(else_branch),
        }
    }

    fn parse_arguments(&mut self) -> Vec<Expression> {
        let mut arguments = Vec::new();
        if !self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            arguments.push(self.parse_expression());
            while self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
                if !self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
                    arguments.push(self.parse_expression());
                }
            }
        }
        arguments
    }

    // === Helpers ===

    fn current_is_name(&self) -> bool {
        matches!(self.current(), Token::Identifier(_) | Token::Keyword(Keyword::Value) | Token::Keyword(Keyword::Type) | Token::Keyword(Keyword::None))
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.position].token
    }

    pub(crate) fn current_span(&self) -> Span {
        self.tokens[self.position].span
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.position].token;
        self.position += 1;
        token
    }

    pub(crate) fn check(&self, expected: &Token) -> bool {
        self.current() == expected
    }

    pub(crate) fn is_next(&self, expected: &Token) -> bool {
        if self.position + 1 < self.tokens.len() {
            &self.tokens[self.position + 1].token == expected
        } else {
            false
        }
    }

    pub(crate) fn expect_token(&mut self, expected: &Token) {
        if !self.check(expected) {
            let span = self.current_span();
            panic!(
                "{}:{}: Expected {:?}, got {:?}",
                span.line, span.column, expected, self.current()
            );
        }
        self.advance();
    }

    pub(crate) fn expect_identifier(&mut self) -> String {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            Token::Keyword(Keyword::Value) => { self.advance(); "value".to_string() }
            Token::Keyword(Keyword::Type) => { self.advance(); "type".to_string() }
            Token::Keyword(Keyword::None) => { self.advance(); "nothing".to_string() }
            other => {
                let span = self.current_span();
                panic!("{}:{}: Expected identifier, got {:?}", span.line, span.column, other);
            }
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::EndOfFile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expr(input: &str) -> Expression {
        let tokens = crate::lexer::Lexer::new(input).tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_expression()
    }

    // --- Atoms ---

    #[test]
    fn test_integer_literal() {
        assert_eq!(parse_expr("42"), Expression::IntegerLiteral(42));
    }

    #[test]
    fn test_boolean_literal() {
        assert_eq!(parse_expr("true"), Expression::BooleanLiteral(true));
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(parse_expr(r#""hello""#), Expression::StringLiteral("hello".to_string()));
    }

    #[test]
    fn test_value_reference() {
        assert_eq!(parse_expr("x"), Expression::ValueReference("x".to_string()));
    }

    #[test]
    fn test_function_call() {
        assert_eq!(parse_expr("foo(1, 2)"), Expression::FunctionCall {
            name: "foo".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(1),
                Expression::IntegerLiteral(2),
            ],
        });
    }

    // --- Arithmetic precedence ---

    #[test]
    fn test_addition() {
        // 1 + 2 → Add(1, 2)
        assert_eq!(parse_expr("1 + 2"), Expression::Arithmetic {
            left: Box::new(Expression::IntegerLiteral(1)),
            operator: ArithmeticOperator::Add,
            right: Box::new(Expression::IntegerLiteral(2)),
        });
    }

    #[test]
    fn test_multiplication_before_addition() {
        // 1 + 2 × 3 → Add(1, Mul(2, 3))
        assert_eq!(parse_expr("1 + 2 × 3"), Expression::Arithmetic {
            left: Box::new(Expression::IntegerLiteral(1)),
            operator: ArithmeticOperator::Add,
            right: Box::new(Expression::Arithmetic {
                left: Box::new(Expression::IntegerLiteral(2)),
                operator: ArithmeticOperator::Multiply,
                right: Box::new(Expression::IntegerLiteral(3)),
            }),
        });
    }

    #[test]
    fn test_left_associative_addition() {
        // 1 + 2 + 3 → Add(Add(1, 2), 3)
        assert_eq!(parse_expr("1 + 2 + 3"), Expression::Arithmetic {
            left: Box::new(Expression::Arithmetic {
                left: Box::new(Expression::IntegerLiteral(1)),
                operator: ArithmeticOperator::Add,
                right: Box::new(Expression::IntegerLiteral(2)),
            }),
            operator: ArithmeticOperator::Add,
            right: Box::new(Expression::IntegerLiteral(3)),
        });
    }

    #[test]
    fn test_parenthesized_expression() {
        // (1 + 2) × 3 → Mul(Add(1, 2), 3)
        assert_eq!(parse_expr("(1 + 2) × 3"), Expression::Arithmetic {
            left: Box::new(Expression::Arithmetic {
                left: Box::new(Expression::IntegerLiteral(1)),
                operator: ArithmeticOperator::Add,
                right: Box::new(Expression::IntegerLiteral(2)),
            }),
            operator: ArithmeticOperator::Multiply,
            right: Box::new(Expression::IntegerLiteral(3)),
        });
    }

    // --- Comparison ---

    #[test]
    fn test_comparison() {
        // a > b → Comparison(a, GreaterThan, b)
        assert_eq!(parse_expr("a > b"), Expression::Comparison {
            left: Box::new(Expression::ValueReference("a".to_string())),
            operator: ComparisonOperator::GreaterThan,
            right: Box::new(Expression::ValueReference("b".to_string())),
        });
    }

    #[test]
    fn test_comparison_with_arithmetic() {
        // a + 1 > b × 2 → Comparison(Add(a, 1), GreaterThan, Mul(b, 2))
        assert_eq!(parse_expr("a + 1 > b × 2"), Expression::Comparison {
            left: Box::new(Expression::Arithmetic {
                left: Box::new(Expression::ValueReference("a".to_string())),
                operator: ArithmeticOperator::Add,
                right: Box::new(Expression::IntegerLiteral(1)),
            }),
            operator: ComparisonOperator::GreaterThan,
            right: Box::new(Expression::Arithmetic {
                left: Box::new(Expression::ValueReference("b".to_string())),
                operator: ArithmeticOperator::Multiply,
                right: Box::new(Expression::IntegerLiteral(2)),
            }),
        });
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(parse_expr("a ≠ b"), Expression::Comparison {
            left: Box::new(Expression::ValueReference("a".to_string())),
            operator: ComparisonOperator::NotEqual,
            right: Box::new(Expression::ValueReference("b".to_string())),
        });
    }

    // --- Logical operators ---

    #[test]
    fn test_logical_and() {
        assert_eq!(parse_expr("a ∧ b"), Expression::LogicalBinary {
            left: Box::new(Expression::ValueReference("a".to_string())),
            operator: LogicalOperator::And,
            right: Box::new(Expression::ValueReference("b".to_string())),
        });
    }

    #[test]
    fn test_logical_not() {
        assert_eq!(parse_expr("¬a"), Expression::LogicalNot(
            Box::new(Expression::ValueReference("a".to_string())),
        ));
    }

    #[test]
    fn test_logical_not_binds_tighter_than_and() {
        // ¬a ∧ b → And(Not(a), b)
        assert_eq!(parse_expr("¬a ∧ b"), Expression::LogicalBinary {
            left: Box::new(Expression::LogicalNot(
                Box::new(Expression::ValueReference("a".to_string())),
            )),
            operator: LogicalOperator::And,
            right: Box::new(Expression::ValueReference("b".to_string())),
        });
    }

    #[test]
    fn test_comparison_binds_tighter_than_logical() {
        // x > 5 ∧ x < 10 → And(Cmp(x, >, 5), Cmp(x, <, 10))
        assert_eq!(parse_expr("x > 5 ∧ x < 10"), Expression::LogicalBinary {
            left: Box::new(Expression::Comparison {
                left: Box::new(Expression::ValueReference("x".to_string())),
                operator: ComparisonOperator::GreaterThan,
                right: Box::new(Expression::IntegerLiteral(5)),
            }),
            operator: LogicalOperator::And,
            right: Box::new(Expression::Comparison {
                left: Box::new(Expression::ValueReference("x".to_string())),
                operator: ComparisonOperator::LessThan,
                right: Box::new(Expression::IntegerLiteral(10)),
            }),
        });
    }

    #[test]
    fn test_and_or_same_precedence_left_assoc() {
        // a ∧ b ∨ c → Or(And(a, b), c)
        assert_eq!(parse_expr("a ∧ b ∨ c"), Expression::LogicalBinary {
            left: Box::new(Expression::LogicalBinary {
                left: Box::new(Expression::ValueReference("a".to_string())),
                operator: LogicalOperator::And,
                right: Box::new(Expression::ValueReference("b".to_string())),
            }),
            operator: LogicalOperator::Or,
            right: Box::new(Expression::ValueReference("c".to_string())),
        });
    }

    // --- Member access / method call ---

    #[test]
    fn test_member_access() {
        assert_eq!(parse_expr("foo → bar"), Expression::MemberAccess {
            object: Box::new(Expression::ValueReference("foo".to_string())),
            member: "bar".to_string(),
        });
    }

    #[test]
    fn test_method_call() {
        assert_eq!(parse_expr("foo → bar(1)"), Expression::MethodCall {
            object: Box::new(Expression::ValueReference("foo".to_string())),
            method: "bar".to_string(),
            arguments: vec![Expression::IntegerLiteral(1)],
        });
    }

    #[test]
    fn test_chained_member_access() {
        // a → b → c → MemberAccess(MemberAccess(a, b), c)
        assert_eq!(parse_expr("a → b → c"), Expression::MemberAccess {
            object: Box::new(Expression::MemberAccess {
                object: Box::new(Expression::ValueReference("a".to_string())),
                member: "b".to_string(),
            }),
            member: "c".to_string(),
        });
    }

    #[test]
    fn test_method_on_arithmetic_result() {
        // Tests that → binds tighter than +
        // a → size + 1 → Add(MemberAccess(a, size), 1)
        assert_eq!(parse_expr("a → size + 1"), Expression::Arithmetic {
            left: Box::new(Expression::MemberAccess {
                object: Box::new(Expression::ValueReference("a".to_string())),
                member: "size".to_string(),
            }),
            operator: ArithmeticOperator::Add,
            right: Box::new(Expression::IntegerLiteral(1)),
        });
    }
}
