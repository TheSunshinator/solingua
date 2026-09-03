#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Becomes,
    // declaration blocks
    Labels,
    Body,
    // Visibility label
    Visibility,
    Public,
    Private,
    // What construct
    Value,
    Function,
    Blueprint,
    Singleton,
    Container,
    // type label
    Type,
    None,
    // parameters label
    Parameters,
    // Control flow
    Return,
    If,
    Then,
    Else,
    While,
    // Boolean operators
    And,
    Or,
    Not,
    // implementation label
    Implementation,
    Full,
    Partial,
    // contract label
    Contract,
    Filled,
    // scope label
    Scope,
    Instance,
    Local,
    Project,
    // generics label
    Generics,
    Of,
    Specific,
    Alias,
    // mutability
    Mutable,
    Immutable,
}

impl Keyword {
    pub fn from(get_char: impl Fn(usize) -> Option<char>) -> Option<Self> {
        let word = read_word(&get_char)?;

        match word.as_str() {
            "let" => Some(Keyword::Let),
            "becomes" => Some(Keyword::Becomes),
            "labels" => Some(Keyword::Labels),
            "body" => Some(Keyword::Body),
            "visibility" => Some(Keyword::Visibility),
            "public" => Some(Keyword::Public),
            "private" => Some(Keyword::Private),
            "value" => Some(Keyword::Value),
            "function" => Some(Keyword::Function),
            "blueprint" => Some(Keyword::Blueprint),
            "singleton" => Some(Keyword::Singleton),
            "container" => Some(Keyword::Container),
            "alias" => Some(Keyword::Alias),
            "type" => Some(Keyword::Type),
            "none" => Some(Keyword::None),
            "return" => Some(Keyword::Return),
            "parameters" => Some(Keyword::Parameters),
            "if" => Some(Keyword::If),
            "then" => Some(Keyword::Then),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "and" => Some(Keyword::And),
            "or" => Some(Keyword::Or),
            "not" => Some(Keyword::Not),
            "implementation" => Some(Keyword::Implementation),
            "full" => Some(Keyword::Full),
            "partial" => Some(Keyword::Partial),
            "contract" => Some(Keyword::Contract),
            "filled" => Some(Keyword::Filled),
            "scope" => Some(Keyword::Scope),
            "instance" => Some(Keyword::Instance),
            "local" => Some(Keyword::Local),
            "project" => Some(Keyword::Project),
            "generics" => Some(Keyword::Generics),
            "of" => Some(Keyword::Of),
            "specific" => Some(Keyword::Specific),
            "mutable" => Some(Keyword::Mutable),
            "immutable" => Some(Keyword::Immutable),
            _ => None,
        }
    }

    pub fn declaration_size(&self) -> usize {
        match self {
            Keyword::Let => 3,
            Keyword::Becomes => 7,
            Keyword::Labels => 6,
            Keyword::Body => 4,
            Keyword::Visibility => 10,
            Keyword::Public => 6,
            Keyword::Private => 7,
            Keyword::Value => 5,
            Keyword::Function => 8,
            Keyword::Blueprint => 9,
            Keyword::Singleton => 9,
            Keyword::Container => 9,
            Keyword::Alias => 5,
            Keyword::Type => 4,
            Keyword::None => 4,
            Keyword::Parameters => 10,
            Keyword::Return => 6,
            Keyword::If => 2,
            Keyword::Then => 4,
            Keyword::Else => 4,
            Keyword::While => 5,
            Keyword::And => 3,
            Keyword::Or => 2,
            Keyword::Not => 3,
            Keyword::Implementation => 14,
            Keyword::Full => 4,
            Keyword::Partial => 7,
            Keyword::Contract => 8,
            Keyword::Filled => 6,
            Keyword::Scope => 5,
            Keyword::Instance => 8,
            Keyword::Local => 5,
            Keyword::Project => 7,
            Keyword::Generics => 8,
            Keyword::Of => 2,
            Keyword::Specific => 8,
            Keyword::Mutable => 7,
            Keyword::Immutable => 9,
        }
    }
}

fn read_word(get_char: &impl Fn(usize) -> Option<char>) -> Option<String> {
    let first = get_char(0)?;
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    let mut word = String::new();
    let mut index = 0;

    loop {
        match get_char(index) {
            Some(c) if c.is_alphanumeric() || c == '_' => {
                word.push(c);
                index += 1;
            }
            _ => break,
        }
    }

    Some(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyword_from(input: &str) -> Option<Keyword> {
        let chars: Vec<char> = input.chars().collect();
        Keyword::from(|i| chars.get(i).copied())
    }

    #[test]
    fn test_variable_keywords() {
        assert_eq!(keyword_from("means"), Some(Keyword::Means));
        assert_eq!(keyword_from("becomes"), Some(Keyword::Becomes));
    }

    #[test]
    fn test_control_flow_keywords() {
        assert_eq!(keyword_from("if"), Some(Keyword::If));
        assert_eq!(keyword_from("else"), Some(Keyword::Else));
        assert_eq!(keyword_from("while"), Some(Keyword::While));
        assert_eq!(keyword_from("return"), Some(Keyword::Return));
    }

    #[test]
    fn test_declaration_keywords() {
        assert_eq!(keyword_from("is"), Some(Keyword::Is));
        assert_eq!(keyword_from("value"), Some(Keyword::Value));
        assert_eq!(keyword_from("function"), Some(Keyword::Function));
        assert_eq!(keyword_from("blueprint"), Some(Keyword::Blueprint));
        assert_eq!(keyword_from("singleton"), Some(Keyword::Singleton));
        assert_eq!(keyword_from("parameters"), Some(Keyword::Parameters));
    }

    #[test]
    fn test_type_keywords() {
        assert_eq!(keyword_from("type"), Some(Keyword::Type));
        assert_eq!(keyword_from("returns"), Some(Keyword::Returns));
        assert_eq!(keyword_from("generics"), Some(Keyword::Generics));
        assert_eq!(keyword_from("of"), Some(Keyword::Of));
    }

    #[test]
    fn test_logical_keywords() {
        assert_eq!(keyword_from("and"), Some(Keyword::And));
        assert_eq!(keyword_from("or"), Some(Keyword::Or));
        assert_eq!(keyword_from("not"), Some(Keyword::Not));
    }

    #[test]
    fn test_scope_keywords() {
        assert_eq!(keyword_from("scope"), Some(Keyword::Scope));
        assert_eq!(keyword_from("instance"), Some(Keyword::Instance));
        assert_eq!(keyword_from("local"), Some(Keyword::Local));
        assert_eq!(keyword_from("project"), Some(Keyword::Project));
    }

    #[test]
    fn test_implementation_keywords() {
        assert_eq!(keyword_from("implementation"), Some(Keyword::Implementation));
        assert_eq!(keyword_from("declared"), Some(Keyword::Declared));
        assert_eq!(keyword_from("full"), Some(Keyword::Full));
        assert_eq!(keyword_from("partial"), Some(Keyword::Partial));
        assert_eq!(keyword_from("contracted"), Some(Keyword::Contracted));
    }

    #[test]
    fn test_not_a_keyword() {
        assert_eq!(keyword_from("hello"), None);
        assert_eq!(keyword_from("foo"), None);
        assert_eq!(keyword_from("main"), None);
        assert_eq!(keyword_from("printLine"), None);
    }

    #[test]
    fn test_case_sensitive() {
        assert_eq!(keyword_from("If"), None);
        assert_eq!(keyword_from("TRUE"), None);
        assert_eq!(keyword_from("Return"), None);
    }

    #[test]
    fn test_keyword_with_trailing_chars() {
        // "isFoo" should read the whole word "isFoo" and not match any keyword
        assert_eq!(keyword_from("isFoo"), None);
        // "if " should read "if" (stops at space) and match
        let input: Vec<char> = "if ".chars().collect();
        assert_eq!(Keyword::from(|i| input.get(i).copied()), Some(Keyword::If));
    }

    #[test]
    fn test_declaration_size() {
        assert_eq!(Keyword::If.declaration_size(), 2);
        assert_eq!(Keyword::While.declaration_size(), 5);
        assert_eq!(Keyword::Implementation.declaration_size(), 14);
        assert_eq!(Keyword::Is.declaration_size(), 2);
        assert_eq!(Keyword::Parameters.declaration_size(), 10);
    }

    #[test]
    fn test_not_starting_with_alpha() {
        let input: Vec<char> = "123".chars().collect();
        assert_eq!(Keyword::from(|i| input.get(i).copied()), None);

        let input: Vec<char> = "+foo".chars().collect();
        assert_eq!(Keyword::from(|i| input.get(i).copied()), None);
    }
}
