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
            r"※※[\s\S]*?※※",            // multiline comments (※※ ... ※※)
            r"|※[^\n]*",                    // single-line comments (※ to end of line)
            r#"|"(?:[^"\\]|\\.)*""#,    // strings (with escaped chars)
            r"|[0-9A-F]+₁₆",              // hex integers (FF₁₆)
            r"|[01]+₂",                    // binary integers (1010₂)
            r"|\d+\.\d+",                  // float literals (3.14)
            r"|[a-zA-Z_][a-zA-Z0-9_]*",   // words (identifiers/keywords)
            r"|\d+",                       // integers
            r"|[><=≥≤≠(){}\[\],+\-×/;#|&𝑓⟨⟩∧∨¬→ℕ]",  // single-char symbols and operators
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

            // Skip comments (single-line ※ and multiline ※※...※※)
            if chunk.starts_with('※') {
                continue;
            }

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
        // Hex literal: FF₁₆
        if let Some(hex_digits) = chunk.strip_suffix("₁₆") {
            if !hex_digits.is_empty() {
                let value = i64::from_str_radix(hex_digits, 16).unwrap_or_else(|_| {
                    panic!("Invalid hex literal: {}", chunk);
                });
                return Token::Literal(Literal::Integer(value));
            }
        }

        // Binary literal: 1010₂
        if let Some(bin_digits) = chunk.strip_suffix('₂') {
            if !bin_digits.is_empty() {
                let value = i64::from_str_radix(bin_digits, 2).unwrap_or_else(|_| {
                    panic!("Invalid binary literal: {}", chunk);
                });
                return Token::Literal(Literal::Integer(value));
            }
        }

        // Float literal: 3.14
        if chunk.contains('.') {
            if let Ok(value) = chunk.parse::<f64>() {
                return Token::Literal(Literal::Float(value));
            }
        }

        let chars: Vec<char> = chunk.chars().collect();

        if chars.len() == 1 {
            if let Some(operator) = ComparisonOperator::from(chars[0]) {
                return Token::ComparisonOperator(operator);
            }

            if let Some(symbol) = Symbol::from(chars[0]) {
                return Token::Symbol(symbol);
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
        assert_eq!(tokenize("+ - × /"), vec![
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Minus)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Times)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Divided)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("> ≥ < ≤ = ≠");
        assert_eq!(tokens[0], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            negated: true,
            checks_smaller_than: true,
        }));
        assert_eq!(tokens[1], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: false,
            negated: true,
            checks_smaller_than: true,
        }));
        assert_eq!(tokens[2], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: false,
            negated: false,
            checks_smaller_than: true,
        }));
        assert_eq!(tokens[3], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            negated: false,
            checks_smaller_than: true,
        }));
        assert_eq!(tokens[4], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            negated: false,
            checks_smaller_than: false,
        }));
        assert_eq!(tokens[5], Token::ComparisonOperator(ComparisonOperator {
            checks_equality: true,
            negated: true,
            checks_smaller_than: false,
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
        let tokens = tokenize("let value greeting");
        assert_eq!(tokens[0], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Value));
        assert_eq!(tokens[2], Token::Identifier("greeting".to_string()));
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
    fn test_full_value_declaration() {
        let tokens = tokenize("let value counter { #return(Integer) #mutable(false) #scope(local) #implementation(full); initially 0 }");
        assert_eq!(tokens[0], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Value));
        assert_eq!(tokens[2], Token::Identifier("counter".to_string()));
        assert_eq!(tokens[3], Token::Symbol(Symbol::Brace(Bound::Opening)));
        assert_eq!(tokens[4], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[5], Token::Keyword(Keyword::Return));
        assert_eq!(tokens[6], Token::Symbol(Symbol::Parentheses(Bound::Opening)));
        assert_eq!(tokens[7], Token::Identifier("Integer".to_string()));
        assert_eq!(tokens[8], Token::Symbol(Symbol::Parentheses(Bound::Closing)));
    }

    #[test]
    fn test_string_template() {
        let tokens = tokenize(r#""Hello \(name)!""#);
        assert_eq!(tokens.len(), 2); // template literal + EOF
        assert!(matches!(&tokens[0], Token::Literal(Literal::String(StringLiteral::Template(_)))));
    }

    #[test]
    fn test_comparison_no_spaces() {
        assert_eq!(tokenize("a≥b"), vec![
            Token::Identifier("a".to_string()),
            Token::ComparisonOperator(ComparisonOperator {
                checks_equality: false,
                negated: true,
                checks_smaller_than: true,
            }),
            Token::Identifier("b".to_string()),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_logical_operators() {
        use super::super::symbol::Logical;
        assert_eq!(tokenize("¬x ∧ y ∨ z"), vec![
            Token::Symbol(Symbol::Logical(Logical::Not)),
            Token::Identifier("x".to_string()),
            Token::Symbol(Symbol::Logical(Logical::And)),
            Token::Identifier("y".to_string()),
            Token::Symbol(Symbol::Logical(Logical::Or)),
            Token::Identifier("z".to_string()),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_label_prefix() {
        let tokens = tokenize("#return(Integer)");
        assert_eq!(tokens[0], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Return));
        assert_eq!(tokens[2], Token::Symbol(Symbol::Parentheses(Bound::Opening)));
        assert_eq!(tokens[3], Token::Identifier("Integer".to_string()));
        assert_eq!(tokens[4], Token::Symbol(Symbol::Parentheses(Bound::Closing)));
    }

    #[test]
    fn test_function_symbol() {
        let tokens = tokenize("let 𝑓 main");
        assert_eq!(tokens[0], Token::Keyword(Keyword::Let));
        assert_eq!(tokens[1], Token::Symbol(Symbol::Function));
        assert_eq!(tokens[2], Token::Identifier("main".to_string()));
    }

    #[test]
    fn test_comment_ignored() {
        assert_eq!(tokenize("※ this is a comment"), vec![
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_comment_after_code() {
        assert_eq!(tokenize("42 ※ a comment"), vec![
            Token::Literal(Literal::Integer(42)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_comment_does_not_consume_next_line() {
        assert_eq!(tokenize("42 ※ comment\n7"), vec![
            Token::Literal(Literal::Integer(42)),
            Token::Literal(Literal::Integer(7)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_multiline_comment() {
        assert_eq!(tokenize("※※ this is\na multiline\ncomment ※※"), vec![
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_multiline_comment_between_code() {
        assert_eq!(tokenize("1 ※※ comment ※※ 2"), vec![
            Token::Literal(Literal::Integer(1)),
            Token::Literal(Literal::Integer(2)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_multiline_comment_spanning_lines() {
        assert_eq!(tokenize("1\n※※\nmultiline\n※※\n2"), vec![
            Token::Literal(Literal::Integer(1)),
            Token::Literal(Literal::Integer(2)),
            Token::EndOfFile,
        ]);
    }

    // === Numeric literal tests ===

    #[test]
    fn test_binary_literal() {
        assert_eq!(tokenize("1010₂"), vec![
            Token::Literal(Literal::Integer(10)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_binary_literal_single_bit() {
        assert_eq!(tokenize("1₂"), vec![
            Token::Literal(Literal::Integer(1)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_binary_literal_byte() {
        assert_eq!(tokenize("11111111₂"), vec![
            Token::Literal(Literal::Integer(255)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(tokenize("FF₁₆"), vec![
            Token::Literal(Literal::Integer(255)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_hex_literal_mixed_digits() {
        assert_eq!(tokenize("1A3₁₆"), vec![
            Token::Literal(Literal::Integer(419)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_hex_literal_zero() {
        assert_eq!(tokenize("0₁₆"), vec![
            Token::Literal(Literal::Integer(0)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_float_literal() {
        assert_eq!(tokenize("3.14"), vec![
            Token::Literal(Literal::Float(3.14)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_float_literal_zero() {
        assert_eq!(tokenize("0.5"), vec![
            Token::Literal(Literal::Float(0.5)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_float_literal_whole() {
        assert_eq!(tokenize("100.0"), vec![
            Token::Literal(Literal::Float(100.0)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_natural_symbol() {
        let tokens = tokenize("42#ℕLong");
        assert_eq!(tokens[0], Token::Literal(Literal::Integer(42)));
        assert_eq!(tokens[1], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[2], Token::Symbol(Symbol::Natural));
        assert_eq!(tokens[3], Token::Identifier("Long".to_string()));
    }

    #[test]
    fn test_unsigned_integer_shorthand() {
        let tokens = tokenize("1234#ℕ");
        assert_eq!(tokens[0], Token::Literal(Literal::Integer(1234)));
        assert_eq!(tokens[1], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[2], Token::Symbol(Symbol::Natural));
        assert_eq!(tokens[3], Token::EndOfFile);
    }

    #[test]
    fn test_hex_with_type_annotation() {
        let tokens = tokenize("FF₁₆#Byte");
        assert_eq!(tokens[0], Token::Literal(Literal::Integer(255)));
        assert_eq!(tokens[1], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[2], Token::Identifier("Byte".to_string()));
    }

    #[test]
    fn test_binary_with_unsigned_type() {
        let tokens = tokenize("1010₂#ℕByte");
        assert_eq!(tokens[0], Token::Literal(Literal::Integer(10)));
        assert_eq!(tokens[1], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[2], Token::Symbol(Symbol::Natural));
        assert_eq!(tokens[3], Token::Identifier("Byte".to_string()));
    }

    #[test]
    fn test_float_not_confused_with_integer() {
        assert_eq!(tokenize("3.14 + 1"), vec![
            Token::Literal(Literal::Float(3.14)),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)),
            Token::Literal(Literal::Integer(1)),
            Token::EndOfFile,
        ]);
    }

    #[test]
    fn test_generic_angle_brackets() {
        let tokens = tokenize("#generic⟨T⟩");
        assert_eq!(tokens[0], Token::Symbol(Symbol::Label));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Generic));
        assert_eq!(tokens[2], Token::Symbol(Symbol::Generics(Bound::Opening)));
        assert_eq!(tokens[3], Token::Identifier("T".to_string()));
        assert_eq!(tokens[4], Token::Symbol(Symbol::Generics(Bound::Closing)));
    }
}
