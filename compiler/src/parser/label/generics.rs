use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Generic {
    Specific,
    Types(Vec<GenericType>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericType {
    pub identifier: String,
    pub requirement: Option<String>,
}

impl Parser {
    pub fn parse_label_generic(&mut self) -> Option<Generic> {
        let label = match self.current().clone() {
            Token::Keyword(Keyword::Specific) => {
                self.advance();
                Some(Generic::Specific)
            }
            Token::Keyword(Keyword::Generics) => {
                self.advance();
                Some(Generic::Types(self.parse_generics_declaration()))
            }
            _ => None,
        };

        if label.is_some() {
            self.expect_token(&Token::Symbol(Symbol::Comma));
        }

        label
    }

    fn parse_generics_declaration(&mut self) -> Vec<GenericType> {
        let mut types = Vec::new();

        loop {
            let identifier = match self.current().clone() {
                Token::Identifier(name) => {
                    self.advance();
                    name
                }
                other => {
                    let span = self.current_span();
                    panic!(
                        "{}:{}: Expected generic type identifier, got {:?}",
                        span.line, span.column, other
                    );
                }
            };

            self.expect_token(&Token::Keyword(Keyword::Type));

            let requirement = match self.current().clone() {
                Token::Keyword(Keyword::None) => {
                    self.advance();
                    None
                }
                Token::Identifier(name) => {
                    self.advance();
                    Some(name)
                }
                other => {
                    let span = self.current_span();
                    panic!(
                        "{}:{}: Expected type requirement or `none` for generic '{}', got {:?}",
                        span.line, span.column, identifier, other
                    );
                }
            };

            types.push(GenericType { identifier, requirement });

            if self.check(&Token::Keyword(Keyword::And)) {
                self.advance();
            } else {
                break;
            }
        }

        types
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
    fn test_specific() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Specific),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_generic(), Some(Generic::Specific));
    }

    #[test]
    fn test_single_generic_no_requirement() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Identifier("T".to_string()),
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::None),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_generic(), Some(Generic::Types(vec![
            GenericType { identifier: "T".to_string(), requirement: None },
        ])));
    }

    #[test]
    fn test_single_generic_with_requirement() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Identifier("T".to_string()),
            Token::Keyword(Keyword::Type),
            Token::Identifier("Comparable".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_generic(), Some(Generic::Types(vec![
            GenericType { identifier: "T".to_string(), requirement: Some("Comparable".to_string()) },
        ])));
    }

    #[test]
    fn test_multiple_generics() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Identifier("K".to_string()),
            Token::Keyword(Keyword::Type),
            Token::Identifier("Hashable".to_string()),
            Token::Keyword(Keyword::And),
            Token::Identifier("V".to_string()),
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::None),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_generic(), Some(Generic::Types(vec![
            GenericType { identifier: "K".to_string(), requirement: Some("Hashable".to_string()) },
            GenericType { identifier: "V".to_string(), requirement: None },
        ])));
    }

    #[test]
    fn test_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_generic(), None);
    }

    #[test]
    fn test_not_present_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Local),
            Token::EndOfFile,
        ]);
        parser.parse_label_generic();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Local));
    }

    #[test]
    #[should_panic(expected = "Expected generic type identifier")]
    fn test_missing_identifier_after_generics() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        parser.parse_label_generic();
    }

    #[test]
    #[should_panic(expected = "Expected type requirement or `none`")]
    fn test_missing_requirement_after_type() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Identifier("T".to_string()),
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::If),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        parser.parse_label_generic();
    }
}
