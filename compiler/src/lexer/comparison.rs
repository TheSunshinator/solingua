#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonOperator {
    pub checks_equality: bool,
    pub orientation: Option<ComparisonOrientation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOrientation {
    GreaterThan,
    LessThan,
}

impl ComparisonOperator {
    pub fn from(character: char, next_character: Option<char>) -> Option<Self> {
        let orientation = match character {
            '<' => Some(ComparisonOrientation::LessThan),
            '>' => Some(ComparisonOrientation::GreaterThan),
            '=' => None,
            _ => return None,
        };

        let checks_equality = character == '=' || next_character == Some('=');

        Some(ComparisonOperator {
            checks_equality,
            orientation,
        })
    }

    pub fn declaration_size(&self) -> usize {
        if self.checks_equality && self.orientation.is_some() {
            2 // >= or <=
        } else {
            1 // >, <, or =
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greater_than() {
        let result = ComparisonOperator::from('>', Some(' '));
        assert_eq!(result, Some(ComparisonOperator {
            checks_equality: false,
            orientation: Some(ComparisonOrientation::GreaterThan),
        }));
        assert_eq!(result.unwrap().declaration_size(), 1);
    }

    #[test]
    fn test_greater_than_or_equal() {
        let result = ComparisonOperator::from('>', Some('='));
        assert_eq!(result, Some(ComparisonOperator {
            checks_equality: true,
            orientation: Some(ComparisonOrientation::GreaterThan),
        }));
        assert_eq!(result.unwrap().declaration_size(), 2);
    }

    #[test]
    fn test_less_than() {
        let result = ComparisonOperator::from('<', Some(' '));
        assert_eq!(result, Some(ComparisonOperator {
            checks_equality: false,
            orientation: Some(ComparisonOrientation::LessThan),
        }));
        assert_eq!(result.unwrap().declaration_size(), 1);
    }

    #[test]
    fn test_less_than_or_equal() {
        let result = ComparisonOperator::from('<', Some('='));
        assert_eq!(result, Some(ComparisonOperator {
            checks_equality: true,
            orientation: Some(ComparisonOrientation::LessThan),
        }));
        assert_eq!(result.unwrap().declaration_size(), 2);
    }

    #[test]
    fn test_equal() {
        let result = ComparisonOperator::from('=', Some(' '));
        assert_eq!(result, Some(ComparisonOperator {
            checks_equality: true,
            orientation: None,
        }));
        assert_eq!(result.unwrap().declaration_size(), 1);
    }

    #[test]
    fn test_not_a_comparison() {
        assert_eq!(ComparisonOperator::from('a', None), None);
        assert_eq!(ComparisonOperator::from('+', None), None);
        assert_eq!(ComparisonOperator::from('.', None), None);
    }
}
