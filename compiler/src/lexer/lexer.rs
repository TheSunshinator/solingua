use regex::Regex;

use super::comparison::ComparisonOperator;
use super::keyword::Keyword;
use super::literal::Literal;
use super::symbol::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Literal(Literal),
    Symbol(Symbol),
    ComparisonOperator(ComparisonOperator),
    Keyword(Keyword),
    EndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub struct Lexer {
    input: String,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.to_string() }
    }

    pub fn tokenize(&self) -> Vec<SpannedToken> {
        let pattern = Regex::new(concat!(
            r#""(?:[^"\\]|\\.)*""#,   // strings (with escaped chars)
            r"|[a-zA-Z_][a-zA-Z0-9_]*", // words (identifiers/keywords)
            r"|\d+",                     // integers
            r"|>=|<=",                   // two-char comparison operators
            r"|[><=(){}\[\],+\-*/;.]",    // single-char symbols and operators
        )).unwrap();

        let mut tokens = Vec::new();
        let mut line = 1usize;
        let mut last_end = 0usize;

        for mat in pattern.find_iter(&self.input) {
            // Track line/column from whitespace between matches
            for ch in self.input[last_end..mat.start()].chars() {
                if ch == '\n' {
                    line += 1;
                }
            }

            let column = self.column_at(mat.start());
            let span = Span { line, column };
            let chunk = mat.as_str();
            last_end = mat.end();

            let token = Self::classify(chunk);
            tokens.push(SpannedToken { token, span });
        }

        tokens.push(SpannedToken {
            token: Token::EndOfFile,
            span: Span { line, column: self.column_at(self.input.len()) },
        });

        tokens
    }

    fn classify(chunk: &str) -> Token {
        let chars: Vec<char> = chunk.chars().collect();

        if let Some(symbol) = Symbol::from(chars[0]) {
            if chars.len() == 1 {
                return Token::Symbol(symbol);
            }
        }

        if let Some(operator) = ComparisonOperator::from(chars[0], chars.get(1).copied()) {
            if chars[0] == '>' || chars[0] == '<' || chars[0] == '=' {
                return Token::ComparisonOperator(operator);
            }
        }

        if let Some(literal) = Literal::from(|index| chars.get(index).copied()) {
            return Token::Literal(literal);
        }

        if let Some(keyword) = Keyword::from(|index| chars.get(index).copied()) {
            return Token::Keyword(keyword);
        }

        Token::Identifier(chunk.to_string())
    }

    fn column_at(&self, byte_pos: usize) -> usize {
        let before = &self.input[..byte_pos];
        match before.rfind('\n') {
            Some(nl) => byte_pos - nl,
            None => byte_pos + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::literal::StringLiteral;
    use super::super::symbol::{Arithmetic, Bound};
    use super::super::comparison::ComparisonOrientation;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::new(input).tokenize().into_iter().map(|st| st.token).collect()
    }

    fn tokenize_spanned(input: &str) -> Vec<SpannedToken> {
        Lexer::new(input).tokenize()
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(tokenize(""), vec![Token::EndOfFile]);
    }

    #[test]
    fn test_identifier() {
        assert_eq!(tokenize("hello"), vec![
            Token::Identifier("hello".to_string()),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_multiple_identifiers() {
        assert_eq!(tokenize("foo bar"), vec![
            Token::Identifier("foo".to_string()),
            Token::Identifier("bar".to_string()),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_keyword() {
        assert_eq!(tokenize("if"), vec![
            Token::Keyword(Keyword::If),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_keyword_vs_identifier() {
        assert_eq!(tokenize("if ifFoo"), vec![
            Token::Keyword(Keyword::If),
            Token::Identifier("ifFoo".to_string()),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_integer_literal() {
        assert_eq!(tokenize("42"), vec![
            Token::Literal(Literal::Integer(42)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(tokenize(r#""hello""#), vec![
            Token::Literal(Literal::String(StringLiteral::Plain("hello".to_string()))),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_boolean_literal() {
        assert_eq!(tokenize("true false"), vec![
            Token::Literal(Literal::Boolean(true)),
            Token::Literal(Literal::Boolean(false)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_symbols() {
        assert_eq!(tokenize("( ) { } ,"), vec![
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::Symbol(Symbol::Brace(Bound::Opening)),
            Token::Symbol(Symbol::Brace(Bound::Closing)),
            Token::Symbol(Symbol::Comma),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_arithmetic_symbols() {
        assert_eq!(tokenize("+ - * /"), vec![
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Times)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Divided)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("> >= < <= =");
        assert_eq!(tokens[0], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: false,
            orientation: Some(ComparisonOrientation::GreaterThan),
        }));
        assert_eq!(tokens[1], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            orientation: Some(ComparisonOrientation::GreaterThan),
        }));
        assert_eq!(tokens[2], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: false,
            orientation: Some(ComparisonOrientation::LessThan),
        }));
        assert_eq!(tokens[3], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            orientation: Some(ComparisonOrientation::LessThan),
        }));
        assert_eq!(tokens[4], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            orientation: None,
        }));
    }

    #[test]
    fn test_no_spaces_between_symbols_and_identifiers() {
        assert_eq!(tokenize("x+1"), vec![
            Token::Identifier("x".to_string()),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)),
            Token::Literal(Literal::Integer(1)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_function_call_no_spaces() {
        assert_eq!(tokenize("foo(bar,baz)"), vec![
            Token::Identifier("foo".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Opening)),
            Token::Identifier("bar".to_string()),
            Token::Symbol(Symbol::Comma),
            Token::Identifier("baz".to_string()),
            Token::Symbol(Symbol::Parentheses(Bound::Closing)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_declaration() {
        let tokens = tokenize("greeting means \"Hello\"");
        assert_eq!(tokens[0], Token::Identifier("greeting".to_string()));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Means));
        assert_eq!(tokens[2], Token::Literal(Literal::String(StringLiteral::Plain("Hello".to_string()))));
    }

    #[test]
    fn test_span_tracking() {
        let tokens = tokenize_spanned("if x");
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[1].span.line, 1);
        assert_eq!(tokens[1].span.column, 4);
    }

    #[test]
    fn test_span_multiline() {
        let tokens = tokenize_spanned("a\nb");
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.column, 1);
    }

    #[test]
    fn test_full_statement() {
        let tokens = tokenize("counter means 0 is value, type Integer, local scope,");
        assert_eq!(tokens[0], Token::Identifier("counter".to_string()));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Means));
        assert_eq!(tokens[2], Token::Literal(Literal::Integer(0)));
        assert_eq!(tokens[3], Token::Keyword(Keyword::Is));
        assert_eq!(tokens[5], Token::Symbol(Symbol::Comma));
        assert_eq!(tokens[6], Token::Keyword(Keyword::Type));
        assert_eq!(tokens[7], Token::Identifier("Integer".to_string()));
        assert_eq!(tokens[8], Token::Symbol(Symbol::Comma));
        assert_eq!(tokens[9], Token::Keyword(Keyword::Local));
        assert_eq!(tokens[10], Token::Keyword(Keyword::Scope));
        assert_eq!(tokens[11], Token::Symbol(Symbol::Comma));
    }

    #[test]
    fn test_string_template() {
        let tokens = tokenize(r#""Hello \(name)!""#);
        assert_eq!(tokens.len(), 2); // template literal + EOF
        assert!(matches!(&tokens[0], Token::Literal(Literal::String(StringLiteral::Template(_)))));
    }

    #[test]
    fn test_comparison_no_spaces() {
        assert_eq!(tokenize("a>=b"), vec![
            Token::Identifier("a".to_string()),
            Token::ComparisonOperator(ComparisonOperator {
                checks_equality: true,
                orientation: Some(ComparisonOrientation::GreaterThan),
            }),
            Token::Identifier("b".to_string()),
            Token::EndOfFile,
        ]);
    }
}
