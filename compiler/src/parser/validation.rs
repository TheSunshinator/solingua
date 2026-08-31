use crate::ast;
use crate::lexer::Span;
use super::declaration::{Construct, Declaration};
use super::labels::{Label, SpannedLabel};
use super::label::means::Means;
use super::label::r#type::TypeDescription;
use super::label::mutable::Mutability;
use super::label::parameters::Parameter as ParsedParameter;

pub fn validate_and_build_ast(
    declarations: Vec<Declaration>,
) -> Result<ast::Program, Vec<String>> {
    let mut ast_declarations = Vec::new();
    let mut errors = Vec::new();

    for decl in declarations {
        match decl.construct {
            Construct::Value => {
                match build_value_declaration(&decl) {
                    Ok(value) => ast_declarations.push(ast::Declaration::Value(value)),
                    Err(errs) => errors.extend(errs),
                }
            }
            Construct::Function => {
                match build_function_declaration(&decl) {
                    Ok(function) => ast_declarations.push(ast::Declaration::Function(function)),
                    Err(errs) => errors.extend(errs),
                }
            }
            Construct::Blueprint => {
                // TODO: implement blueprint AST construction
                errors.push(format_error(
                    decl.declaration_location,
                    "Blueprint declarations not yet implemented in new parser",
                ));
            }
            Construct::Container => {
                // TODO: implement container AST construction
                errors.push(format_error(
                    decl.declaration_location,
                    "Container declarations not yet implemented in new parser",
                ));
            }
            Construct::Singleton => {
                ast_declarations.push(ast::Declaration::Singleton(decl.name));
            }
            Construct::Alias => {
                errors.push(format_error(
                    decl.declaration_location,
                    "Alias declarations not yet implemented",
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(ast::Program { declarations: ast_declarations })
    } else {
        Err(errors)
    }
}

fn build_value_declaration(decl: &Declaration) -> Result<ast::ValueDeclaration, Vec<String>> {
    let mut errors = Vec::new();
    let span = decl.declaration_location;

    let (type_name, type_generics) = match find_label_type(&decl.labels) {
        Some(Some(TypeDescription::Some { identifier, generics })) => {
            (identifier.clone(), generics.clone())
        }
        Some(Some(TypeDescription::None)) => {
            errors.push(format_error(span, "Value cannot have type `none`"));
            (String::new(), Vec::new())
        }
        _ => {
            errors.push(format_error(span, &format!(
                "Value '{}' is missing required `type` label", decl.name
            )));
            (String::new(), Vec::new())
        }
    };

    let assigned_value = match find_label_means(&decl.labels) {
        Some(Some(Means::Expression(expr))) => expr.clone(),
        Some(Some(Means::Block(_))) => {
            errors.push(format_error(span, &format!(
                "Value '{}' cannot have a block body, expected an expression", decl.name
            )));
            ast::Expression::IntegerLiteral(0)
        }
        _ => {
            errors.push(format_error(span, &format!(
                "Value '{}' is missing required `means` label", decl.name
            )));
            ast::Expression::IntegerLiteral(0)
        }
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    let is_mutable = match find_label_mutability(&decl.labels) {
        Some(Some(Mutability::Mutable)) => true,
        _ => false,
    };

    Ok(ast::ValueDeclaration {
        name: decl.name.clone(),
        type_name,
        type_generics,
        assigned_value,
        is_mutable,
    })
}

fn build_function_declaration(decl: &Declaration) -> Result<ast::FunctionDeclaration, Vec<String>> {
    let mut errors = Vec::new();
    let span = decl.declaration_location;

    let return_type = match find_label_return(&decl.labels) {
        Some(Some(TypeDescription::Some { identifier, .. })) => identifier.clone(),
        Some(Some(TypeDescription::None)) => "nothing".to_string(),
        _ => {
            errors.push(format_error(span, &format!(
                "Function '{}' is missing required `returns` label", decl.name
            )));
            "nothing".to_string()
        }
    };

    let parameters = match find_label_parameters(&decl.labels) {
        Some(Some(params)) => params.iter().map(|p| ast::Parameter {
            name: p.name.clone(),
            type_name: match &p.type_description {
                TypeDescription::Some { identifier, .. } => identifier.clone(),
                TypeDescription::None => "none".to_string(),
            },
        }).collect(),
        Some(None) => {
            errors.push(format_error(span, &format!(
                "Function '{}' is missing required `parameters` label", decl.name
            )));
            Vec::new()
        }
        None => Vec::new(),
    };

    let body = match find_label_means(&decl.labels) {
        Some(Some(Means::Block(statements))) => statements.clone(),
        Some(Some(Means::Expression(_))) => {
            errors.push(format_error(span, &format!(
                "Function '{}' must have a block body, not an expression", decl.name
            )));
            Vec::new()
        }
        _ => {
            errors.push(format_error(span, &format!(
                "Function '{}' is missing required `means` label", decl.name
            )));
            Vec::new()
        }
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ast::FunctionDeclaration {
        name: decl.name.clone(),
        parameters,
        return_type,
        body,
    })
}

// --- Label lookup helpers ---

fn find_label_type(labels: &[SpannedLabel]) -> Option<&Option<TypeDescription>> {
    labels.iter().find_map(|sl| match &sl.label {
        Label::Type(t) => Some(t),
        _ => None,
    })
}

fn find_label_return(labels: &[SpannedLabel]) -> Option<&Option<TypeDescription>> {
    labels.iter().find_map(|sl| match &sl.label {
        Label::Return(t) => Some(t),
        _ => None,
    })
}

fn find_label_means(labels: &[SpannedLabel]) -> Option<&Option<Means>> {
    labels.iter().find_map(|sl| match &sl.label {
        Label::Means(m) => Some(m),
        _ => None,
    })
}

fn find_label_parameters(labels: &[SpannedLabel]) -> Option<&Option<Vec<ParsedParameter>>> {
    labels.iter().find_map(|sl| match &sl.label {
        Label::Parameters(p) => Some(p),
        _ => None,
    })
}

fn find_label_mutability(labels: &[SpannedLabel]) -> Option<&Option<Mutability>> {
    labels.iter().find_map(|sl| match &sl.label {
        Label::Mutability(m) => Some(m),
        _ => None,
    })
}

fn format_error(span: Span, message: &str) -> String {
    format!("{}:{}: {}", span.line, span.column, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::label::scope::Scope;
    use super::super::label::implementation::Implementation;

    fn make_decl(construct: Construct, name: &str, labels: Vec<SpannedLabel>) -> Declaration {
        Declaration {
            declaration_location: Span { line: 1, column: 1 },
            construct,
            name: name.to_string(),
            labels,
        }
    }

    fn spanned(label: Label) -> SpannedLabel {
        SpannedLabel { label, span: Span { line: 1, column: 1 } }
    }

    #[test]
    fn test_valid_value_declaration() {
        let decl = make_decl(Construct::Value, "x", vec![
            spanned(Label::Is(Is::Value)),
            spanned(Label::Type(Some(TypeDescription::Some {
                identifier: "Integer".to_string(),
                generics: vec![],
            }))),
            spanned(Label::Scope(Some(Scope::Local))),
            spanned(Label::Means(Some(Means::Expression(ast::Expression::IntegerLiteral(42))))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.declarations.len(), 1);
        assert!(matches!(&program.declarations[0], ast::Declaration::Value(v) if v.name == "x"));
    }

    #[test]
    fn test_value_missing_type() {
        let decl = make_decl(Construct::Value, "x", vec![
            spanned(Label::Is(Is::Value)),
            spanned(Label::Type(None)),
            spanned(Label::Means(Some(Means::Expression(ast::Expression::IntegerLiteral(0))))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("missing required `type` label"));
    }

    #[test]
    fn test_value_missing_means() {
        let decl = make_decl(Construct::Value, "x", vec![
            spanned(Label::Is(Is::Value)),
            spanned(Label::Type(Some(TypeDescription::Some {
                identifier: "Integer".to_string(),
                generics: vec![],
            }))),
            spanned(Label::Means(None)),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("missing required `means` label"));
    }

    #[test]
    fn test_value_cannot_have_block_body() {
        let decl = make_decl(Construct::Value, "x", vec![
            spanned(Label::Is(Is::Value)),
            spanned(Label::Type(Some(TypeDescription::Some {
                identifier: "Integer".to_string(),
                generics: vec![],
            }))),
            spanned(Label::Means(Some(Means::Block(vec![])))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("cannot have a block body"));
    }

    #[test]
    fn test_valid_function_declaration() {
        let decl = make_decl(Construct::Function, "main", vec![
            spanned(Label::Is(Is::Function)),
            spanned(Label::Return(Some(TypeDescription::None))),
            spanned(Label::Scope(Some(Scope::Project))),
            spanned(Label::Parameters(Some(vec![]))),
            spanned(Label::Means(Some(Means::Block(vec![
                ast::Statement::ReturnStatement(ast::Expression::IntegerLiteral(0)),
            ])))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(matches!(&program.declarations[0], ast::Declaration::Function(f) if f.name == "main"));
    }

    #[test]
    fn test_function_missing_returns() {
        let decl = make_decl(Construct::Function, "foo", vec![
            spanned(Label::Is(Is::Function)),
            spanned(Label::Return(None)),
            spanned(Label::Parameters(Some(vec![]))),
            spanned(Label::Means(Some(Means::Block(vec![])))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("missing required `returns` label"));
    }

    #[test]
    fn test_function_must_have_block() {
        let decl = make_decl(Construct::Function, "foo", vec![
            spanned(Label::Is(Is::Function)),
            spanned(Label::Return(Some(TypeDescription::None))),
            spanned(Label::Parameters(Some(vec![]))),
            spanned(Label::Means(Some(Means::Expression(ast::Expression::IntegerLiteral(0))))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("must have a block body"));
    }

    #[test]
    fn test_singleton_declaration() {
        let decl = make_decl(Construct::Singleton, "Red", vec![
            spanned(Label::Is(Is::Singleton)),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(matches!(&program.declarations[0], ast::Declaration::Singleton(name) if name == "Red"));
    }

    #[test]
    fn test_multiple_declarations() {
        let declarations = vec![
            make_decl(Construct::Singleton, "Red", vec![
                spanned(Label::Is(Is::Singleton)),
            ]),
            make_decl(Construct::Value, "color", vec![
                spanned(Label::Is(Is::Value)),
                spanned(Label::Type(Some(TypeDescription::Some {
                    identifier: "Color".to_string(),
                    generics: vec![],
                }))),
                spanned(Label::Means(Some(Means::Expression(
                    ast::Expression::VariableReference("Red".to_string()),
                )))),
            ]),
        ];

        let result = validate_and_build_ast(declarations);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().declarations.len(), 2);
    }

    #[test]
    fn test_value_with_generic_type() {
        let decl = make_decl(Construct::Value, "names", vec![
            spanned(Label::Is(Is::Value)),
            spanned(Label::Type(Some(TypeDescription::Some {
                identifier: "List".to_string(),
                generics: vec!["String".to_string()],
            }))),
            spanned(Label::Means(Some(Means::Expression(
                ast::Expression::VariableReference("myList".to_string()),
            )))),
        ]);

        let result = validate_and_build_ast(vec![decl]);
        assert!(result.is_ok());
        let program = result.unwrap();
        match &program.declarations[0] {
            ast::Declaration::Value(v) => {
                assert_eq!(v.type_name, "List");
                assert_eq!(v.type_generics, vec!["String".to_string()]);
            }
            _ => panic!("Expected Value"),
        }
    }
}
