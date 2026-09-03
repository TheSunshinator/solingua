use crate::lexer::keyword::Keyword;
use crate::lexer::literal::{Literal, StringLiteral};
use crate::lexer::symbol::{Symbol, Bound};
use crate::lexer::{Span, Token};
use crate::util::Trivalent;
use super::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    Return(Trivalent<String>),
    Generics(Trivalent<Vec<String>>),
    Mutability(Trivalent<bool>),
    Visibility(Trivalent<String>),
    Scope(Trivalent<String>),
    Implementation(Trivalent<String>),
    Contract(Trivalent<String>),
}

#[derive(Debug, Clone)]
pub struct LabelDefinition {
    pub label: Label,
    pub span: Span,
}

impl Parser {
    pub fn parse_label_declaration(&mut self) -> Vec<LabelDefinition> {
        self.expect_token(&Token::Keyword(Keyword::Labels));
        self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Opening)));

        let labels = vec![
            self.parse_label_return(),
            self.parse_label_generics(),
            self.parse_label_mutable(),
            self.parse_label_visibility(),
            self.parse_label_scope(),
            self.parse_label_implementation(),
            self.parse_label_contract(),
        ];

        self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Closing)));
        labels
    }

    fn parse_label_return(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Return)) {
            return LabelDefinition { label: Label::Return(Trivalent::NotApplicable), span };
        }
        self.advance();
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        let trivalent = if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            Trivalent::None
        } else {
            let value = self.expect_identifier();
            Trivalent::Some(value)
        };

        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        self.skip_comma();
        LabelDefinition { label: Label::Return(trivalent), span }
    }

    fn parse_label_mutable(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Mutable)) {
            return LabelDefinition { label: Label::Mutability(Trivalent::NotApplicable), span };
        }
        self.advance();
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        let trivalent = if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            Trivalent::None
        } else {
            let value = self.expect_label_value();
            Trivalent::Some(value == "true")
        };

        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        self.skip_comma();
        LabelDefinition { label: Label::Mutability(trivalent), span }
    }

    fn parse_label_visibility(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Visibility)) {
            return LabelDefinition { label: Label::Visibility(Trivalent::NotApplicable), span };
        }
        self.advance();
        let trivalent = self.parse_parenthesized_string();
        self.skip_comma();
        LabelDefinition { label: Label::Visibility(trivalent), span }
    }

    fn parse_label_scope(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Scope)) {
            return LabelDefinition { label: Label::Scope(Trivalent::NotApplicable), span };
        }
        self.advance();
        let trivalent = self.parse_parenthesized_string();
        self.skip_comma();
        LabelDefinition { label: Label::Scope(trivalent), span }
    }

    fn parse_label_generics(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Generics)) {
            return LabelDefinition { label: Label::Generics(Trivalent::NotApplicable), span };
        }
        self.advance();
        self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Opening)));

        let trivalent = if self.check(&Token::Symbol(Symbol::Bracket(Bound::Closing))) {
            Trivalent::None
        } else {
            let mut generics = vec![self.expect_identifier()];
            while self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
                if !self.check(&Token::Symbol(Symbol::Bracket(Bound::Closing))) {
                    generics.push(self.expect_identifier());
                }
            }
            Trivalent::Some(generics)
        };

        self.expect_token(&Token::Symbol(Symbol::Bracket(Bound::Closing)));
        self.skip_comma();
        LabelDefinition { label: Label::Generics(trivalent), span }
    }

    fn parse_label_implementation(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Implementation)) {
            return LabelDefinition { label: Label::Implementation(Trivalent::NotApplicable), span };
        }
        self.advance();
        let trivalent = self.parse_parenthesized_string();
        self.skip_comma();
        LabelDefinition { label: Label::Implementation(trivalent), span }
    }

    fn parse_label_contract(&mut self) -> LabelDefinition {
        let span = self.current_span();
        if !self.check(&Token::Keyword(Keyword::Contract)) {
            return LabelDefinition { label: Label::Contract(Trivalent::NotApplicable), span };
        }
        self.advance();
        let trivalent = self.parse_parenthesized_string();
        self.skip_comma();
        LabelDefinition { label: Label::Contract(trivalent), span }
    }

    /// Parse `(value)` or `()` → Trivalent::Some(string) or Trivalent::None
    /// Accepts identifiers and keywords as values.
    fn parse_parenthesized_string(&mut self) -> Trivalent<String> {
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
            return Trivalent::None;
        }

        let value = self.expect_label_value();
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        Trivalent::Some(value)
    }

    /// Accept any identifier, keyword, or literal token and return it as a string.
    fn expect_label_value(&mut self) -> String {
        let token = self.advance().clone();
        match token {
            Token::Identifier(name) => name,
            Token::Keyword(kw) => format!("{:?}", kw).to_lowercase(),
            Token::Literal(Literal::Boolean(b)) => b.to_string(),
            Token::Literal(Literal::Integer(n)) => n.to_string(),
            Token::Literal(Literal::String(StringLiteral::Plain(s))) => s,
            other => {
                let span = self.current_span();
                panic!("{}:{}: Expected label value, got {:?}", span.line, span.column, other);
            }
        }
    }

    fn skip_comma(&mut self) {
        if self.check(&Token::Symbol(Symbol::Comma)) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SpannedToken;

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned, false)
    }

    fn label_tokens(inner: Vec<Token>) -> Vec<Token> {
        let mut tokens = vec![
            Token::Keyword(Keyword::Labels),
            Token::Symbol(Symbol::Bracket(Bound::Opening)),
        ];
        tokens.extend(inner);
        tokens.push(Token::Symbol(Symbol::Bracket(Bound::Closing)));
        tokens.push(Token::EndOfFile);
        tokens
    }

    #[test]
    fn test_all_not_applicable() {
        let mut parser = parser_from_tokens(label_tokens(vec![]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels.len(), 7);
        for ld in &labels {
            match &ld.label {
                Label::Return(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Mutability(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Visibility(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Scope(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Generics(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Implementation(t) => assert_eq!(*t, Trivalent::NotApplicable),
                Label::Contract(t) => assert_eq!(*t, Trivalent::NotApplicable),
            }
        }
    }

    #[test]
    fn test_return_empty() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Return(Trivalent::None));
    }

    #[test]
    fn test_return_with_type() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Return(Trivalent::Some("Integer".to_string())));
    }

    #[test]
    fn test_mutable() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Mutable),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[1].label, Label::Mutability(Trivalent::Some(true)));
    }

    #[test]
    fn test_immutable() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Immutable),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[1].label, Label::Mutability(Trivalent::Some(false)));
    }

    #[test]
    fn test_visibility() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Visibility),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("public".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[2].label, Label::Visibility(Trivalent::Some("public".to_string())));
    }

    #[test]
    fn test_scope() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::Project),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[3].label, Label::Scope(Trivalent::Some("project".to_string())));
    }

    #[test]
    fn test_generics_empty() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[4].label, Label::Generics(Trivalent::None));
    }

    #[test]
    fn test_generics_with_types() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Generics),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("T".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Identifier("U".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[4].label, Label::Generics(Trivalent::Some(vec![
            "T".to_string(), "U".to_string(),
        ])));
    }

    #[test]
    fn test_implementation() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("full".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[5].label, Label::Implementation(Trivalent::Some("full".to_string())));
    }

    #[test]
    fn test_contract() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("filled".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[6].label, Label::Contract(Trivalent::Some("filled".to_string())));
    }

    #[test]
    fn test_full_hello_sol_labels() {
        // return(), visibility(public), scope(project), generics(), immutable,
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Visibility),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("public".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::Project),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Generics),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Immutable),
            Token::Symbol(Symbol::Comma),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Return(Trivalent::None));
        assert_eq!(labels[1].label, Label::Mutability(Trivalent::Some(false)));
        assert_eq!(labels[2].label, Label::Visibility(Trivalent::Some("public".to_string())));
        assert_eq!(labels[3].label, Label::Scope(Trivalent::Some("project".to_string())));
        assert_eq!(labels[4].label, Label::Generics(Trivalent::None));
        assert_eq!(labels[5].label, Label::Implementation(Trivalent::NotApplicable));
        assert_eq!(labels[6].label, Label::Contract(Trivalent::NotApplicable));
    }
}
