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

#[derive(Debug, Clone, Copy)]
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
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            if self.position >= self.input.len() {
                tokens.push(SpannedToken { token: Token::EndOfFile, span: self.current_span() });
                break;
            }

            let span = self.current_span();
            let token = self.next_token();
            tokens.push(SpannedToken { token, span });
        }

        tokens
    }

    fn next_token(&mut self) -> Token {
        let character = self.input[self.position];

        if let Some(symbol) = Symbol::from(character) {
            self.advance_by(1);
            return Token::Symbol(symbol);
        }

        if let Some(operator) = ComparisonOperator::from(character, self.peek(1)) {
            self.advance_by(operator.declaration_size());
            return Token::ComparisonOperator(operator);
        }

        if let Some(literal) = Literal::from(|index| self.peek(index)) {
            self.advance_by(literal.declaration_size());
            return Token::Literal(literal);
        }

        if let Some(keyword) = Keyword::from(|index| self.peek(index)) {
            self.advance_by(keyword.declaration_size());
            return Token::Keyword(keyword);
        }

        self.read_identifier()
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            if self.input[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn advance_by(&mut self, count: usize) {
        self.column += count;
        self.position += count;
    }

    fn current_span(&self) -> Span {
        Span { line: self.line, column: self.column }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len()
            && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_')
        {
            self.position += 1;
            self.column += 1;
        }
        let name: String = self.input[start..self.position].iter().collect();
        Token::Identifier(name)
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
    fn test_mixed_expression() {
        assert_eq!(tokenize("x + 1"), vec![
            Token::Identifier("x".to_string()),
            Token::Symbol(Symbol::Arithmetic(Arithmetic::Plus)),
            Token::Literal(Literal::Integer(1)),
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
}
