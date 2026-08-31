use std::collections::HashMap;

use crate::ast::{
    BlueprintDeclaration, Declaration, Expression, FunctionDeclaration, Program, Statement,
};

#[derive(Clone, Debug)]
struct VariableInfo {
    type_name: String,
    mutable: bool,
}

#[derive(Clone, Debug)]
struct FunctionInfo {
    parameter_types: Vec<String>,
    return_type: String,
}

#[derive(Clone, Debug)]
struct BlueprintInfo {
    fields: Vec<(String, String)>,
    constructor_param_count: usize,
    methods: HashMap<String, FunctionInfo>,
    is_declared: bool,
}

pub struct TypeChecker {
    functions: HashMap<String, FunctionInfo>,
    blueprints: HashMap<String, BlueprintInfo>,
    singletons: Vec<String>,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            functions: HashMap::new(),
            blueprints: HashMap::new(),
            singletons: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &Program) -> Result<(), Vec<String>> {
        self.register_declarations(program);
        self.check_declarations(program);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn register_declarations(&mut self, program: &Program) {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(function) => {
                    let info = FunctionInfo {
                        parameter_types: function
                            .parameters
                            .iter()
                            .map(|p| p.type_name.clone())
                            .collect(),
                        return_type: function.return_type.clone(),
                    };
                    self.functions.insert(function.name.clone(), info);
                }
                Declaration::Blueprint(blueprint) => {
                    let fields: Vec<(String, String)> = blueprint
                        .parameters
                        .iter()
                        .map(|p| (p.name.clone(), p.type_name.clone()))
                        .collect();

                    let mut methods = HashMap::new();
                    for method in &blueprint.methods {
                        methods.insert(
                            method.name.clone(),
                            FunctionInfo {
                                parameter_types: method
                                    .parameters
                                    .iter()
                                    .map(|p| p.type_name.clone())
                                    .collect(),
                                return_type: method.return_type.clone(),
                            },
                        );
                    }

                    let param_count = blueprint.parameters.len();
                    self.blueprints.insert(
                        blueprint.name.clone(),
                        BlueprintInfo {
                            fields,
                            constructor_param_count: param_count,
                            methods,
                            is_declared: blueprint.is_declared,
                        },
                    );
                }
                Declaration::Container(container) => {
                    let mut fields: Vec<(String, String)> = container
                        .parameters
                        .iter()
                        .map(|p| (p.name.clone(), p.type_name.clone()))
                        .collect();
                    for computed in &container.computed_values {
                        fields.push((computed.name.clone(), computed.type_name.clone()));
                    }
                    let param_count = container.parameters.len();
                    self.blueprints.insert(
                        container.name.clone(),
                        BlueprintInfo {
                            fields,
                            constructor_param_count: param_count,
                            methods: HashMap::new(),
                            is_declared: false,
                        },
                    );
                }
                Declaration::Singleton(name) => {
                    self.singletons.push(name.clone());
                }
                Declaration::Value(_) => {}
                            }
        }
    }

    fn check_declarations(&mut self, program: &Program) {
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(function) => {
                    self.check_function(function);
                }
                Declaration::Blueprint(blueprint) => {
                    if !blueprint.is_declared {
                        self.check_blueprint(blueprint);
                    }
                }
                Declaration::Container(_) => {}
                Declaration::Singleton(_) => {}
                Declaration::Value(_) => {}
                            }
        }
    }

    fn check_function(&mut self, function: &FunctionDeclaration) {
        let mut scope: HashMap<String, VariableInfo> = HashMap::new();

        for parameter in &function.parameters {
            scope.insert(
                parameter.name.clone(),
                VariableInfo {
                    type_name: parameter.type_name.clone(),
                    mutable: false,
                },
            );
        }

        for statement in &function.body {
            self.check_statement(statement, &mut scope, &function.return_type, &function.name);
        }
    }

    fn check_blueprint(&mut self, blueprint: &BlueprintDeclaration) {
        for method in &blueprint.methods {
            let mut scope: HashMap<String, VariableInfo> = HashMap::new();

            // `self` is implicitly available with the blueprint's type
            scope.insert(
                "self".to_string(),
                VariableInfo {
                    type_name: blueprint.name.clone(),
                    mutable: false,
                },
            );

            // Instance fields are accessible
            for field in &blueprint.parameters {
                scope.insert(
                    field.name.clone(),
                    VariableInfo {
                        type_name: field.type_name.clone(),
                        mutable: false,
                    },
                );
            }

            for parameter in &method.parameters {
                scope.insert(
                    parameter.name.clone(),
                    VariableInfo {
                        type_name: parameter.type_name.clone(),
                        mutable: false,
                    },
                );
            }

            for statement in &method.body {
                self.check_statement(statement, &mut scope, &method.return_type, &method.name);
            }
        }
    }

    fn check_statement(
        &mut self,
        statement: &Statement,
        scope: &mut HashMap<String, VariableInfo>,
        expected_return: &str,
        function_name: &str,
    ) {
        match statement {
            Statement::ValueDeclaration {
                name,
                type_name,
                type_argument,
                assigned_value,
            } => {
                let full_type = match type_argument {
                    Some(arg) => format!("{} of {}", type_name, arg),
                    None => type_name.clone(),
                };
                let actual_type = self.infer_type(assigned_value, scope);
                if let Some(actual) = &actual_type {
                    if !self.types_compatible(actual, type_name) {
                        self.errors.push(format!(
                            "Type mismatch in '{}': declared type '{}' but assigned value has type '{}'",
                            name, full_type, actual
                        ));
                    }
                }
                scope.insert(
                    name.clone(),
                    VariableInfo {
                        type_name: full_type,
                        mutable: false,
                    },
                );
            }
            Statement::MutationStatement { name, new_value } => {
                if let Some(info) = scope.get(name) {
                    if !info.mutable {
                        self.errors.push(format!(
                            "Cannot mutate '{}': it is declared as a value, not a variable",
                            name
                        ));
                    }
                    let expected_type = info.type_name.clone();
                    let actual_type = self.infer_type(new_value, scope);
                    if let Some(actual) = &actual_type {
                        if !self.types_compatible(actual, &expected_type) {
                            self.errors.push(format!(
                                "Type mismatch in mutation of '{}': expected '{}' but got '{}'",
                                name, expected_type, actual
                            ));
                        }
                    }
                } else {
                    self.errors.push(format!("Cannot mutate undefined variable '{}'", name));
                }
            }
            Statement::ReturnStatement(expression) => {
                if expected_return != "nothing" {
                    let actual_type = self.infer_type(expression, scope);
                    if let Some(actual) = &actual_type {
                        if !self.types_compatible(actual, expected_return) {
                            self.errors.push(format!(
                                "Return type mismatch in '{}': expected '{}' but got '{}'",
                                function_name, expected_return, actual
                            ));
                        }
                    }
                }
            }
            Statement::ExpressionStatement(expression) => {
                self.infer_type(expression, scope);
            }
            Statement::IfStatement { condition, body } => {
                let cond_type = self.infer_type(condition, scope);
                if let Some(t) = &cond_type {
                    if t != "Boolean" {
                        self.errors.push(format!(
                            "If condition must be Boolean, got '{}'",
                            t
                        ));
                    }
                }
                self.infer_type(body, scope);
            }
            Statement::WhileLoop { condition, body } => {
                let cond_type = self.infer_type(condition, scope);
                if let Some(t) = &cond_type {
                    if t != "Boolean" {
                        self.errors.push(format!(
                            "While condition must be Boolean, got '{}'",
                            t
                        ));
                    }
                }
                for statement in body {
                    self.check_statement(statement, scope, expected_return, function_name);
                }
            }
        }
    }

    fn infer_type(
        &mut self,
        expression: &Expression,
        scope: &HashMap<String, VariableInfo>,
    ) -> Option<String> {
        match expression {
            Expression::IntegerLiteral(_) => Some("Integer".to_string()),
            Expression::StringLiteral(_) => Some("String".to_string()),
            Expression::BooleanLiteral(_) => Some("Boolean".to_string()),
            Expression::ValueReference(name) => {
                if let Some(info) = scope.get(name) {
                    Some(info.type_name.clone())
                } else if self.singletons.contains(name) {
                    Some("Singleton".to_string())
                } else {
                    self.errors.push(format!("Undefined variable '{}'", name));
                    None
                }
            }
            Expression::Arithmetic { left, right, .. } => {
                let left_type = self.infer_type(left, scope);
                let right_type = self.infer_type(right, scope);
                if let (Some(l), Some(r)) = (&left_type, &right_type) {
                    if l != "Integer" {
                        self.errors.push(format!(
                            "Left side of arithmetic must be Integer, got '{}'", l
                        ));
                    }
                    if r != "Integer" {
                        self.errors.push(format!(
                            "Right side of arithmetic must be Integer, got '{}'", r
                        ));
                    }
                }
                Some("Integer".to_string())
            }
            Expression::Comparison { left, right, .. } => {
                let left_type = self.infer_type(left, scope);
                let right_type = self.infer_type(right, scope);
                if let (Some(l), Some(r)) = (&left_type, &right_type) {
                    if !self.types_compatible(l, r) && !self.types_compatible(r, l) {
                        self.errors.push(format!(
                            "Comparison operands must be the same type: got '{}' and '{}'",
                            l, r
                        ));
                    }
                }
                Some("Boolean".to_string())
            }
            Expression::LogicalBinary { left, right, .. } => {
                let left_type = self.infer_type(left, scope);
                let right_type = self.infer_type(right, scope);
                if let Some(l) = &left_type {
                    if l != "Boolean" {
                        self.errors.push(format!(
                            "Left side of logical operator must be Boolean, got '{}'", l
                        ));
                    }
                }
                if let Some(r) = &right_type {
                    if r != "Boolean" {
                        self.errors.push(format!(
                            "Right side of logical operator must be Boolean, got '{}'", r
                        ));
                    }
                }
                Some("Boolean".to_string())
            }
            Expression::LogicalNot(operand) => {
                let operand_type = self.infer_type(operand, scope);
                if let Some(t) = &operand_type {
                    if t != "Boolean" {
                        self.errors.push(format!(
                            "Operand of 'not' must be Boolean, got '{}'", t
                        ));
                    }
                }
                Some("Boolean".to_string())
            }
            Expression::FunctionCall { name, arguments } => {
                if name == "printLine" || name == "print" {
                    if arguments.len() != 1 {
                        self.errors.push(format!("{} expects exactly 1 argument", name));
                    } else {
                        self.infer_type(&arguments[0], scope);
                    }
                    return Some("nothing".to_string());
                }

                if name == "List" {
                    for argument in arguments {
                        self.infer_type(argument, scope);
                    }
                    return Some("List".to_string());
                }

                // Check if it's a blueprint constructor
                if let Some(blueprint) = self.blueprints.get(name).cloned() {
                    if blueprint.is_declared {
                        self.errors.push(format!(
                            "Cannot instantiate declared blueprint '{}'", name
                        ));
                    }
                    let expected_count = blueprint.constructor_param_count;
                    if arguments.len() != expected_count {
                        self.errors.push(format!(
                            "'{}' expects {} argument(s), got {}",
                            name, expected_count, arguments.len()
                        ));
                    } else {
                        for (index, argument) in arguments.iter().enumerate() {
                            let actual = self.infer_type(argument, scope);
                            if let Some(actual) = &actual {
                                let expected = &blueprint.fields[index].1;
                                if !self.types_compatible(actual, expected) {
                                    self.errors.push(format!(
                                        "Argument {} of '{}': expected '{}', got '{}'",
                                        index + 1, name, expected, actual
                                    ));
                                }
                            }
                        }
                    }
                    return Some(name.clone());
                }

                // Regular function call
                if let Some(func_info) = self.functions.get(name).cloned() {
                    let expected_count = func_info.parameter_types.len();
                    if arguments.len() != expected_count {
                        self.errors.push(format!(
                            "'{}' expects {} argument(s), got {}",
                            name, expected_count, arguments.len()
                        ));
                    } else {
                        for (index, argument) in arguments.iter().enumerate() {
                            let actual = self.infer_type(argument, scope);
                            if let Some(actual) = &actual {
                                let expected = &func_info.parameter_types[index];
                                if !self.types_compatible(actual, expected) {
                                    self.errors.push(format!(
                                        "Argument {} of '{}': expected '{}', got '{}'",
                                        index + 1, name, expected, actual
                                    ));
                                }
                            }
                        }
                    }
                    Some(func_info.return_type.clone())
                } else {
                    self.errors.push(format!("Undefined function '{}'", name));
                    None
                }
            }
            Expression::MemberAccess { object, member } => {
                let object_type = self.infer_type(object, scope);
                if let Some(type_name) = &object_type {
                    let base = self.base_type(type_name).to_string();

                    // Built-in List properties
                    if base == "List" {
                        if member == "size" {
                            return Some("Integer".to_string());
                        }
                        self.errors.push(format!("List has no property '{}'", member));
                        return None;
                    }

                    if let Some(blueprint) = self.blueprints.get(&base).cloned() {
                        if let Some((_, field_type)) =
                            blueprint.fields.iter().find(|(name, _)| name == member)
                        {
                            // Resolve generic type: if field is `T` and type is `Some of String`, resolve to `String`
                            let resolved = self.resolve_generic_type(
                                field_type, &base, type_name,
                            );
                            return Some(resolved);
                        } else {
                            self.errors.push(format!(
                                "Blueprint '{}' has no field '{}'", base, member
                            ));
                        }
                    } else {
                        self.errors.push(format!(
                            "Cannot access member '{}' on non-blueprint type '{}'",
                            member, type_name
                        ));
                    }
                }
                None
            }
            Expression::MethodCall { object, method, arguments } => {
                let object_type = self.infer_type(object, scope);
                if let Some(type_name) = &object_type {
                    let base = self.base_type(type_name).to_string();

                    // Built-in List methods
                    if base == "List" {
                        if method == "at" {
                            for arg in arguments {
                                self.infer_type(arg, scope);
                            }
                            // Return the element type from `List of T`
                            let element_type = type_name.split(" of ").nth(1)
                                .unwrap_or("Unknown").to_string();
                            return Some(element_type);
                        }
                        self.errors.push(format!("List has no method '{}'", method));
                        return None;
                    }

                    if let Some(blueprint) = self.blueprints.get(&base).cloned() {
                        if let Some(method_info) = blueprint.methods.get(method).cloned() {
                            let expected_count = method_info.parameter_types.len();
                            if arguments.len() != expected_count {
                                self.errors.push(format!(
                                    "Method '{}.{}' expects {} argument(s), got {}",
                                    type_name, method, expected_count, arguments.len()
                                ));
                            } else {
                                for (index, argument) in arguments.iter().enumerate() {
                                    let actual = self.infer_type(argument, scope);
                                    if let Some(actual) = &actual {
                                        let expected = &method_info.parameter_types[index];
                                        if !self.types_compatible(actual, expected) {
                                            self.errors.push(format!(
                                                "Argument {} of '{}.{}': expected '{}', got '{}'",
                                                index + 1, type_name, method, expected, actual
                                            ));
                                        }
                                    }
                                }
                            }
                            return Some(method_info.return_type.clone());
                        } else {
                            self.errors.push(format!(
                                "Blueprint '{}' has no method '{}'", type_name, method
                            ));
                        }
                    } else {
                        self.errors.push(format!(
                            "Cannot call method '{}' on non-blueprint type '{}'",
                            method, type_name
                        ));
                    }
                }
                None
            }
            Expression::IfExpression { branches, else_branch } => {
                let mut result_type: Option<String> = None;

                for branch in branches {
                    let cond_type = self.infer_type(&branch.condition, scope);
                    if let Some(t) = &cond_type {
                        if t != "Boolean" {
                            self.errors.push(format!(
                                "If branch condition must be Boolean, got '{}'", t
                            ));
                        }
                    }
                    let branch_type = self.infer_type(&branch.result, scope);
                    if result_type.is_none() {
                        result_type = branch_type;
                    }
                }

                let else_type = self.infer_type(else_branch, scope);
                if result_type.is_none() {
                    result_type = else_type;
                }

                result_type
            }
            Expression::StringTemplate { parts } => {
                for part in parts {
                    if let crate::ast::StringTemplatePart::Expression(expr) = part {
                        self.infer_type(expr, scope);
                    }
                }
                Some("String".to_string())
            }
        }
    }

    fn types_compatible(&self, actual: &str, expected: &str) -> bool {
        if actual == expected {
            return true;
        }
        // Generic type parameters (single uppercase letter) accept any type
        if expected.len() == 1 && expected.chars().all(|c| c.is_uppercase()) {
            return true;
        }
        if actual.len() == 1 && actual.chars().all(|c| c.is_uppercase()) {
            return true;
        }
        // Singletons can be assigned to any user-defined type name
        if actual == "Singleton" {
            return true;
        }
        // Generic compatibility: `Some` is compatible with `Some of String`
        if expected.starts_with(actual) || actual.starts_with(expected) {
            return true;
        }
        // Extract base type from generic: `Some of String` -> base is `Some`
        let actual_base = self.base_type(actual);
        let expected_base = self.base_type(expected);
        if actual_base == expected_base {
            return true;
        }
        false
    }

    fn base_type<'a>(&self, type_name: &'a str) -> &'a str {
        type_name.split(" of ").next().unwrap_or(type_name)
    }

    fn resolve_generic_type(&self, field_type: &str, base_type: &str, full_type: &str) -> String {
        // If field_type is a generic parameter (e.g. "T"), resolve it from the full type
        if let Some(blueprint) = self.blueprints.get(base_type) {
            if blueprint.is_declared {
                return field_type.to_string();
            }
        }
        // Check if this is a generic blueprint with parameters
        // full_type is like "Some of String", extract "String"
        if full_type.contains(" of ") {
            let parts: Vec<&str> = full_type.splitn(2, " of ").collect();
            if parts.len() == 2 {
                let concrete_type = parts[1];
                // If field_type matches a generic param name (single uppercase letter or
                // matches a declared generic), substitute it
                if field_type.len() == 1 && field_type.chars().all(|c| c.is_uppercase()) {
                    return concrete_type.to_string();
                }
            }
        }
        field_type.to_string()
    }
}
