use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Contract {
    None,
    Filled,
}

impl Parser {
    pub fn parse_label_contract(&mut self) -> Option<Contract> {
        let contract = match self.current().clone() {
            Token::Keyword(Keyword::None) => Some(Contract::None),
            Token::Keyword(Keyword::Filled) => Some(Contract::Filled),
            _ => None,
        };

        if contract.is_some() {
            self.advance();
            self.expect_token(&Token::Keyword(Keyword::Contract));
            self.expect_token(&Token::Symbol(Symbol::Comma));
        }


        contract
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
    fn test_no_contract() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::None),
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_contract(), Some(Contract::None));
    }

    #[test]
    fn test_filled_contract() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Filled),
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_contract(), Some(Contract::Filled));
    }

    #[test]
    fn test_not_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Local),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_contract(), None);
    }

    #[test]
    fn test_not_present_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::EndOfFile,
        ]);
        parser.parse_label_contract();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Full));
    }

    #[test]
    fn test_identifier_is_not_contract() {
        let mut parser = parser_from_tokens(vec![
            Token::Identifier("filled".to_string()),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_contract(), None);
    }
}
