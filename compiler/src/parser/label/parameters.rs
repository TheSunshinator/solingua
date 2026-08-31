use crate::lexer::{Keyword, Symbol, Token};
use crate::lexer::symbol::Bound;
use crate::parser::Parser;
use super::contract::Contract;
use super::implementation::Implementation;
use super::scope::Scope;
use super::r#type::TypeDescription;

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_description: TypeDescription,
    pub scope: Scope,
    pub implementation: Option<Implementation>,
    pub contract: Option<Contract>,
}

impl Parser {
    pub fn parse_label_parameters(&mut self) -> Option<Vec<Parameter>> {
        if !self.check(&Token::Keyword(Keyword::Parameters)) {
            return None;
        }
        self.advance(); // consume `parameters`
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        let mut parameters = Vec::new();

        while !self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            parameters.push(self.parse_single_parameter());
        }

        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        self.expect_token(&Token::Symbol(Symbol::Comma));

        Some(parameters)
    }

    fn parse_single_parameter(&mut self) -> Parameter {
        // Parse identifier
        let name = match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            }
            other => {
                let span = self.current_span();
                panic!(
                    "{}:{}: Expected parameter name, got {:?}",
                    span.line, span.column, other
                );
            }
        };

        // Expect `is value,`
        self.expect_token(&Token::Keyword(Keyword::Is));
        self.expect_token(&Token::Keyword(Keyword::Value));
        self.expect_token(&Token::Symbol(Symbol::Comma));

        // Parse type label (required)
        let type_description = self.parse_type_label(Keyword::Type)
            .unwrap_or_else(|| {
                let span = self.current_span();
                panic!(
                    "{}:{}: Expected `type` label for parameter '{}'",
                    span.line, span.column, name
                );
            });

        // Parse scope label (required)
        let scope = self.parse_label_scope()
            .unwrap_or_else(|| {
                let span = self.current_span();
                panic!(
                    "{}:{}: Expected scope label for parameter '{}'",
                    span.line, span.column, name
                );
            });

        // Parse optional implementation label
        let implementation = self.parse_label_implementation();

        // Parse optional contract label
        let contract = self.parse_label_contract();

        Parameter {
            name,
            type_description,
            scope,
            implementation,
            contract,
        }
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
    fn test_no_parameters_label() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_parameters(), None);
    }

    #[test]
    fn test_empty_parameters() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_parameters(), Some(vec![]));
    }

    #[test]
    fn test_single_parameter() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            // a is value, type Integer, local scope,
            Token::Identifier("a".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            //
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_parameters(), Some(vec![
            Parameter {
                name: "a".to_string(),
                type_description: TypeDescription::Some {
                    identifier: "Integer".to_string(),
                    generics: vec![],
                },
                scope: Scope::Local,
                implementation: None,
                contract: None,
            },
        ]));
    }

    #[test]
    fn test_multiple_parameters() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            // a is value, type Integer, local scope,
            Token::Identifier("a".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            // b is value, type String, local scope,
            Token::Identifier("b".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            //
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        let params = parser.parse_label_parameters().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "a");
        assert_eq!(params[1].name, "b");
        assert_eq!(params[1].type_description, TypeDescription::Some {
            identifier: "String".to_string(),
            generics: vec![],
        });
    }

    #[test]
    fn test_parameter_with_instance_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            // name is value, type String, instance scope,
            Token::Identifier("name".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Instance),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            //
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        let params = parser.parse_label_parameters().unwrap();
        assert_eq!(params[0].scope, Scope::Instance);
    }

    #[test]
    fn test_parameter_with_implementation_and_contract() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            // name is value, type String, instance scope, full implementation, no contract,
            Token::Identifier("name".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Instance),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Full),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::None),
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Comma),
            //
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        let params = parser.parse_label_parameters().unwrap();
        assert_eq!(params[0].implementation, Some(Implementation::Full));
        assert_eq!(params[0].contract, Some(Contract::None));
    }

    #[test]
    fn test_parameter_with_generic_type() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            // items is value, type List of String, local scope,
            Token::Identifier("items".to_string()),
            Token::Keyword(Keyword::Is),
            Token::Keyword(Keyword::Value),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Type),
            Token::Identifier("List".to_string()),
            Token::Keyword(Keyword::Of),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            //
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        let params = parser.parse_label_parameters().unwrap();
        assert_eq!(params[0].type_description, TypeDescription::Some {
            identifier: "List".to_string(),
            generics: vec!["String".to_string()],
        });
    }

    #[test]
    fn test_not_present_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::EndOfFile,
        ]);
        parser.parse_label_parameters();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Means));
    }

    #[test]
    #[should_panic(expected = "Expected parameter name")]
    fn test_invalid_parameter_name() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Parameters),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::If),
            Token::EndOfFile,
        ]);
        parser.parse_label_parameters();
    }
}
