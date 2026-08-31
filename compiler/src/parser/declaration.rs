use crate::lexer::{Keyword, Span};
use crate::parser::Parser;
use super::labels::{Label, SpannedLabel};
use super::label::is::Is;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub declaration_location: Span,
    pub construct: Construct,
    pub name: String,
    pub labels: Vec<SpannedLabel>,
}

#[derive(Debug, Clone)]
pub enum Construct {
    Function,
    Blueprint,
    Container,
    Singleton,
    Value,
    Alias,
}

impl Parser {
    pub fn parse_declaration(&mut self, name: String, is_label: Is, span_start: Span, verbose: bool) -> Declaration {
        let construct = match is_label {
            Is::Value => Construct::Value,
            Is::Function => Construct::Function,
            Is::Blueprint => Construct::Blueprint,
            Is::Container => Construct::Container,
            Is::Alias => Construct::Alias,
            Is::Singleton => Construct::Singleton,
        };

        if verbose {
            println!("Parsing construct: {:?}", construct);
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: type");
        }

        let mut labels = Vec::new();
        labels.push(SpannedLabel {
            label: Label::Is(is_label),
            span: span_start,
        });

        let span = self.current_span();
        let value_type = self.parse_type_label(Keyword::Type);
        labels.push(SpannedLabel {
            label: Label::Type(value_type),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: mutable");
        }

        let span = self.current_span();
        let scope = self.parse_label_mutable();
        labels.push(SpannedLabel {
            label: Label::Mutability(scope),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: return");
        }

        let span = self.current_span();
        let return_type = self.parse_type_label(Keyword::Return);
        labels.push(SpannedLabel {
            label: Label::Return(return_type),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: scope");
        }

        let span = self.current_span();
        let scope = self.parse_label_scope();
        labels.push(SpannedLabel {
            label: Label::Scope(scope),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: implementation");
        }

        let span = self.current_span();
        let implementation = self.parse_label_implementation();
        labels.push(SpannedLabel {
            label: Label::Implementation(implementation),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: generic");
        }

        let span = self.current_span();
        let generic = self.parse_label_generic();
        labels.push(SpannedLabel {
            label: Label::Generic(generic),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: contract");
        }

        let span = self.current_span();
        let contract = self.parse_label_contract();
        labels.push(SpannedLabel {
            label: Label::Contract(contract),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: parameters");
        }

        let span = self.current_span();
        let parameters = self.parse_label_parameters();
        labels.push(SpannedLabel {
            label: Label::Parameters(parameters),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
            println!("Current token: {:?}", self.current().clone());
            println!("Next label: means");
        }

        let span = self.current_span();
        let means = self.parse_label_means();
        labels.push(SpannedLabel {
            label: Label::Means(means),
            span,
        });

        if verbose {
            println!("Last label: {:?}", labels.last());
        }

        Declaration {
            declaration_location: span_start,
            construct,
            name,
            labels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{SpannedToken, Symbol};
    use crate::lexer::symbol::Bound;
    use crate::lexer::literal::{Literal, StringLiteral};
    use super::super::label::r#type::TypeDescription;
    use super::super::label::scope::Scope;
    use super::super::label::implementation::Implementation;
    use super::super::label::contract::Contract;
    use super::super::label::means::Means;
    use crate::ast::Expression;

    fn parser_from_tokens(tokens: Vec<Token>) -> Parser {
        let spanned: Vec<SpannedToken> = tokens
            .into_iter()
            .map(|token| SpannedToken { token, span: Span { line: 1, column: 1 } })
            .collect();
        Parser::new(spanned)
    }

    fn span() -> Span {
        Span { line: 1, column: 1 }
    }

    #[test]
    fn test_all_labels_always_present() {
        // Even with minimal input, all 6 labels are pushed
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(42)),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("x".to_string(), span());
        assert_eq!(decl.labels.len(), 6); // Is, Type, Scope, Implementation, Contract, Means
    }

    #[test]
    fn test_missing_labels_are_none() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(0)),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("x".to_string(), span());

        // Type is None (not present in input)
        assert!(matches!(&decl.labels[1].label, Label::Type(None)));
        // Scope is None
        assert!(matches!(&decl.labels[2].label, Label::Scope(None)));
        // Implementation is None
        assert!(matches!(&decl.labels[3].label, Label::Implementation(None)));
        // Contract is None
        assert!(matches!(&decl.labels[4].label, Label::Contract(None)));
        // Means is Some
        assert!(matches!(&decl.labels[5].label, Label::Means(Some(_))));
    }

    #[test]
    fn test_full_declaration_all_some() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("String".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Local),
            Token::Keyword(Keyword::Scope),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Full),
            Token::Keyword(Keyword::Implementation),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::None),
            Token::Keyword(Keyword::Contract),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::String(StringLiteral::Plain("Hello".to_string()))),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("greeting".to_string(), span());

        assert!(matches!(&decl.labels[0].label, Label::Is(Is::Value)));
        assert!(matches!(&decl.labels[1].label, Label::Type(Some(TypeDescription::Some { .. }))));
        assert!(matches!(&decl.labels[2].label, Label::Scope(Some(Scope::Local))));
        assert!(matches!(&decl.labels[3].label, Label::Implementation(Some(Implementation::Full))));
        assert!(matches!(&decl.labels[4].label, Label::Contract(Some(Contract::None))));
        assert!(matches!(&decl.labels[5].label, Label::Means(Some(Means::Expression(_)))));
    }

    #[test]
    fn test_labels_have_spans() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Type),
            Token::Identifier("Integer".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(5)),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("x".to_string(), span());

        // Every label has a span
        for spanned_label in &decl.labels {
            assert!(spanned_label.span.line >= 1);
            assert!(spanned_label.span.column >= 1);
        }
    }

    #[test]
    fn test_construct_is_value() {
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(0)),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("x".to_string(), span());
        assert!(matches!(decl.construct, Construct::Value));
    }

    #[test]
    fn test_declaration_location() {
        let start = Span { line: 5, column: 3 };
        let mut parser = parser_from_tokens(vec![
            Token::Keyword(Keyword::Means),
            Token::Literal(Literal::Integer(0)),
            Token::EndOfFile,
        ]);
        let decl = parser.parse_value_declaration("y".to_string(), start);
        assert_eq!(decl.declarationLocation.line, 5);
        assert_eq!(decl.declarationLocation.column, 3);
    }
}
