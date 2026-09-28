#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonOperator {
    pub checks_equality: bool,
    pub negated: bool,
    pub checks_smaller_than: bool,
}

impl ComparisonOperator {
    pub fn from(character: char) -> Option<Self> {
        return match character {
            '=' => Some(ComparisonOperator {
                checks_equality: true,
                negated: false,
                checks_smaller_than: false,
            }),
            '≠' => Some(ComparisonOperator {
                checks_equality: true,
                negated: true,
                checks_smaller_than: false,
            }),
            '<' => Some(ComparisonOperator {
                checks_equality: false,
                negated: false,
                checks_smaller_than: true,
            }),
            '>' => Some(ComparisonOperator {
                checks_equality: true,
                negated: true,
                checks_smaller_than: true,
            }),
            '≤' => Some(ComparisonOperator {
                checks_equality: true,
                negated: false,
                checks_smaller_than: true,
            }),
            '≥' => Some(ComparisonOperator {
                checks_equality: false,
                negated: true,
                checks_smaller_than: true,
            }),
            _ => return None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(ComparisonOperator::from('='), Some(ComparisonOperator {
            checks_equality: true,
            negated: false,
            checks_smaller_than: false,
        }));
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(ComparisonOperator::from('≠'), Some(ComparisonOperator {
            checks_equality: true,
            negated: true,
            checks_smaller_than: false,
        }));
    }

    #[test]
    fn test_less_than() {
        assert_eq!(ComparisonOperator::from('<'), Some(ComparisonOperator {
            checks_equality: false,
            negated: false,
            checks_smaller_than: true,
        }));
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(ComparisonOperator::from('>'), Some(ComparisonOperator {
            checks_equality: true,
            negated: true,
            checks_smaller_than: true,
        }));
    }

    #[test]
    fn test_less_than_or_equal() {
        assert_eq!(ComparisonOperator::from('≤'), Some(ComparisonOperator {
            checks_equality: true,
            negated: false,
            checks_smaller_than: true,
        }));
    }

    #[test]
    fn test_greater_than_or_equal() {
        assert_eq!(ComparisonOperator::from('≥'), Some(ComparisonOperator {
            checks_equality: false,
            negated: true,
            checks_smaller_than: true,
        }));
    }

    #[test]
    fn test_not_a_comparison() {
        assert_eq!(ComparisonOperator::from('a'), None);
        assert_eq!(ComparisonOperator::from('+'), None);
        assert_eq!(ComparisonOperator::from('.'), None);
    }
}
