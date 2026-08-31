use std::collections::HashMap;

use crate::ast::{
    ArithmeticOperator, BlueprintDeclaration, ComparisonOperator, ContainerDeclaration,
    ConditionBranch, Declaration, Expression, FunctionDeclaration, LogicalOperator, Program,
    Statement,
};

/// Generates aarch64 macOS assembly from a Solingua AST
pub struct CodeGenerator {
    output: String,
    string_literals: Vec<String>,
    /// Maps variable names to their stack offset within the current function
    local_variables: HashMap<String, usize>,
    /// Maps variable names to their declared type
    variable_types: HashMap<String, String>,
    /// Next available stack slot (each slot is 8 bytes)
    next_stack_offset: usize,
    /// Counter for generating unique labels
    label_counter: usize,
    /// The current function's epilogue label (for return statements)
    current_epilogue_label: String,
    /// Blueprint definitions: name -> (field name, field type) pairs in order
    blueprints: HashMap<String, Vec<(String, String)>>,
    /// Singleton definitions: name -> unique tag value
    singletons: HashMap<String, i64>,
    /// Next singleton tag
    next_singleton_tag: i64,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            output: String::new(),
            string_literals: Vec::new(),
            local_variables: HashMap::new(),
            variable_types: HashMap::new(),
            next_stack_offset: 0,
            label_counter: 0,
            current_epilogue_label: String::new(),
            blueprints: HashMap::new(),
            singletons: HashMap::new(),
            next_singleton_tag: 1,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        let mut functions = Vec::new();

        // First pass: register blueprints and collect functions
        for declaration in &program.declarations {
            match declaration {
                Declaration::Function(function) => {
                    functions.push(function);
                }
                Declaration::Blueprint(blueprint) => {
                    if blueprint.is_declared {
                        continue;
                    }
                    self.register_blueprint(blueprint);
                    // Methods are emitted separately with blueprint-prefixed names
                }
                Declaration::Container(container) => {
                    self.register_container(container);
                }
                Declaration::Singleton(name) => {
                    let tag = self.next_singleton_tag;
                    self.next_singleton_tag += 1;
                    self.singletons.insert(name.clone(), tag);
                }
                Declaration::Value(_) => {}
                            }
        }

        // Collect string literals from all function and method bodies
        for function in &functions {
            for statement in &function.body {
                self.collect_strings(statement);
            }
        }
        for declaration in &program.declarations {
            match declaration {
                Declaration::Blueprint(blueprint) => {
                    for method in &blueprint.methods {
                        for statement in &method.body {
                            self.collect_strings(statement);
                        }
                    }
                }
                Declaration::Container(container) => {
                    for computed in &container.computed_values {
                        self.collect_strings_from_expression(&computed.expression);
                    }
                }
                _ => {}
            }
        }

        // Add built-in strings
        for literal in ["%ld\n", "%s", "true", "false"] {
            if !self.string_literals.contains(&literal.to_string()) {
                self.string_literals.push(literal.to_string());
            }
        }

        // Emit assembly (text first, data after — so codegen can add string literals)
        self.emit_text_section_from_refs(&functions, program);
        self.emit_data_section();

        self.output.clone()
    }

    fn register_container(&mut self, container: &ContainerDeclaration) {
        // Register all fields: parameters + computed values
        let mut fields: Vec<(String, String)> = container
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.type_name.clone()))
            .collect();
        for computed in &container.computed_values {
            fields.push((computed.name.clone(), computed.type_name.clone()));
        }
        self.blueprints.insert(container.name.clone(), fields);
    }

    fn emit_container_constructor(&mut self, container: &ContainerDeclaration) {
        let label = format!("_{}", container.name);
        let param_count = container.parameters.len();
        let computed_count = container.computed_values.len();
        let total_fields = param_count + computed_count;
        let instance_size = total_fields * 8;

        self.local_variables.clear();
        self.variable_types.clear();
        self.next_stack_offset = 16;

        let total_locals = param_count + computed_count + 8;
        let frame_size = align_to_16(16 + total_locals * 8);

        self.emit(&format!("{label}:"));
        self.emit(&format!("    sub sp, sp, #{frame_size}"));
        self.emit(&format!("    stp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add x29, sp, #{}", frame_size - 16));

        // Save constructor arguments to stack and register them as local variables
        for (index, parameter) in container.parameters.iter().enumerate() {
            let offset = self.next_stack_offset;
            self.local_variables.insert(parameter.name.clone(), offset);
            self.variable_types.insert(parameter.name.clone(), parameter.type_name.clone());
            self.next_stack_offset += 8;
            self.emit(&format!("    str x{index}, [sp, #{offset}]"));
        }

        // Evaluate computed values
        let mut computed_results: Vec<usize> = Vec::new();
        for computed in &container.computed_values {
            let offset = self.next_stack_offset;
            self.next_stack_offset += 8;
            self.emit_expression_into_x0(&computed.expression);
            self.emit(&format!("    str x0, [sp, #{offset}]"));
            computed_results.push(offset);
        }

        // Call malloc to allocate the instance
        self.emit(&format!("    mov x0, #{instance_size}"));
        self.emit("    bl _malloc");

        // Store parameter fields
        for index in 0..param_count {
            let param_offset = 16 + index * 8;
            self.emit(&format!("    ldr x9, [sp, #{}]", param_offset));
            self.emit(&format!("    str x9, [x0, #{}]", index * 8));
        }

        // Store computed value fields
        for (index, &result_offset) in computed_results.iter().enumerate() {
            self.emit(&format!("    ldr x9, [sp, #{result_offset}]"));
            self.emit(&format!("    str x9, [x0, #{}]", (param_count + index) * 8));
        }

        // Return instance pointer (in x0)
        self.emit(&format!("    ldp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add sp, sp, #{frame_size}"));
        self.emit("    ret");
        self.emit("");
    }

    fn register_blueprint(&mut self, blueprint: &BlueprintDeclaration) {
        let fields: Vec<(String, String)> = blueprint
            .parameters
            .iter()
            .map(|p| (p.name.clone(), p.type_name.clone()))
            .collect();
        self.blueprints.insert(blueprint.name.clone(), fields);
    }

    fn collect_strings(&mut self, statement: &Statement) {
        match statement {
            Statement::ExpressionStatement(expression) => {
                self.collect_strings_from_expression(expression);
            }
            Statement::ValueDeclaration { assigned_value, .. }
            => {
                self.collect_strings_from_expression(assigned_value);
            }
            Statement::MutationStatement { new_value, .. } => {
                self.collect_strings_from_expression(new_value);
            }
            Statement::ReturnStatement(expression) => {
                self.collect_strings_from_expression(expression);
            }
            Statement::IfStatement { condition, body } => {
                self.collect_strings_from_expression(condition);
                self.collect_strings_from_expression(body);
            }
            Statement::WhileLoop { condition, body } => {
                self.collect_strings_from_expression(condition);
                for statement in body {
                    self.collect_strings(statement);
                }
            }
        }
    }

    fn collect_strings_from_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::StringLiteral(value) => {
                if !self.string_literals.contains(value) {
                    self.string_literals.push(value.clone());
                }
            }
            Expression::FunctionCall { arguments, .. } => {
                for argument in arguments {
                    self.collect_strings_from_expression(argument);
                }
            }
            Expression::IfExpression { branches, else_branch } => {
                for branch in branches {
                    self.collect_strings_from_expression(&branch.condition);
                    self.collect_strings_from_expression(&branch.result);
                }
                self.collect_strings_from_expression(else_branch);
            }
            Expression::Comparison { left, right, .. }
            | Expression::Arithmetic { left, right, .. }
            | Expression::LogicalBinary { left, right, .. } => {
                self.collect_strings_from_expression(left);
                self.collect_strings_from_expression(right);
            }
            Expression::LogicalNot(operand) => {
                self.collect_strings_from_expression(operand);
            }
            Expression::MethodCall { object, arguments, .. } => {
                self.collect_strings_from_expression(object);
                for argument in arguments {
                    self.collect_strings_from_expression(argument);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.collect_strings_from_expression(object);
            }
            Expression::StringTemplate { parts } => {
                for part in parts {
                    match part {
                        crate::ast::StringTemplatePart::Literal(s) => {
                            if !self.string_literals.contains(s) {
                                self.string_literals.push(s.clone());
                            }
                        }
                        crate::ast::StringTemplatePart::Expression(expr) => {
                            self.collect_strings_from_expression(expr);
                        }
                    }
                }
            }
            Expression::IntegerLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::ValueReference(_) => {}
        }
    }

    fn emit_data_section(&mut self) {
        if self.string_literals.is_empty() {
            return;
        }

        self.emit(".section __DATA,__cstring");
        let literals = self.string_literals.clone();
        for (index, literal) in literals.iter().enumerate() {
            self.emit(&format!("_string_{index}:"));
            self.emit(&format!("    .asciz \"{}\"", escape_string(literal)));
        }
        self.emit("");
    }

    fn emit_text_section_from_refs(
        &mut self,
        functions: &[&FunctionDeclaration],
        program: &Program,
    ) {
        self.emit(".section __TEXT,__text");
        self.emit(".globl _main");
        self.emit("");

        // Emit blueprint constructors and methods (skip declared/interface blueprints)
        for declaration in &program.declarations {
            if let Declaration::Blueprint(blueprint) = declaration {
                if blueprint.is_declared {
                    continue;
                }
                self.emit_blueprint_constructor(blueprint);
                for method in &blueprint.methods {
                    self.emit_method(&blueprint.name, method);
                }
            }
        }

        // Emit container constructors
        for declaration in &program.declarations {
            if let Declaration::Container(container) = declaration {
                self.emit_container_constructor(container);
            }
        }

        // Emit regular functions
        for function in functions {
            self.emit_function(function);
        }
    }

    fn emit_blueprint_constructor(&mut self, blueprint: &BlueprintDeclaration) {
        let label = format!("_{}", blueprint.name);
        let field_count = blueprint.parameters.len();
        let instance_size = field_count * 8;

        self.emit(&format!("{label}:"));

        // Prologue
        let frame_size = align_to_16(16 + (field_count + 2) * 8);
        self.emit(&format!("    sub sp, sp, #{frame_size}"));
        self.emit(&format!("    stp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add x29, sp, #{}", frame_size - 16));

        // Save constructor arguments to temp stack slots
        for index in 0..field_count {
            self.emit(&format!("    str x{index}, [sp, #{}]", 16 + index * 8));
        }

        // Call malloc to allocate the instance
        self.emit(&format!("    mov x0, #{instance_size}"));
        self.emit("    bl _malloc");
        // x0 now holds the instance pointer

        // Store each field into the instance
        for index in 0..field_count {
            self.emit(&format!("    ldr x9, [sp, #{}]", 16 + index * 8));
            self.emit(&format!("    str x9, [x0, #{}]", index * 8));
        }

        // Return instance pointer (already in x0)
        self.emit(&format!("    ldp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add sp, sp, #{frame_size}"));
        self.emit("    ret");
        self.emit("");
    }

    fn emit_method(&mut self, blueprint_name: &str, method: &FunctionDeclaration) {
        // Methods are emitted as: _BlueprintName_methodName
        // First parameter is implicitly the instance pointer (self) in x0
        self.local_variables.clear();
        self.variable_types.clear();
        self.next_stack_offset = 16;

        // Count locals: self + explicit params + value declarations
        let explicit_param_count = method.parameters.len();
        let value_declaration_count = method
            .body
            .iter()
            .filter(|s| matches!(s, Statement::ValueDeclaration { .. }))
            .count();
        let total_locals = 1 + explicit_param_count + value_declaration_count + 8;

        let label = format!("_{blueprint_name}_{}", method.name);
        let epilogue_label = format!(".L_{blueprint_name}_{}_epilogue", method.name);
        self.current_epilogue_label = epilogue_label.clone();

        self.emit(&format!("{label}:"));

        let locals_size = total_locals * 8;
        let frame_size = align_to_16(16 + locals_size);

        self.emit(&format!("    sub sp, sp, #{frame_size}"));
        self.emit(&format!("    stp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add x29, sp, #{}", frame_size - 16));

        // Store `self` (instance pointer, arrives in x0)
        let self_offset = self.next_stack_offset;
        self.local_variables.insert("self".to_string(), self_offset);
        self.variable_types.insert("self".to_string(), blueprint_name.to_string());
        self.next_stack_offset += 8;
        self.emit(&format!("    str x0, [sp, #{self_offset}]"));

        // Store explicit parameters (in x1, x2, ...)
        for (index, parameter) in method.parameters.iter().enumerate() {
            let offset = self.next_stack_offset;
            self.local_variables.insert(parameter.name.clone(), offset);
            self.variable_types.insert(parameter.name.clone(), parameter.type_name.clone());
            self.next_stack_offset += 8;
            self.emit(&format!("    str x{}, [sp, #{offset}]", index + 1));
        }

        // Emit method body
        for statement in &method.body {
            self.emit_statement(statement);
        }

        // Epilogue
        self.emit(&format!("{epilogue_label}:"));
        self.emit(&format!("    ldp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add sp, sp, #{frame_size}"));
        self.emit("    ret");
        self.emit("");
    }

    fn emit_function(&mut self, function: &FunctionDeclaration) {
        // Reset local variable state for each function
        self.local_variables.clear();
        self.variable_types.clear();
        // Reserve [sp, #0..#15] (16 bytes) for outgoing variadic/call arguments.
        // Local variables start at offset 16.
        self.next_stack_offset = 16;

        // Count local slots needed: parameters + declarations + temp space for calls
        let parameter_count = function.parameters.len();
        let declaration_count = function
            .body
            .iter()
            .filter(|statement| {
                matches!(
                    statement,
                    Statement::ValueDeclaration { .. }
                )
            })
            .count();
        // +2 for the outgoing arg area, +8 for temporary argument passing
        let total_locals = 2 + parameter_count + declaration_count + 8;

        // Function label
        let label = if function.name == "main" {
            "_main".to_string()
        } else {
            format!("_{}", function.name)
        };

        let epilogue_label = format!(".L_{}_epilogue", function.name);
        self.current_epilogue_label = epilogue_label.clone();

        self.emit(&format!("{label}:"));

        // Function prologue
        let locals_size = total_locals * 8;
        let frame_size = align_to_16(16 + locals_size);

        self.emit(&format!("    sub sp, sp, #{frame_size}"));
        self.emit(&format!("    stp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add x29, sp, #{}", frame_size - 16));

        // Store parameters from registers to stack
        for (index, parameter) in function.parameters.iter().enumerate() {
            let offset = self.next_stack_offset;
            self.local_variables.insert(parameter.name.clone(), offset);
            self.variable_types.insert(parameter.name.clone(), parameter.type_name.clone());
            self.next_stack_offset += 8;
            self.emit(&format!("    str x{index}, [sp, #{offset}]"));
        }

        // Emit function body
        for statement in &function.body {
            self.emit_statement(statement);
        }

        // Default: return 0 from main (if no explicit return reached)
        if function.name == "main" {
            self.emit("    mov x0, #0");
        }

        // Function epilogue
        self.emit(&format!("{epilogue_label}:"));
        self.emit(&format!("    ldp x29, x30, [sp, #{}]", frame_size - 16));
        self.emit(&format!("    add sp, sp, #{frame_size}"));
        self.emit("    ret");
        self.emit("");
    }

    fn emit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ExpressionStatement(expression) => {
                self.emit_expression_into_x0(expression);
            }
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
                self.variable_types.insert(name.clone(), full_type);
                self.emit_value_declaration(name, assigned_value);
            }
            Statement::MutationStatement { name, new_value } => {
                self.emit_mutation(name, new_value);
            }
            Statement::ReturnStatement(expression) => {
                self.emit_return_statement(expression);
            }
            Statement::IfStatement { condition, body } => {
                self.emit_if_statement(condition, body);
            }
            Statement::WhileLoop { condition, body } => {
                self.emit_while_loop(condition, body);
            }
        }
    }

    fn emit_while_loop(&mut self, condition: &Expression, body: &[Statement]) {
        let loop_label = self.next_label("while_loop");
        let end_label = self.next_label("while_end");

        self.emit(&format!("{loop_label}:"));
        self.emit_condition(condition, &end_label);

        for statement in body {
            self.emit_statement(statement);
        }

        self.emit(&format!("    b {loop_label}"));
        self.emit(&format!("{end_label}:"));
    }

    fn emit_if_statement(&mut self, condition: &Expression, body: &Expression) {
        let skip_label = self.next_label("end_if_stmt");
        self.emit_condition(condition, &skip_label);
        self.emit_expression_into_x0(body);
        self.emit(&format!("{skip_label}:"));
    }

    fn emit_value_declaration(&mut self, name: &str, assigned_value: &Expression) {
        let offset = self.next_stack_offset;
        self.next_stack_offset += 8;
        self.local_variables.insert(name.to_string(), offset);

        // Evaluate the initial value into x0, then store on the stack
        self.emit_expression_into_x0(assigned_value);
        self.emit(&format!("    str x0, [sp, #{offset}]"));
    }

    fn emit_mutation(&mut self, name: &str, new_value: &Expression) {
        let offset = *self.local_variables.get(name).unwrap_or_else(|| {
            panic!("Cannot mutate undefined variable: {}", name);
        });
        self.emit_expression_into_x0(new_value);
        self.emit(&format!("    str x0, [sp, #{offset}]"));
    }

    fn emit_return_statement(&mut self, expression: &Expression) {
        // Evaluate expression into x0 (the return register)
        self.emit_expression_into_x0(expression);
        // Jump to function epilogue
        let epilogue_label = self.current_epilogue_label.clone();
        self.emit(&format!("    b {epilogue_label}"));
    }

    /// Evaluate an expression, leaving the result in x0
    fn emit_expression_into_x0(&mut self, expression: &Expression) {
        match expression {
            Expression::IntegerLiteral(value) => {
                self.emit(&format!("    mov x0, #{value}"));
            }
            Expression::BooleanLiteral(value) => {
                let num = if *value { 1 } else { 0 };
                self.emit(&format!("    mov x0, #{num}"));
            }
            Expression::StringLiteral(value) => {
                let index = self.string_literals.iter().position(|s| s == value).unwrap();
                self.emit(&format!("    adrp x0, _string_{index}@PAGE"));
                self.emit(&format!("    add x0, x0, _string_{index}@PAGEOFF"));
            }
            Expression::ValueReference(name) => {
                if let Some(tag) = self.singletons.get(name).copied() {
                    self.emit(&format!("    mov x0, #{tag}"));
                } else {
                    let offset = *self.local_variables.get(name).unwrap_or_else(|| {
                        panic!("Undefined variable: {}", name);
                    });
                    self.emit(&format!("    ldr x0, [sp, #{offset}]"));
                }
            }
            Expression::FunctionCall { name, arguments } => {
                self.emit_function_call(name, arguments);
            }
            Expression::IfExpression { branches, else_branch } => {
                self.emit_if_expression(branches, else_branch);
            }
            Expression::Comparison { left, operator, right } => {
                self.emit_comparison(left, operator, right);
            }
            Expression::Arithmetic { left, operator, right } => {
                self.emit_arithmetic(left, operator, right);
            }
            Expression::LogicalBinary { left, operator, right } => {
                self.emit_logical_binary(left, operator, right);
            }
            Expression::LogicalNot(operand) => {
                self.emit_expression_into_x0(operand);
                self.emit("    cmp x0, #0");
                self.emit("    cset x0, eq");
            }
            Expression::StringTemplate { parts } => {
                self.emit_string_template(parts);
            }
            Expression::MemberAccess { object, member } => {
                self.emit_member_access(object, member);
            }
            Expression::MethodCall { object, method, arguments } => {
                self.emit_method_call(object, method, arguments);
            }
        }
    }

    fn emit_member_access(&mut self, object: &Expression, member: &str) {
        let full_type = self.infer_type(object);
        let base_type = full_type.split(" of ").next().unwrap_or(&full_type).to_string();

        // Built-in List properties
        if base_type == "List" && member == "size" {
            self.emit_expression_into_x0(object);
            // Size is stored at offset 0 of the list
            self.emit("    ldr x0, [x0, #0]");
            return;
        }

        // Load the instance pointer into x0
        self.emit_expression_into_x0(object);
        // Determine the field offset from the blueprint
        let fields = self.blueprints.get(&base_type).unwrap_or_else(|| {
            panic!("Not a blueprint type: {}", base_type);
        }).clone();
        let field_index = fields.iter().position(|(name, _)| name == member).unwrap_or_else(|| {
            panic!("Unknown field '{}' on type '{}'", member, base_type);
        });
        let field_offset = field_index * 8;
        // Load the field value from the instance
        self.emit(&format!("    ldr x0, [x0, #{field_offset}]"));
    }

    fn emit_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
    ) {
        let full_type = self.infer_type(object);
        let type_name = full_type.split(" of ").next().unwrap_or(&full_type).to_string();

        // Built-in List methods
        if type_name == "List" && method == "at" {
            if let Some(index_expr) = arguments.first() {
                let temp = self.next_stack_offset;
                // Evaluate the list pointer
                self.emit_expression_into_x0(object);
                self.emit(&format!("    str x0, [sp, #{}]", temp));
                // Evaluate the index
                self.emit_expression_into_x0(index_expr);
                // Convert 1-based index to 0-based, then offset = (index) * 8
                // Element at 1-based index i is at offset i * 8 (since offset 0 is size)
                self.emit("    lsl x9, x0, #3"); // x9 = index * 8
                self.emit(&format!("    ldr x0, [sp, #{}]", temp)); // x0 = list pointer
                self.emit("    add x0, x0, x9"); // x0 = list + index*8
                self.emit("    ldr x0, [x0]"); // load element
            }
            return;
        }

        let temp_base = self.next_stack_offset;

        // Evaluate the instance pointer (will be x0 / first arg to method)
        self.emit_expression_into_x0(object);
        self.emit(&format!("    str x0, [sp, #{}]", temp_base));

        // Evaluate explicit arguments
        for (index, argument) in arguments.iter().enumerate() {
            self.emit_expression_into_x0(argument);
            self.emit(&format!("    str x0, [sp, #{}]", temp_base + (index + 1) * 8));
        }

        // Load instance into x0, arguments into x1, x2, ...
        self.emit(&format!("    ldr x0, [sp, #{}]", temp_base));
        for index in 0..arguments.len() {
            self.emit(&format!("    ldr x{}, [sp, #{}]", index + 1, temp_base + (index + 1) * 8));
        }

        // Call the method
        self.emit(&format!("    bl _{type_name}_{method}"));
    }

    fn emit_string_template(&mut self, parts: &[crate::ast::StringTemplatePart]) {
        // Strategy: build a format string, evaluate expressions to temp slots,
        // then call sprintf into a malloc'd buffer.
        // Build format string at compile time
        let mut format_string = String::new();
        let mut expressions: Vec<&Expression> = Vec::new();

        for part in parts {
            match part {
                crate::ast::StringTemplatePart::Literal(s) => {
                    format_string.push_str(s);
                }
                crate::ast::StringTemplatePart::Expression(expr) => {
                    // For now, assume all interpolated expressions are strings (%s)
                    // TODO: detect Integer expressions and use %ld
                    if self.expression_is_integer(expr) {
                        format_string.push_str("%ld");
                    } else {
                        format_string.push_str("%s");
                    }
                    expressions.push(expr);
                }
            }
        }

        // Add format string to string literals
        if !self.string_literals.contains(&format_string) {
            self.string_literals.push(format_string.clone());
        }

        let temp_base = self.next_stack_offset;

        // Evaluate each expression and store to temp slots
        for (index, expr) in expressions.iter().enumerate() {
            self.emit_expression_into_x0(expr);
            self.emit(&format!("    str x0, [sp, #{}]", temp_base + index * 8));
        }

        // Allocate buffer (256 bytes)
        self.emit("    mov x0, #256");
        self.emit("    bl _malloc");
        // Save buffer pointer
        self.emit(&format!("    str x0, [sp, #{}]", temp_base + expressions.len() * 8));

        // Call sprintf: sprintf(buffer, format, args...)
        // On Apple ARM64, sprintf is variadic — named params in registers, variadic on stack
        // sprintf(char *buf, const char *fmt, ...) — buf=x0, fmt=x1, variadic on stack
        // Actually sprintf has 2 named params, so buf=x0, fmt=x1, variadic starts on stack

        // Load buffer into x0
        self.emit(&format!("    ldr x0, [sp, #{}]", temp_base + expressions.len() * 8));
        // Load format string into x1
        let format_index = self.string_literals.iter().position(|s| s == &format_string).unwrap();
        self.emit(&format!("    adrp x1, _string_{format_index}@PAGE"));
        self.emit(&format!("    add x1, x1, _string_{format_index}@PAGEOFF"));

        // Store variadic args on the stack at [sp]
        for (index, _) in expressions.iter().enumerate() {
            self.emit(&format!("    ldr x9, [sp, #{}]", temp_base + index * 8));
            self.emit(&format!("    str x9, [sp, #{}]", index * 8));
        }

        self.emit("    bl _sprintf");

        // Result: buffer pointer (reload it into x0)
        self.emit(&format!("    ldr x0, [sp, #{}]", temp_base + expressions.len() * 8));
    }

    fn expression_is_integer(&self, expression: &Expression) -> bool {
        match expression {
            Expression::IntegerLiteral(_) => true,
            Expression::Arithmetic { .. } => true,
            Expression::ValueReference(name) => {
                self.variable_types.get(name).map_or(false, |t| t == "Integer")
            }
            _ => false,
        }
    }

    fn infer_type(&self, expression: &Expression) -> String {
        match expression {
            Expression::ValueReference(name) => {
                self.variable_types.get(name).cloned().unwrap_or_else(|| {
                    panic!("Cannot infer type of variable '{}'", name);
                })
            }
            _ => panic!("Cannot infer type of complex expression for member access"),
        }
    }

    fn emit_if_expression(
        &mut self,
        branches: &[ConditionBranch],
        else_branch: &Expression,
    ) {
        let end_label = self.next_label("end_if");

        for branch in branches {
            let else_label = self.next_label("else");

            // Emit the condition and branch to else_label if false
            self.emit_condition(&branch.condition, &else_label);

            // Then branch: evaluate result into x0
            self.emit_expression_into_x0(&branch.result);
            self.emit(&format!("    b {end_label}"));

            // Else label
            self.emit(&format!("{else_label}:"));
        }

        // Else branch: evaluate into x0
        self.emit_expression_into_x0(else_branch);

        self.emit(&format!("{end_label}:"));
    }

    /// Emit a condition check that branches to `else_label` if false
    fn emit_condition(&mut self, condition: &Expression, else_label: &str) {
        match condition {
            Expression::Comparison { left, operator, right } => {
                let temp_offset = self.next_stack_offset;
                self.next_stack_offset += 8;

                self.emit_expression_into_x0(left);
                self.emit(&format!("    str x0, [sp, #{temp_offset}]"));
                self.emit_expression_into_x0(right);
                self.emit(&format!("    ldr x9, [sp, #{temp_offset}]"));
                self.emit("    cmp x9, x0");

                self.next_stack_offset -= 8;

                // Branch to else if the condition is FALSE
                match operator {
                    ComparisonOperator::GreaterThan => {
                        self.emit(&format!("    b.le {else_label}"));
                    }
                    ComparisonOperator::GreaterThanOrEqual => {
                        self.emit(&format!("    b.lt {else_label}"));
                    }
                    ComparisonOperator::LessThan => {
                        self.emit(&format!("    b.ge {else_label}"));
                    }
                    ComparisonOperator::LessThanOrEqual => {
                        self.emit(&format!("    b.gt {else_label}"));
                    }
                    ComparisonOperator::Equal => {
                        self.emit(&format!("    b.ne {else_label}"));
                    }
                }
            }
            _ => {
                // General case: evaluate condition into x0, branch if zero
                self.emit_expression_into_x0(condition);
                self.emit("    cmp x0, #0");
                self.emit(&format!("    b.eq {else_label}"));
            }
        }
    }

    fn emit_comparison(
        &mut self,
        left: &Expression,
        operator: &ComparisonOperator,
        right: &Expression,
    ) {
        // Produce a boolean value in x0 (1 if true, 0 if false)
        let true_label = self.next_label("cmp_true");
        let end_label = self.next_label("cmp_end");

        let temp_offset = self.next_stack_offset;
        self.next_stack_offset += 8;

        self.emit_expression_into_x0(left);
        self.emit(&format!("    str x0, [sp, #{temp_offset}]"));
        self.emit_expression_into_x0(right);
        self.emit(&format!("    ldr x9, [sp, #{temp_offset}]"));
        self.emit("    cmp x9, x0");

        self.next_stack_offset -= 8;

        // Branch to true_label if the condition IS true
        let branch_instruction = match operator {
            ComparisonOperator::GreaterThan => "b.gt",
            ComparisonOperator::GreaterThanOrEqual => "b.ge",
            ComparisonOperator::LessThan => "b.lt",
            ComparisonOperator::LessThanOrEqual => "b.le",
            ComparisonOperator::Equal => "b.eq",
        };
        self.emit(&format!("    {branch_instruction} {true_label}"));
        self.emit("    mov x0, #0");
        self.emit(&format!("    b {end_label}"));
        self.emit(&format!("{true_label}:"));
        self.emit("    mov x0, #1");
        self.emit(&format!("{end_label}:"));
    }

    fn emit_logical_binary(
        &mut self,
        left: &Expression,
        operator: &LogicalOperator,
        right: &Expression,
    ) {
        let end_label = self.next_label("logical_end");

        // Evaluate left into x0
        self.emit_expression_into_x0(left);
        self.emit("    cmp x0, #0");

        match operator {
            LogicalOperator::And => {
                // Short-circuit: if left is false (0), result is 0
                self.emit(&format!("    b.eq {end_label}"));
            }
            LogicalOperator::Or => {
                // Short-circuit: if left is true (non-zero), result is 1
                let true_label = self.next_label("or_true");
                self.emit(&format!("    b.ne {true_label}"));
                // Left was false, evaluate right
                self.emit_expression_into_x0(right);
                self.emit(&format!("    b {end_label}"));
                // Left was true, result is 1
                self.emit(&format!("{true_label}:"));
                self.emit("    mov x0, #1");
                self.emit(&format!("{end_label}:"));
                return;
            }
        }

        // For `and`: left was true, evaluate right (right's value is the result)
        self.emit_expression_into_x0(right);
        self.emit(&format!("{end_label}:"));
    }

    fn emit_arithmetic(
        &mut self,
        left: &Expression,
        operator: &ArithmeticOperator,
        right: &Expression,
    ) {
        // Save left result on the stack to avoid clobbering during right evaluation
        let temp_offset = self.next_stack_offset;
        self.next_stack_offset += 8;

        self.emit_expression_into_x0(left);
        self.emit(&format!("    str x0, [sp, #{temp_offset}]"));
        self.emit_expression_into_x0(right);
        self.emit(&format!("    ldr x9, [sp, #{temp_offset}]"));

        match operator {
            ArithmeticOperator::Add => self.emit("    add x0, x9, x0"),
            ArithmeticOperator::Subtract => self.emit("    sub x0, x9, x0"),
            ArithmeticOperator::Multiply => self.emit("    mul x0, x9, x0"),
            ArithmeticOperator::Divide => self.emit("    sdiv x0, x9, x0"),
        }

        self.next_stack_offset -= 8;
    }

    fn emit_function_call(&mut self, name: &str, arguments: &[Expression]) {
        match name {
            "printLine" => self.emit_builtin_print_line(arguments),
            "print" => self.emit_builtin_print(arguments),
            "List" => self.emit_list_constructor(arguments),
            _ => self.emit_user_function_call(name, arguments),
        }
    }

    fn emit_builtin_print(&mut self, arguments: &[Expression]) {
        if let Some(argument) = arguments.first() {
            self.emit_expression_into_x0(argument);
            // Use printf("%s", str) — variadic arg on stack
            self.emit("    str x0, [sp]");
            let format_index = self.string_literals.iter().position(|s| s == "%s").unwrap();
            self.emit(&format!("    adrp x0, _string_{format_index}@PAGE"));
            self.emit(&format!("    add x0, x0, _string_{format_index}@PAGEOFF"));
            self.emit("    bl _printf");
        }
    }

    fn emit_list_constructor(&mut self, arguments: &[Expression]) {
        let count = arguments.len();
        // Allocate (count + 1) * 8 bytes: first slot is size, rest are elements
        let alloc_size = (count + 1) * 8;

        // Save arguments to temp slots first
        let temp_base = self.next_stack_offset;
        for (index, argument) in arguments.iter().enumerate() {
            self.emit_expression_into_x0(argument);
            self.emit(&format!("    str x0, [sp, #{}]", temp_base + index * 8));
        }

        // Call malloc
        self.emit(&format!("    mov x0, #{alloc_size}"));
        self.emit("    bl _malloc");
        // x0 = pointer to list

        // Store size at offset 0
        self.emit(&format!("    mov x9, #{count}"));
        self.emit("    str x9, [x0, #0]");

        // Store elements at offset 8, 16, 24, ...
        for index in 0..count {
            self.emit(&format!("    ldr x9, [sp, #{}]", temp_base + index * 8));
            self.emit(&format!("    str x9, [x0, #{}]", (index + 1) * 8));
        }
        // x0 = list pointer (return value)
    }

    fn emit_builtin_print_line(&mut self, arguments: &[Expression]) {
        if let Some(argument) = arguments.first() {
            if self.expression_is_string(argument) {
                self.emit_expression_into_x0(argument);
                self.emit("    bl _puts");
            } else if self.expression_is_boolean(argument) {
                self.emit_print_boolean(argument);
            } else {
                self.emit_expression_into_x0(argument);
                self.emit("    str x0, [sp]");
                let format_index = self.string_literals.iter().position(|s| s == "%ld\n").unwrap();
                self.emit(&format!("    adrp x0, _string_{format_index}@PAGE"));
                self.emit(&format!("    add x0, x0, _string_{format_index}@PAGEOFF"));
                self.emit("    bl _printf");
            }
        } else {
            panic!("printLine expects one argument");
        }
    }

    fn emit_print_boolean(&mut self, argument: &Expression) {
        let false_label = self.next_label("print_false");
        let end_label = self.next_label("print_bool_end");

        self.emit_expression_into_x0(argument);
        self.emit("    cmp x0, #0");
        self.emit(&format!("    b.eq {false_label}"));

        // Print "true"
        let true_index = self.string_literals.iter().position(|s| s == "true").unwrap();
        self.emit(&format!("    adrp x0, _string_{true_index}@PAGE"));
        self.emit(&format!("    add x0, x0, _string_{true_index}@PAGEOFF"));
        self.emit("    bl _puts");
        self.emit(&format!("    b {end_label}"));

        // Print "false"
        self.emit(&format!("{false_label}:"));
        let false_index = self.string_literals.iter().position(|s| s == "false").unwrap();
        self.emit(&format!("    adrp x0, _string_{false_index}@PAGE"));
        self.emit(&format!("    add x0, x0, _string_{false_index}@PAGEOFF"));
        self.emit("    bl _puts");

        self.emit(&format!("{end_label}:"));
    }

    fn emit_user_function_call(&mut self, name: &str, arguments: &[Expression]) {
        // Strategy: evaluate each argument into x0, store to a temp stack area,
        // then load all into x0-x7 before the call. This avoids clobbering.
        let temp_base = self.next_stack_offset;
        let argument_offsets: Vec<usize> = (0..arguments.len())
            .map(|index| temp_base + index * 8)
            .collect();

        // Evaluate each argument and store to temp slots
        for (index, argument) in arguments.iter().enumerate() {
            self.emit_expression_into_x0(argument);
            self.emit(&format!("    str x0, [sp, #{}]", argument_offsets[index]));
        }

        // Load all arguments from temp slots into registers
        for (index, offset) in argument_offsets.iter().enumerate() {
            self.emit(&format!("    ldr x{index}, [sp, #{offset}]"));
        }

        // Call the function
        self.emit(&format!("    bl _{name}"));
        // Result is in x0
    }

    fn expression_is_string(&self, expression: &Expression) -> bool {
        match expression {
            Expression::StringLiteral(_) | Expression::StringTemplate { .. } => true,
            Expression::ValueReference(name) => {
                self.variable_types.get(name).map_or(false, |t| t == "String")
            }
            Expression::MemberAccess { object, member } => {
                let full_type = self.infer_type(object);
                let base_type = full_type.split(" of ").next().unwrap_or(&full_type).to_string();
                let type_arg = full_type.split(" of ").nth(1);
                if let Some(fields) = self.blueprints.get(&base_type) {
                    fields.iter().any(|(name, t)| {
                        if name != member { return false; }
                        if t == "String" { return true; }
                        // Resolve generic: if field type is a single uppercase letter, use type_arg
                        if t.len() == 1 && t.chars().all(|c| c.is_uppercase()) {
                            if let Some(arg) = type_arg {
                                return arg == "String";
                            }
                        }
                        false
                    })
                } else {
                    false
                }
            }
            Expression::MethodCall { object, method, .. } => {
                if method == "at" {
                    let full_type = self.infer_type(object);
                    let type_arg = full_type.split(" of ").nth(1);
                    return type_arg == Some("String");
                }
                false
            }
            _ => false,
        }
    }

    fn expression_is_boolean(&self, expression: &Expression) -> bool {
        match expression {
            Expression::BooleanLiteral(_) => true,
            Expression::Comparison { .. } => true,
            Expression::LogicalBinary { .. } => true,
            Expression::LogicalNot(_) => true,
            Expression::ValueReference(name) => {
                self.variable_types.get(name).map_or(false, |t| t == "Boolean")
            }
            _ => false,
        }
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let label = format!(".L_{prefix}_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
}

/// Round up to the nearest multiple of 16 (required for ARM64 stack alignment)
fn align_to_16(size: usize) -> usize {
    (size + 15) & !15
}

/// Escape special characters for assembly string literals
fn escape_string(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
