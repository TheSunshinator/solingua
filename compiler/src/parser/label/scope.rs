use crate::lexer::{Keyword, Symbol, Token};
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Project,
    Local,
    Instance,
    Blueprint,
}

impl Parser {
    pub fn parse_label_scope(&mut self) -> Option<Scope> {
        let scope = match self.current().clone() {
            Token::Keyword(Keyword::Project) => Some(Scope::Project),
            Token::Keyword(Keyword::Local) => Some(Scope::Local),
            Token::Keyword(Keyword::Instance) => Some(Scope::Instance),
            Token::Keyword(Keyword::Blueprint) => Some(Scope::Blueprint),
            _ => None,
        };

        if scope.is_some() {
            self.advance(); // consume the scope keyword
            self.expect_token(&Token::Keyword(Keyword::Scope));
            self.expect_token(&Token::Symbol(Symbol::Comma));
        }

        scope
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
    fn test_project_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Project),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), Some(Scope::Project));
    }

    #[test]
    fn test_local_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), Some(Scope::Local));
    }

    #[test]
    fn test_instance_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Instance),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), Some(Scope::Instance));
    }

    #[test]
    fn test_blueprint_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Blueprint),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), Some(Scope::Blueprint));
    }

    #[test]
    fn test_no_scope_present() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), None);
    }

    #[test]
    fn test_no_scope_does_not_advance() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Full),
            Token::EndOfFile,
        ]);
        parser.parse_label_scope();
        assert_eq!(parser.current().clone(), Token::Keyword(Keyword::Full));
    }

    #[test]
    fn test_identifier_is_not_scope() {
        let mut parser = parser_from_tokens(vec![
            Token::Identifier("foo".to_string()),
            Token::EndOfFile,
        ]);
        assert_eq!(parser.parse_label_scope(), None);
    }
}
