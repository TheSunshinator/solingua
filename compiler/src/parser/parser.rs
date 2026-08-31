use crate::ast::{
    ArithmeticOperator, ComparisonOperator, ConditionBranch,
    Expression, LogicalOperator, Program, Statement, StringTemplatePart,
};
use crate::lexer::{Span, SpannedToken, Token};
use crate::lexer::keyword::Keyword;
use crate::lexer::symbol::{Symbol, Bound, Arithmetic};
use crate::lexer::comparison::{ComparisonOperator as LexerComparison, ComparisonOrientation};
use crate::lexer::literal::{Literal, StringLiteral, StringPart};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, position: 0 }
    }

    pub fn parse_program(&mut self, verbose: bool) -> Program {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            let span_start = self.current_span();
            let name = self.expect_identifier();
            let is_label = self.parse_label_is();
            let decl = self.parse_declaration(name, is_label, span_start, verbose);
            declarations.push(decl);
        }

        match super::validation::validate_and_build_ast(declarations) {
            Ok(program) => program,
            Err(errors) => {
                for error in &errors {
                    eprintln!("  {}", error);
                }
                panic!("Validation failed with {} error(s)", errors.len());
            }
        }
    }

    fn current_is_name(&self) -> bool {
        matches!(
            self.current(),
            Token::Identifier(_)
                | Token::Keyword(Keyword::Value)
                | Token::Keyword(Keyword::Type)
                | Token::Keyword(Keyword::None)
        )
    }

    /// Parse a statement
    pub(crate) fn parse_statement(&mut self) -> Statement {
        println!("Parsing statement");

        // Return statement
        if self.check(&Token::Keyword(Keyword::Return)) {
            return self.parse_return_statement();
        }

        // While loop: `while condition { body }`
        if self.check(&Token::Keyword(Keyword::While)) {
            return self.parse_while_loop();
        }

        // Single-branch if statement: `if condition then body`
        if self.check(&Token::Keyword(Keyword::If))
            && !self.is_next(&Token::Symbol(Symbol::Brace(Bound::Opening)))
        {
            return self.parse_if_statement();
        }

        // Mutation: `name becomes expression`
        if self.is_mutation_statement() {
            return self.parse_mutation_statement();
        }

        // Local value declaration: `name is value, type T, local scope, means expr`
        if self.is_local_declaration() {
            return self.parse_local_declaration();
        }

        // Expression statement
        println!("Parsing expression statement");
        let expression = self.parse_expression();
        Statement::ExpressionStatement(expression)
    }

    /// Parse: `if condition then body`
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

    /// Parse: `while condition { body }`
    fn parse_while_loop(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::While));
        let condition = self.parse_expression();
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Opening)));

        let mut body = Vec::new();
        while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
            body.push(self.parse_statement());
        }
        self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));

        Statement::WhileLoop { condition, body }
    }

    /// Parse: `return expression`
    fn parse_return_statement(&mut self) -> Statement {
        self.expect_token(&Token::Keyword(Keyword::Return));
        let expression = self.parse_expression();
        Statement::ReturnStatement(expression)
    }

    /// Look ahead: `name is ...` (local declaration)
    fn is_local_declaration(&self) -> bool {
        if self.current_is_name() {
            if self.position + 1 < self.tokens.len() {
                return self.tokens[self.position + 1].token == Token::Keyword(Keyword::Is);
            }
        }
        false
    }

    /// Parse a local declaration as a statement
    fn parse_local_declaration(&mut self) -> Statement {
        let name = self.expect_identifier();
        let is_label = self.parse_label_is();

        // Parse the remaining labels to get type and means
        let type_desc = self.parse_type_label(Keyword::Type);
        let mutability = self.parse_label_mutable();
        if mutability.is_none() {
            let span = self.current_span();
            panic!(
                "{}:{}: Local declaration '{}' is missing mutability label (mutable/immutable)",
                span.line, span.column, name
            );
        }
        let _scope = self.parse_label_scope();
        let _implementation = self.parse_label_implementation();
        let _contract = self.parse_label_contract();
        let means = self.parse_label_means();

        let type_name = match &type_desc {
            Some(super::label::r#type::TypeDescription::Some { identifier, .. }) => identifier.clone(),
            _ => "Unknown".to_string(),
        };
        let type_argument = match &type_desc {
            Some(super::label::r#type::TypeDescription::Some { generics, .. }) if !generics.is_empty() => {
                Some(generics[0].clone())
            }
            _ => None,
        };

        let assigned_value = match means {
            Some(super::label::means::Means::Expression(expr)) => expr,
            Some(super::label::means::Means::Block(_)) => {
                let span = self.current_span();
                panic!("{}:{}: Local value cannot have a block body", span.line, span.column);
            }
            None => {
                let span = self.current_span();
                panic!("{}:{}: Local value '{}' is missing `means` label", span.line, span.column, name);
            }
        };

        Statement::ValueDeclaration {
            name,
            type_name,
            type_argument,
            assigned_value,
        }
    }

    /// Look ahead: `name becomes expr`
    fn is_mutation_statement(&self) -> bool {
        if self.current_is_name() {
            if self.position + 1 < self.tokens.len() {
                return self.tokens[self.position + 1].token == Token::Keyword(Keyword::Becomes);
            }
        }
        false
    }

    /// Parse: `name becomes expression`
    fn parse_mutation_statement(&mut self) -> Statement {
        let name = self.expect_identifier();
        self.expect_token(&Token::Keyword(Keyword::Becomes));
        let new_value = self.parse_expression();
        Statement::MutationStatement { name, new_value }
    }

    /// Parse an expression — lowest precedence: `and`/`or` (same level, left-associative)
    pub(crate) fn parse_expression(&mut self) -> Expression {
        println!("Parsing expression");

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

        println!("Expression: {:?}", left);

        left
    }

    /// Parse `not` (unary prefix, binds tighter than and/or but looser than comparison)
    fn parse_not_expression(&mut self) -> Expression {
        println!("Parsing `not` expression");

        if self.check(&Token::Keyword(Keyword::Not)) {
            self.advance();
            let operand = self.parse_not_expression();
            return Expression::LogicalNot(Box::new(operand));
        }
        self.parse_comparison_expression()
    }

    /// Parse comparison expressions
    fn parse_comparison_expression(&mut self) -> Expression {
        let left = self.parse_additive_expression();
        println!("Parsed additive expression: {:?}", left.clone());

        let operator = match self.current() {
            Token::ComparisonOperator(LexerComparison { checks_equality: false, orientation: Some(ComparisonOrientation::GreaterThan) }) => {
                Some(ComparisonOperator::GreaterThan)
            }
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: Some(ComparisonOrientation::GreaterThan) }) => {
                Some(ComparisonOperator::GreaterThanOrEqual)
            }
            Token::ComparisonOperator(LexerComparison { checks_equality: false, orientation: Some(ComparisonOrientation::LessThan) }) => {
                Some(ComparisonOperator::LessThan)
            }
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: Some(ComparisonOrientation::LessThan) }) => {
                Some(ComparisonOperator::LessThanOrEqual)
            }
            Token::ComparisonOperator(LexerComparison { checks_equality: true, orientation: None }) => {
                Some(ComparisonOperator::Equal)
            }
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

    /// Parse addition and subtraction (left-associative)
    fn parse_additive_expression(&mut self) -> Expression {
        let mut left = self.parse_multiplicative_expression();
        println!("Parsed multiplicative expression: {:?}", left.clone());

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

    /// Parse multiplication and division (left-associative)
    fn parse_multiplicative_expression(&mut self) -> Expression {
        let mut left = self.parse_postfix_expression();
        println!("Parsed postfix expression: {:?}", left.clone());

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

    /// Parse postfix expressions: member access (.) and method calls (.method())
    fn parse_postfix_expression(&mut self) -> Expression {
        let mut expression = self.parse_primary_expression();
        println!("Parsed primary expression: {:?}", expression.clone());
        println!("Current token: {:?}", self.current());

        while self.check(&Token::Symbol(Symbol::Dot)) {
            self.advance(); // consume '.'
            println!("Found dot");

            let member = self.expect_identifier();
            println!("Member: {:?}", member.clone());

            if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Opening))) {
                self.advance(); // consume '('
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

    /// Parse a primary expression (literals, identifiers, function calls, if)
    fn parse_primary_expression(&mut self) -> Expression {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                // Check if it's a function call
                if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Opening))) {
                    self.advance(); // consume '('
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
                self.build_string_template(parts)
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
                self.advance(); // consume '('
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

    /// Build a StringTemplate expression from pre-parsed template parts.
    /// Interpolation expressions are stored as raw strings and need sub-parsing.
    fn build_string_template(&mut self, parts: Vec<StringPart>) -> Expression {
        let mut template_parts = Vec::new();
        for part in parts {
            match part {
                StringPart::Text(text) => {
                    template_parts.push(StringTemplatePart::Literal(text));
                }
                StringPart::Interpolation(expr_text) => {
                    // Sub-lex and parse the interpolation expression
                    let mut sub_lexer = crate::lexer::Lexer::new(&expr_text);
                    let sub_tokens = sub_lexer.tokenize();
                    let mut sub_parser = Parser::new(sub_tokens);
                    let expr = sub_parser.parse_expression();
                    template_parts.push(StringTemplatePart::Expression(expr));
                }
            }
        }
        Expression::StringTemplate { parts: template_parts }
    }

    /// Parse: `if { condition then result ... else then result }`
    fn parse_if_expression(&mut self) -> Expression {
        self.expect_token(&Token::Keyword(Keyword::If));
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

    /// Parse comma-separated arguments
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

    // --- Helper methods ---

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
