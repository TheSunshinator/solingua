#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    String(StringLiteral),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringLiteral {
    Plain(String),
    Template(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Text(String),
    Interpolation(String),
}

impl Literal {
    pub fn from(get_char: impl Fn(usize) -> Option<char>) -> Option<Self> {
        let first_char = get_char(0)?;

        match first_char {
            '"' => read_string(&get_char),
            't' if matches_word(&get_char, "true") => Some(Literal::Boolean(true)),
            'f' if matches_word(&get_char, "false") => Some(Literal::Boolean(false)),
            _ if first_char.is_ascii_digit() => read_integer(&get_char),
            _ => None,
        }
    }

    pub fn declaration_size(&self) -> usize {
        match self {
            Literal::Boolean(true) => 4,
            Literal::Boolean(false) => 5,
            Literal::Integer(x) => {
                if *x == 0 {
                    1
                } else {
                    ((*x as f64).abs().log10().floor() as usize) + 1
                }
            }
            Literal::String(string_literal) => string_literal.declaration_size(),
        }
    }
}

impl StringLiteral {
    fn declaration_size(&self) -> usize {
        match self {
            StringLiteral::Plain(s) => s.len() + 2, // +2 for the quotes
            StringLiteral::Template(parts) => {
                let content_size: usize = parts.iter().map(|part| match part {
                    StringPart::Text(s) => s.len(),
                    StringPart::Interpolation(s) => s.len() + 3, // \( + content + )
                }).sum();
                content_size + 2 // +2 for the quotes
            }
        }
    }
}

fn matches_word(get_char: &impl Fn(usize) -> Option<char>, word: &str) -> bool {
    for (index, expected) in word.chars().enumerate() {
        if get_char(index) != Some(expected) {
            return false;
        }
    }
    // Ensure the word isn't part of a longer identifier
    match get_char(word.len()) {
        Some(c) if c.is_alphanumeric() || c == '_' => false,
        _ => true,
    }
}

fn read_integer(get_char: &impl Fn(usize) -> Option<char>) -> Option<Literal> {
    let mut number: i64 = 0;
    let mut index = 0;

    loop {
        match get_char(index) {
            Some(c) if c.is_ascii_digit() => {
                number = number * 10 + (c as i64 - '0' as i64);
                index += 1;
            }
            _ => break,
        }
    }

    if index > 0 {
        Some(Literal::Integer(number))
    } else {
        None
    }
}

fn read_string(get_char: &impl Fn(usize) -> Option<char>) -> Option<Literal> {
    // Skip opening quote (index 0 is '"')
    let mut index = 1;
    let mut parts: Vec<StringPart> = Vec::new();
    let mut current_text = String::new();
    let mut is_template = false;

    loop {
        match get_char(index) {
            None => return None, // unterminated string
            Some('"') => {
                // End of string
                break;
            }
            Some('\\') if get_char(index + 1) == Some('(') => {
                // String interpolation: \(...)
                is_template = true;
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }
                index += 2; // skip \(

                // Read until matching closing )
                let mut paren_depth = 1;
                let mut interpolation = String::new();
                loop {
                    match get_char(index) {
                        None => return None,
                        Some('(') => {
                            paren_depth += 1;
                            interpolation.push('(');
                            index += 1;
                        }
                        Some(')') => {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                index += 1; // skip closing )
                                break;
                            }
                            interpolation.push(')');
                            index += 1;
                        }
                        Some(c) => {
                            interpolation.push(c);
                            index += 1;
                        }
                    }
                }
                parts.push(StringPart::Interpolation(interpolation));
            }
            Some(c) => {
                current_text.push(c);
                index += 1;
            }
        }
    }

    // index now points at the closing quote
    if is_template {
        if !current_text.is_empty() {
            parts.push(StringPart::Text(current_text));
        }
        Some(Literal::String(StringLiteral::Template(parts)))
    } else {
        Some(Literal::String(StringLiteral::Plain(current_text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_literal() {
        let input: Vec<char> = "42".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, Some(Literal::Integer(42)));
    }

    #[test]
    fn test_integer_zero() {
        let input: Vec<char> = "0".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, Some(Literal::Integer(0)));
        assert_eq!(result.unwrap().declaration_size(), 1);
    }

    #[test]
    fn test_integer_declaration_size() {
        let lit = Literal::Integer(123);
        assert_eq!(lit.declaration_size(), 3);

        let lit = Literal::Integer(7);
        assert_eq!(lit.declaration_size(), 1);

        let lit = Literal::Integer(1000);
        assert_eq!(lit.declaration_size(), 4);
    }

    #[test]
    fn test_boolean_true() {
        let input: Vec<char> = "true".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, Some(Literal::Boolean(true)));
        assert_eq!(result.unwrap().declaration_size(), 4);
    }

    #[test]
    fn test_boolean_false() {
        let input: Vec<char> = "false".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, Some(Literal::Boolean(false)));
        assert_eq!(result.unwrap().declaration_size(), 5);
    }

    #[test]
    fn test_boolean_not_prefix_of_identifier() {
        let input: Vec<char> = "trueValue".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, None);
    }

    #[test]
    fn test_plain_string() {
        let input: Vec<char> = r#""hello world""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(
            result,
            Some(Literal::String(StringLiteral::Plain("hello world".to_string())))
        );
    }

    #[test]
    fn test_plain_string_declaration_size() {
        let input: Vec<char> = r#""abc""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied()).unwrap();
        assert_eq!(result.declaration_size(), 5); // "abc" = 3 chars + 2 quotes
    }

    #[test]
    fn test_empty_string() {
        let input: Vec<char> = r#""""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(
            result,
            Some(Literal::String(StringLiteral::Plain("".to_string())))
        );
        assert_eq!(result.unwrap().declaration_size(), 2);
    }

    #[test]
    fn test_string_template_single_interpolation() {
        let input: Vec<char> = r#""Hello \(name)!""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(
            result,
            Some(Literal::String(StringLiteral::Template(vec![
                StringPart::Text("Hello ".to_string()),
                StringPart::Interpolation("name".to_string()),
                StringPart::Text("!".to_string()),
            ])))
        );
    }

    #[test]
    fn test_string_template_multiple_interpolations() {
        let input: Vec<char> = r#""\(a) and \(b)""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(
            result,
            Some(Literal::String(StringLiteral::Template(vec![
                StringPart::Interpolation("a".to_string()),
                StringPart::Text(" and ".to_string()),
                StringPart::Interpolation("b".to_string()),
            ])))
        );
    }

    #[test]
    fn test_string_template_with_nested_parens() {
        let input: Vec<char> = r#""result: \(foo(1))""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(
            result,
            Some(Literal::String(StringLiteral::Template(vec![
                StringPart::Text("result: ".to_string()),
                StringPart::Interpolation("foo(1)".to_string()),
            ])))
        );
    }

    #[test]
    fn test_string_template_declaration_size() {
        // "Hello \(name)!" = 16 chars total including quotes
        let input: Vec<char> = r#""Hello \(name)!""#.chars().collect();
        let result = Literal::from(|i| input.get(i).copied()).unwrap();
        assert_eq!(result.declaration_size(), 16);
    }

    #[test]
    fn test_not_a_literal() {
        let input: Vec<char> = "hello".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, None);
    }

    #[test]
    fn test_not_a_literal_symbol() {
        let input: Vec<char> = "+".chars().collect();
        let result = Literal::from(|i| input.get(i).copied());
        assert_eq!(result, None);
    }
}
