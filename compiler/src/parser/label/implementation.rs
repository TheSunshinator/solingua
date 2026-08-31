use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Implementation {
    Full,
    Partial,
    None,
}

impl Parser {
    pub fn parse_label_implementation(&mut self) -> Option<Implementation> {
        let implementation = match self.current().clone() {
            Token::Keyword(Keyword::Full) => Some(Implementation::Full),
            Token::Keyword(Keyword::Partial) => Some(Implementation::Partial),
            Token::Keyword(Keyword::None) => Some(Implementation::None),
            _ => None,
        };

        if implementation.is_some() {
            self.advance();
            self.expect_token(&Token::Keyword(Keyword::Implementation));
            self.expect_token(&Token::Symbol(Symbol::Comma));
        }

        implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{SpannedToken, Span};

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned)
    }

    #[test]
    fn test_full_implementation() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_implementation(), Some(Implementation::Full));
    }

    #[test]
    fn test_partial_implementation() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Partial),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_implementation(), Some(Implementation::Partial));
    }

    #[test]
    fn test_no_implementation() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::None),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_implementation(), Some(Implementation::None));
    }

    #[test]
    fn test_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Project),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_implementation(), None);
    }

    #[test]
    fn test_not_present_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Project),
            Token::EndOfFile,
        ]);
        parser.parse_label_implementation();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Project));
    }

    #[test]
    fn test_identifier_is_not_implementation() {
        let mut parser = parser_from_tokens(vec![
            Token::Identifier("full".to_string()),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_implementation(), None);
    }
}
