#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Comma,
    Semicolon,
    Dot,
    Arithmetic(Arithmetic),
    Brace(Bound),
    Parentheses(Bound),
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

impl Symbol {
    pub fn from(character: char) -> Option<Self> {
        match character {
            '{' => Some(Symbol::Brace(Bound::Opening)),
            '}' => Some(Symbol::Brace(Bound::Closing)),
            '(' => Some(Symbol::Parentheses(Bound::Opening)),
            ')' => Some(Symbol::Parentheses(Bound::Closing)),
            ',' => Some(Symbol::Comma),
            '+' => Some(Symbol::Arithmetic(Arithmetic::Plus)),
            '-' => Some(Symbol::Arithmetic(Arithmetic::Minus)),
            '*' => Some(Symbol::Arithmetic(Arithmetic::Times)),
            '/' => Some(Symbol::Arithmetic(Arithmetic::Divided)),
            ';' => Some(Symbol::Semicolon),
            '.' => Some(Symbol::Dot),
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
    fn test_parentheses() {
        assert_eq!(Symbol::from('('), Some(Symbol::Parentheses(Bound::Opening)));
        assert_eq!(Symbol::from(')'), Some(Symbol::Parentheses(Bound::Closing)));
    }

    #[test]
    fn test_comma() {
        assert_eq!(Symbol::from(','), Some(Symbol::Comma));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(Symbol::from('+'), Some(Symbol::Arithmetic(Arithmetic::Plus)));
        assert_eq!(Symbol::from('-'), Some(Symbol::Arithmetic(Arithmetic::Minus)));
        assert_eq!(Symbol::from('*'), Some(Symbol::Arithmetic(Arithmetic::Times)));
        assert_eq!(Symbol::from('/'), Some(Symbol::Arithmetic(Arithmetic::Divided)));
    }

    #[test]
    fn test_not_a_symbol() {
        assert_eq!(Symbol::from('a'), None);
        assert_eq!(Symbol::from('5'), None);
        assert_eq!(Symbol::from(' '), None);
        assert_eq!(Symbol::from('.'), None);
    }
}
