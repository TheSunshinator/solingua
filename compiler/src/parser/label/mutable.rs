use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

impl Parser {
    pub fn parse_label_mutable(&mut self) -> Option<Mutability> {
        let contract = match self.current().clone() {
            Token::Keyword(Keyword::Mutable) => Some(Mutability::Mutable),
            Token::Keyword(Keyword::Immutable) => Some(Mutability::Immutable),
            _ => None,
        };

        if contract.is_some() {
            self.advance();
            self.expect_token(&Token::Symbol(Symbol::Comma));
        }

        contract
    }
}
