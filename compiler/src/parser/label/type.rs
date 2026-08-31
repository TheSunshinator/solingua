use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDescription {
    None,
    Some {
        identifier: String,
        generics: Vec<String>,
    },
}

impl Parser {
    pub fn parse_type_label(&mut self, label_name: Keyword) -> Option<TypeDescription> {
        let Token::Keyword(k) = self.current().clone() else { return None; };
        if k != label_name { return None; }
        self.advance(); // consume `label_name`

        let description = match self.current().clone() {
            Token::Keyword(Keyword::None) => {
                self.advance();
                TypeDescription::None
            }
            Token::Identifier(identifier) => {
                self.advance();
                let generics = self.parse_of_generics();
                TypeDescription::Some { identifier, generics }
            }
            other => {
                let span = self.current_span();
                panic!(
                    "{}:{}: Expected type name or `none` after `{:?}`, got {:?}",
                    span.line, span.column, label_name, other
                );
            }
        };

        self.expect_token(&Token::Symbol(Symbol::Comma));
        Some(description)
    }

    fn parse_of_generics(&mut self) -> Vec<String> {
        if !self.check(&Token::Keyword(Keyword::Of)) {
            return Vec::new();
        }
        self.advance(); // consume `of`

        let mut generics = Vec::new();

        loop {
            match self.current().clone() {
                Token::Identifier(identifier) => {
                    self.advance();
                    generics.push(identifier);
                }
                _ => {
                    let span = self.current_span();
                    panic!(
                        "{}:{}: Expected type identifier in generic list, got {:?}",
                        span.line, span.column, self.current()
                    );
                }
            }

            // Check for `;` separator between multiple generic args
            if self.check(&Token::Keyword(Keyword::And)) {
                self.advance();
            } else {
                break;
            }
        }

        generics
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

    // --- type label ---

    #[test]
    fn test_type_label_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Local),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), None);
    }

    #[test]
    fn test_type_label_none() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::None),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), Some(TypeDescription::None));
    }

    #[test]
    fn test_type_label_simple_identifier() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), Some(TypeDescription::Some {
            identifier: "String".to_string(),
            generics: vec![],
        }));
    }

    #[test]
    fn test_type_label_with_single_generic() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("List".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), Some(TypeDescription::Some {
            identifier: "List".to_string(),
            generics: vec!["String".to_string()],
        }));
    }

    #[test]
    fn test_type_label_with_multiple_generics() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("Map".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Semicolon),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), Some(TypeDescription::Some {
            identifier: "Map".to_string(),
            generics: vec!["String".to_string(), "Integer".to_string()],
        }));
    }

    #[test]
    #[should_panic(expected = "Expected type name or `none`")]
    fn test_type_label_invalid_after_keyword() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Keyword(Keyword::If),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        parser.parse_type_label(Keyword::Type);
    }

    #[test]
    #[should_panic(expected = "Expected type identifier in generic list")]
    fn test_type_label_of_without_identifier() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("List".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        parser.parse_type_label(Keyword::Type);
    }

    // --- returns label ---

    #[test]
    fn test_returns_label_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Local),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Return), None);
    }

    #[test]
    fn test_returns_label_none() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Keyword(Keyword::None),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Return), Some(TypeDescription::None));
    }

    #[test]
    fn test_returns_label_identifier() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Return), Some(TypeDescription::Some {
            identifier: "Integer".to_string(),
            generics: vec![],
        }));
    }

    #[test]
    fn test_returns_label_generic_type() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Identifier("Option".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Return), Some(TypeDescription::Some {
            identifier: "Option".to_string(),
            generics: vec!["String".to_string()],
        }));
    }

    // --- used with any keyword ---

    #[test]
    fn test_does_not_consume_wrong_keyword() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        // Looking for `returns` but `type` is present — should return None and not advance
        assert_eq!(parser.parse_type_label(Keyword::Return), None);
        // Parser position unchanged — `type` is still current
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Type));
    }

    #[test]
    fn test_three_generics() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("Tuple".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Identifier("A".to_string()),
            Token::Symbol(Symbol::Semicolon),
            Token::Identifier("B".to_string()),
            Token::Symbol(Symbol::Semicolon),
            Token::Identifier("C".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_type_label(Keyword::Type), Some(TypeDescription::Some {
            identifier: "Tuple".to_string(),
            generics: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        }));
    }
}
