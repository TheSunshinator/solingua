use crate::ast::{Expression, Statement};
use crate::lexer::{Keyword, Symbol, Token};
use crate::lexer::symbol::Bound;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Means {
    Block(Vec<Statement>),
    Expression(Expression),
}

impl Parser {
    pub fn parse_label_means(&mut self) -> Option<Means> {
        if !self.check(&Token::Keyword(Keyword::Means)) {
            return None;
        }
        self.advance(); // consume `means`

        if self.check(&Token::Symbol(Symbol::Brace(Bound::Opening))) {
            println!("Parsing block");
            self.advance(); // consume `{`
            let mut statements = Vec::new();

            while !self.check(&Token::Symbol(Symbol::Brace(Bound::Closing))) {
                statements.push(self.parse_statement());
            }

            self.expect_token(&Token::Symbol(Symbol::Brace(Bound::Closing)));
            Some(Means::Block(statements))
        } else {
            println!("Parsing expresion");
            let expression = self.parse_expression();
            Some(Means::Expression(expression))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expression;
    use crate::lexer::{SpannedToken, Span};
    use crate::lexer::literal::{Literal, StringLiteral};

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned)
    }

    #[test]
    fn test_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_means(), None);
    }

    #[test]
    fn test_not_present_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::EndOfFile,
        ]);
        parser.parse_label_means();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Full));
    }

    #[test]
    fn test_means_expression_integer() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(42)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Expression(Expression::IntegerLiteral(42))));
    }

    #[test]
    fn test_means_expression_string() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::String(StringLiteral::Plain("hello".to_string()))),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Expression(Expression::StringLiteral("hello".to_string()))));
    }

    #[test]
    fn test_means_expression_boolean() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Boolean(true)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Expression(Expression::BooleanLiteral(true))));
    }

    #[test]
    fn test_means_expression_variable_reference() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Identifier("x".to_string()),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Expression(Expression::ValueReference("x".to_string()))));
    }

    #[test]
    fn test_means_empty_block() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Symbol(Symbol::Brace(Bound::Opening)),
            Token::Symbol(Symbol::Brace(Bound::Closing)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Block(vec![])));
    }

    #[test]
    fn test_means_block_with_return_statement() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Symbol(Symbol::Brace(Bound::Opening)),
            Token::Keyword(Keyword::Return),
            Token::Literal(Literal::Integer(5)),
            Token::Symbol(Symbol::Brace(Bound::Closing)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Block(vec![
            Statement::ReturnStatement(Expression::IntegerLiteral(5)),
        ])));
    }

    #[test]
    fn test_means_block_with_expression_statement() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Symbol(Symbol::Brace(Bound::Opening)),
            // printLine("hello")
            Token::Identifier("printLine".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::String(StringLiteral::Plain("hello".to_string()))),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Brace(Bound::Closing)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means();
        assert_eq!(result, Some(Means::Block(vec![
            Statement::ExpressionStatement(Expression::FunctionCall {
                name: "printLine".to_string(),
                arguments: vec![Expression::StringLiteral("hello".to_string())],
            }),
        ])));
    }

    #[test]
    fn test_means_block_multiple_statements() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Symbol(Symbol::Brace(Bound::Opening)),
            // printLine("a")
            Token::Identifier("printLine".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::String(StringLiteral::Plain("a".to_string()))),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            // printLine("b")
            Token::Identifier("printLine".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::String(StringLiteral::Plain("b".to_string()))),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Brace(Bound::Closing)),
            Token::EndOfFile,
        ]);
        let result = parser.parse_label_means().unwrap();
        match result {
            Means::Block(statements) => assert_eq!(statements.len(), 2),
            _ => panic!("Expected block"),
        }
    }
}
