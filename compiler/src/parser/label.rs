use crate::lexer::keyword::Keyword;
use crate::lexer::literal::{Literal, StringLiteral};
use crate::lexer::symbol::{Symbol, Bound};
use crate::lexer::Token;

#[cfg(test)]
use crate::lexer::Span;
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
}

impl Parser {
    pub fn parse_label_declaration(&mut self) -> Vec<LabelDefinition> {
        let mut labels = Vec::new();

        while self.check(&Token::Symbol(Symbol::Label)) {
            self.advance(); // consume #

            let label = match self.current() {
                Token::Keyword(Keyword::Return) => self.parse_label_return_value(),
                Token::Keyword(Keyword::Generic) => self.parse_label_generics_value(),
                Token::Keyword(Keyword::Mutable) => self.parse_label_mutable_value(),
                Token::Keyword(Keyword::Visibility) => self.parse_label_visibility_value(),
                Token::Keyword(Keyword::Scope) => self.parse_label_scope_value(),
                Token::Keyword(Keyword::Implementation) => self.parse_label_implementation_value(),
                Token::Keyword(Keyword::Contract) => self.parse_label_contract_value(),
                other => {
                    let span = self.current_span();
                    panic!("{}:{}: Unknown label: {:?}", span.line, span.column, other);
                }
            };

            labels.push(LabelDefinition { label });
        }

        // Consume optional terminating semicolon
        if self.check(&Token::Symbol(Symbol::Semicolon)) {
            self.advance();
        }

        labels
    }

    fn parse_label_return_value(&mut self) -> Label {
        self.advance(); // consume 'return'
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        let trivalent = if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            Trivalent::None
        } else {
            // Handle compound types: Entity & Moveable
            let mut value = self.expect_identifier();
            while self.check(&Token::Symbol(Symbol::Ampersand)) {
                self.advance(); // consume '&'
                let next = self.expect_identifier();
                value = format!("{} & {}", value, next);
            }
            Trivalent::Some(value)
        };

        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        Label::Return(trivalent)
    }

    fn parse_label_mutable_value(&mut self) -> Label {
        self.advance(); // consume 'mutable'
        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Opening)));

        let trivalent = if self.check(&Token::Symbol(Symbol::Parentheses(Bound::Closing))) {
            Trivalent::None
        } else {
            let value = self.expect_label_value();
            Trivalent::Some(value == "true")
        };

        self.expect_token(&Token::Symbol(Symbol::Parentheses(Bound::Closing)));
        Label::Mutability(trivalent)
    }

    fn parse_label_visibility_value(&mut self) -> Label {
        self.advance(); // consume 'visibility'
        let trivalent = self.parse_parenthesized_string();
        Label::Visibility(trivalent)
    }

    fn parse_label_scope_value(&mut self) -> Label {
        self.advance(); // consume 'scope'
        let trivalent = self.parse_parenthesized_string();
        Label::Scope(trivalent)
    }

    fn parse_label_generics_value(&mut self) -> Label {
        let open_angle = Token::Symbol(Symbol::Generics(Bound::Opening));
        let close_angle = Token::Symbol(Symbol::Generics(Bound::Closing));

        self.advance(); // consume 'generic'
        self.expect_token(&open_angle);

        let trivalent = if self.check(&close_angle) {
            Trivalent::None
        } else {
            let mut generics = vec![self.expect_identifier()];
            while self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
                if !self.check(&close_angle) {
                    generics.push(self.expect_identifier());
                }
            }
            Trivalent::Some(generics)
        };

        self.expect_token(&close_angle);
        Label::Generics(trivalent)
    }

    fn parse_label_implementation_value(&mut self) -> Label {
        self.advance(); // consume 'implementation'
        let trivalent = self.parse_parenthesized_string();
        Label::Implementation(trivalent)
    }

    fn parse_label_contract_value(&mut self) -> Label {
        self.advance(); // consume 'contract'
        let trivalent = self.parse_parenthesized_string();
        Label::Contract(trivalent)
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


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SpannedToken;

    fn open_angle() -> Token {
        Token::Symbol(Symbol::Generics(Bound::Opening))
    }

    fn close_angle() -> Token {
        Token::Symbol(Symbol::Generics(Bound::Closing))
    }

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned)
    }

    fn label_tokens(inner: Vec<Token>) -> Vec<Token> {
        let mut tokens = vec![];
        tokens.extend(inner);
        tokens.push(Token::EndOfFile);
        tokens
    }

    #[test]
    fn test_no_labels() {
        let mut parser = parser_from_tokens(label_tokens(vec![]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels.len(), 0);
    }

    #[test]
    fn test_return_empty() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Return(Trivalent::None));
    }

    #[test]
    fn test_return_with_type() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Return(Trivalent::Some("Integer".to_string())));
    }

    #[test]
    fn test_mutable_true() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Mutable),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::Boolean(true)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Mutability(Trivalent::Some(true)));
    }

    #[test]
    fn test_mutable_false() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Mutable),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::Boolean(false)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Mutability(Trivalent::Some(false)));
    }

    #[test]
    fn test_visibility() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Visibility),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("public".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Visibility(Trivalent::Some("public".to_string())));
    }

    #[test]
    fn test_scope() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::Project),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Scope(Trivalent::Some("project".to_string())));
    }

    #[test]
    fn test_generics_empty() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Generic),
            open_angle(),
            close_angle(),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Generics(Trivalent::None));
    }

    #[test]
    fn test_generics_with_types() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Generic),
            open_angle(),
            Token::Identifier("T".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Identifier("U".to_string()),
            close_angle(),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Generics(Trivalent::Some(vec![
            "T".to_string(), "U".to_string(),
        ])));
    }

    #[test]
    fn test_implementation() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("full".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Implementation(Trivalent::Some("full".to_string())));
    }

    #[test]
    fn test_contract() {
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("filled".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels[0].label, Label::Contract(Trivalent::Some("filled".to_string())));
    }

    #[test]
    fn test_full_hello_sol_labels() {
        // #return() #generic⟨⟩ #mutable(false) #visibility(public) #scope(project) #implementation(full);
        let mut parser = parser_from_tokens(label_tokens(vec![
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Return),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Generic),
            open_angle(),
            close_angle(),
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Mutable),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Literal(Literal::Boolean(false)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Visibility),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("public".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::Project),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Label),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Keyword(Keyword::Full),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Semicolon),
        ]));
        let labels = parser.parse_label_declaration();
        assert_eq!(labels.len(), 6);
        assert_eq!(labels[0].label, Label::Return(Trivalent::None));
        assert_eq!(labels[1].label, Label::Generics(Trivalent::None));
        assert_eq!(labels[2].label, Label::Mutability(Trivalent::Some(false)));
        assert_eq!(labels[3].label, Label::Visibility(Trivalent::Some("public".to_string())));
        assert_eq!(labels[4].label, Label::Scope(Trivalent::Some("project".to_string())));
        assert_eq!(labels[5].label, Label::Implementation(Trivalent::Some("full".to_string())));
    }
}
