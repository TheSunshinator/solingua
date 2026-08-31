use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Is {
    Value,
    Function,
    Blueprint,
    Container,
    Alias,
    Singleton,
}

impl Parser {
    pub fn parse_label_is(&mut self) -> Is {
        self.expect_token(&Token::Keyword(Keyword::Is));
        let token = self.advance().clone();
        let label_is = match token {
            Token::Keyword(Keyword::Value) => Is::Value,
            Token::Keyword(Keyword::Function) => Is::Function,
            Token::Keyword(Keyword::Blueprint) => Is::Blueprint,
            Token::Keyword(Keyword::Container) => Is::Container,
            Token::Keyword(Keyword::Alias) => Is::Alias,
            _ => {
                let span = self.current_span();
                panic!(
                    "{}:{}: Expected `is` label definition (value, function, blueprint, or container), got {:?}",
                    span.line, span.column, token
                );
            }
        };
        self.expect_token(&Token::Symbol(Symbol::Comma));
        label_is
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, SpannedToken, Span};

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned)
    }

    #[test]
    fn test_parse_is_value() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_is(), Is::Value);
    }

    #[test]
    fn test_parse_is_function() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Function),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_is(), Is::Function);
    }

    #[test]
    fn test_parse_is_blueprint() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Blueprint),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_is(), Is::Blueprint);
    }

    #[test]
    fn test_parse_is_container() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Container),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_is(), Is::Container);
    }

    #[test]
    #[should_panic(expected = "Expected `is` label definition")]
    fn test_parse_is_invalid() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::If),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        parser.parse_label_is();
    }
}
