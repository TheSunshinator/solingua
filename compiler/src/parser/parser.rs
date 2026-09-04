use crate::ast::{
    ArithmeticOperator, ComparisonOperator, ConditionBranch,
    Expression, LogicalOperator, Program, Statement, StringTemplatePart,
    Declaration, FunctionDeclaration, ValueDeclaration, Parameter,
};
use crate::lexer::{Span, SpannedToken, Token};
use crate::lexer::keyword::Keyword;
use crate::lexer::symbol::{Symbol, Bound, Arithmetic};
use crate::lexer::comparison::{ComparisonOperator as LexerComparison, ComparisonOrientation};
use crate::lexer::literal::{Literal, StringLiteral, StringPart};
use crate::util::Trivalent;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    position: usize,
    verbose: bool,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>, verbose: bool) -> Self {
        Parser { tokens, position: 0, verbose }
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
            Token::Keyword(Keyword::Function) => {
                let decl = self.parse_function_declaration();
                Declaration::Function(decl)
            }
            other => {
                let span = self.current_span();
                panic!("{}:{}: Expected construct type (function, value, ...), got {:?}",
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

        let mut parameters = Vec::new();
        if self.check(&Token::Keyword(Keyword::Parameters)) {
            self.advance();
            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));
            parameters = self.parse_parameter_list();
            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
        }

        let mut body = Vec::new();
        if self.check(&Token::Keyword(Keyword::Body)) {
            self.advance();
            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));
            body = self.parse_statement_list();
            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        FunctionDeclaration { name, parameters, return_type, body }
    }

    fn parse_parameter_list(&mut self) -> Vec<Parameter> {
        let mut parameters = Vec::new();

        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            self.expect_token(&Token::Keyword(Keyword::Let));
            let name = self.expect_identifier();
            self.expect_token(&Token::Keyword(Keyword::Value));
            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

            let labels = self.parse_label_declaration();

            let mut type_name = "Unknown".to_string();
            for ld in &labels {
                if let super::label::Label::Return(Trivalent::Some(t)) = &ld.label {
                    type_name = t.clone();
                }
            }

            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

            parameters.push(Parameter { name, type_name });
        }

        parameters
    }

    fn parse_statement_list(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            statements.push(self.parse_statement());
            // Optional semicolons between statements
            if self.check(&Token::Symbol(Symbol::Semicolon)) {
                self.advance();
            }
        }
        statements
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

        let mut type_name = "Unknown".to_string();
        let mut type_argument: Option<String> = None;
        let mut assigned_value: Option<Expression> = None;

        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            match self.current().clone() {
                Token::Keyword(Keyword::Labels) => {
                    self.advance();
                    self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Opening)));
                    while !self.check(&Token::Symbol(Symbol::Bracket(Bound::Closing))) {
                        match self.current().clone() {
                            Token::Keyword(Keyword::Type) => {
                                self.advance();
                                self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));
                                type_name = self.expect_identifier();
                                self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
                            }
                            Token::Symbol(Symbol::Comma) => { self.advance(); }
                            _ => { self.advance(); } // skip other labels for now
                        }
                    }
                    self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Closing)));
                }
                Token::Identifier(ref s) if s == "means" => {
                    self.advance();
                    assigned_value = Some(self.parse_expression());
                }
                other => {
                    let span = self.current_span();
                    panic!("{}:{}: Unexpected in value declaration: {:?}", span.line, span.column, other);
                }
            }
        }

        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        Statement::ValueDeclaration {
            name,
            type_name,
            type_argument,
            assigned_value: assigned_value.unwrap_or(Expression::IntegerLiteral(0)),
        }
    }

    // === Expressions ===

    pub(crate) fn parse_expression(&mut self) -> Expression {
        let mut left = self.parse_not_expression();

        loop {
            let operator = if self.check(&Token::Keyword(Keyword::And)) {
                Some(LogicalOperator::And)
            } else if self.check(&Token::Keyword(Keyword::Or)) {
                Some(LogicalOperator::Or)
            } else {
                None
            };

            if let Some(operator) = operator {
                self.advance();
                let right = self.parse_not_expression();
                left = Expression::LogicalBinary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        left
    }

    fn parse_not_expression(&mut self) -> Expression {
        if self.check(&Token::Keyword(Keyword::Not)) {
            self.advance();
            let operand = self.parse_not_expression();
            return Expression::LogicalNot(Box::new(operand));
        }
        self.parse_comparison_expression()
    }

    fn parse_comparison_expression(&mut self) -> Expression {
        let left = self.parse_additive_expression();

        let operator = match self.current() {
            Token::ComparisonOperator(LexerComparison { checks_equality: false, orientation: Some(ComparisonOrientation::GreaterThan) }) => Some(ComparisonOperator::GreaterThan),
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: Some(ComparisonOrientation::GreaterThan) }) => Some(ComparisonOperator::GreaterThanOrEqual),
            Token::ComparisonOperator(LexerComparison { checks_equality: false, orientation: Some(ComparisonOrientation::LessThan) }) => Some(ComparisonOperator::LessThan),
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: Some(ComparisonOrientation::LessThan) }) => Some(ComparisonOperator::LessThanOrEqual),
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: None }) => Some(ComparisonOperator::Equal),
            _ => None,
        };

        if let Some(operator) = operator {
            self.advance();
            let right = self.parse_additive_expression();
            return Expression::Comparison {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        left
    }

    fn parse_additive_expression(&mut self) -> Expression {
        let mut left = self.parse_multiplicative_expression();

        loop {
            let operator = if self.check(&Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus))) {
                Some(ArithmeticOperator::Add)
            } else if self.check(&Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus))) {
                Some(ArithmeticOperator::Subtract)
            } else {
                None
            };

            if let Some(operator) = operator {
                self.advance();
                let right = self.parse_multiplicative_expression();
                left = Expression::Arithmetic {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        left
    }

    fn parse_multiplicative_expression(&mut self) -> Expression {
        let mut left = self.parse_postfix_expression();

        loop {
            let operator = if self.check(&Token::Symbol(Symbol::Arithmetic(Arithmetic::Times))) {
                Some(ArithmeticOperator::Multiply)
            } else if self.check(&Token::Symbol(Symbol::Arithmetic(Arithmetic::Divided))) {
                Some(ArithmeticOperator::Divide)
            } else {
                None
            };

            if let Some(operator) = operator {
                self.advance();
                let right = self.parse_postfix_expression();
                left = Expression::Arithmetic {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        left
    }

    fn parse_postfix_expression(&mut self) -> Expression {
        let mut expression = self.parse_primary_expression();

        while self.check(&Token::Symbol(Symbol::Dot)) {
            self.advance();
            let member = self.expect_identifier();

            if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Opening))) {
                self.advance();
                let arguments = self.parse_arguments();
                self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
                expression = Expression::MethodCall {
                    object: Box::new(expression),
                    method: member,
                    arguments,
                };
            } else {
                expression = Expression::MemberAccess {
                    object: Box::new(expression),
                    member,
                };
            }
        }

        expression
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
                                false,
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
            Token::Literal(Literal::Boolean(value)) => {
                self.advance();
                Expression::BooleanLiteral(value)
            }
            Token::Keyword(Keyword::If) => {
                self.parse_if_expression()
            }
            Token::Symbol(Symbol::Parentheses(Bound::Opening)) => {
                self.advance();
                let expression = self.parse_expression();
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
            // Allow keywords used as identifiers in name positions
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
