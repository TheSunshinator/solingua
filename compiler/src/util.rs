/// A three-state value for labels:
/// - `NotApplicable` — the label doesn't apply to this construct
/// - `None` — the label applies but has no value (e.g., `return()`, `generics()`)
/// - `Some(T)` — the label has a value
#[derive(Debug, Clone, PartialEq)]
pub enum Trivalent<T> {
    NotApplicable,
    None,
    Some(T),
}

impl<T> Trivalent<T> {
    pub fn is_applicable(&self) -> bool {
        !matches!(self, Trivalent::NotApplicable)
    }

    pub fn is_some(&self) -> bool {
        matches!(self, Trivalent::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Trivalent::None)
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Trivalent::Some(v) => Some(v),
            _ => Option::None,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Trivalent<U> {
        match self {
            Trivalent::Some(v) => Trivalent::Some(f(v)),
            Trivalent::None => Trivalent::None,
            Trivalent::NotApplicable => Trivalent::NotApplicable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_applicable() {
        let t: Trivalent<String> = Trivalent::NotApplicable;
        assert!(!t.is_applicable());
        assert!(!t.is_some());
        assert!(!t.is_none());
        assert_eq!(t.value(), Option::None);
    }

    #[test]
    fn test_none() {
        let t: Trivalent<String> = Trivalent::None;
        assert!(t.is_applicable());
        assert!(!t.is_some());
        assert!(t.is_none());
        assert_eq!(t.value(), Option::None);
    }

    #[test]
    fn test_some() {
        let t = Trivalent::Some("hello".to_string());
        assert!(t.is_applicable());
        assert!(t.is_some());
        assert!(!t.is_none());
        assert_eq!(t.value(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_map() {
        let t = Trivalent::Some(42);
        let mapped = t.map(|v| v.to_string());
        assert_eq!(mapped, Trivalent::Some("42".to_string()));

        let t: Trivalent<i32> = Trivalent::None;
        let mapped = t.map(|v| v.to_string());
        assert_eq!(mapped, Trivalent::None);

        let t: Trivalent<i32> = Trivalent::NotApplicable;
        let mapped = t.map(|v| v.to_string());
        assert_eq!(mapped, Trivalent::NotApplicable);
    }
}
