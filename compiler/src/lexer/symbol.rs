#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Comma,
    Semicolon,
    Comment,
    Arrow,
    Pipe,
    Ampersand,
    Function,
    Label,
    Arithmetic(Arithmetic),
    Logical(Logical),
    Brace(Bound),
    Bracket(Bound),
    Parentheses(Bound),
    Generics(Bound),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bound {
    Opening,
    Closing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arithmetic {
    Plus,
    Minus,
    Times,
    Divided,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logical {
    And,
    Or,
    Not,
}

impl Symbol {
    pub fn from(character: char) -> Option<Self> {
        match character {
            '{' => Some(Symbol::Brace(Bound::Opening)),
            '}' => Some(Symbol::Brace(Bound::Closing)),
            '[' => Some(Symbol::Bracket(Bound::Opening)),
            ']' => Some(Symbol::Bracket(Bound::Closing)),
            '(' => Some(Symbol::Parentheses(Bound::Opening)),
            ')' => Some(Symbol::Parentheses(Bound::Closing)),
            ',' => Some(Symbol::Comma),
            '+' => Some(Symbol::Arithmetic(Arithmetic::Plus)),
            '-' => Some(Symbol::Arithmetic(Arithmetic::Minus)),
            '×' => Some(Symbol::Arithmetic(Arithmetic::Times)),
            '/' => Some(Symbol::Arithmetic(Arithmetic::Divided)),
            ';' => Some(Symbol::Semicolon),
            '※' => Some(Symbol::Comment),
            '→' => Some(Symbol::Arrow),
            '|' => Some(Symbol::Pipe),
            '&' => Some(Symbol::Ampersand),
            '#' => Some(Symbol::Label),
            '𝑓' => Some(Symbol::Function),
            '⟨' => Some(Symbol::Generics(Bound::Opening)),
            '⟩' => Some(Symbol::Generics(Bound::Closing)),
            '∧' => Some(Symbol::Logical(Logical::And)),
            '∨' => Some(Symbol::Logical(Logical::Or)),
            '¬' => Some(Symbol::Logical(Logical::Not)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braces() {
        assert_eq!(Symbol::from('{'), Some(Symbol::Brace(Bound::Opening)));
        assert_eq!(Symbol::from('}'), Some(Symbol::Brace(Bound::Closing)));
    }

    #[test]
    fn test_brackets() {
        assert_eq!(Symbol::from('['), Some(Symbol::Bracket(Bound::Opening)));
        assert_eq!(Symbol::from(']'), Some(Symbol::Bracket(Bound::Closing)));
    }

    #[test]
    fn test_parentheses() {
        assert_eq!(Symbol::from('('), Some(Symbol::Parentheses(Bound::Opening)));
        assert_eq!(Symbol::from(')'), Some(Symbol::Parentheses(Bound::Closing)));
    }

    #[test]
    fn test_generics() {
        assert_eq!(Symbol::from('⟨'), Some(Symbol::Generics(Bound::Opening)));
        assert_eq!(Symbol::from('⟩'), Some(Symbol::Generics(Bound::Closing)));
    }

    #[test]
    fn test_comma() {
        assert_eq!(Symbol::from(','), Some(Symbol::Comma));
    }

    #[test]
    fn test_semicolon() {
        assert_eq!(Symbol::from(';'), Some(Symbol::Semicolon));
    }

    #[test]
    fn test_comment() {
        assert_eq!(Symbol::from('※'), Some(Symbol::Comment));
    }

    #[test]
    fn test_arrow() {
        assert_eq!(Symbol::from('→'), Some(Symbol::Arrow));
    }

    #[test]
    fn test_pipe_and_ampersand() {
        assert_eq!(Symbol::from('|'), Some(Symbol::Pipe));
        assert_eq!(Symbol::from('&'), Some(Symbol::Ampersand));
    }

    #[test]
    fn test_label() {
        assert_eq!(Symbol::from('#'), Some(Symbol::Label));
    }

    #[test]
    fn test_function() {
        assert_eq!(Symbol::from('𝑓'), Some(Symbol::Function));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(Symbol::from('+'), Some(Symbol::Arithmetic(Arithmetic::Plus)));
        assert_eq!(Symbol::from('-'), Some(Symbol::Arithmetic(Arithmetic::Minus)));
        assert_eq!(Symbol::from('×'), Some(Symbol::Arithmetic(Arithmetic::Times)));
        assert_eq!(Symbol::from('/'), Some(Symbol::Arithmetic(Arithmetic::Divided)));
    }

    #[test]
    fn test_logical() {
        assert_eq!(Symbol::from('∧'), Some(Symbol::Logical(Logical::And)));
        assert_eq!(Symbol::from('∨'), Some(Symbol::Logical(Logical::Or)));
        assert_eq!(Symbol::from('¬'), Some(Symbol::Logical(Logical::Not)));
    }

    #[test]
    fn test_not_a_symbol() {
        assert_eq!(Symbol::from('a'), None);
        assert_eq!(Symbol::from('5'), None);
        assert_eq!(Symbol::from(' '), None);
    }
}
