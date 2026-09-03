mod ast;
mod codegen;
mod lexer;
mod parser;
mod typechecker;
mod util;

use codegen::CodeGenerator;
use lexer::Lexer;
use parser::Parser;
use typechecker::TypeChecker;

use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    let verbose = arguments.contains(&"--verbose".to_string());
    let source_path = arguments
        .iter()
        .skip(1)
        .find(|argument| !argument.starts_with("--"))
        .unwrap_or_else(|| {
            eprintln!("Usage: solingua [--verbose] <source.sol>");
            std::process::exit(1);
        });

    let source = fs::read_to_string(source_path).unwrap_or_else(|error| {
        eprintln!("Error reading '{}': {}", source_path, error);
        std::process::exit(1);
    });

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if verbose {
        println!("═══ Lexer Output ═══");
        for spanned in &tokens {
            println!("  {}:{} {:?}", spanned.span.line, spanned.span.column, spanned.token);
        }
        println!();
    }

    // Parse
    let mut parser = Parser::new(tokens, verbose);
    let program = parser.parse_program();

    if verbose {
        println!("═══ Abstract Syntax Tree ═══");
        println!("{:#?}", program);
        println!();
    }

    // Type check
    let mut type_checker = TypeChecker::new();
    if let Err(errors) = type_checker.check(&program) {
        eprintln!("Type errors:");
        for error in &errors {
            eprintln!("  {}", error);
        }
        std::process::exit(1);
    }

    if verbose {
        println!("═══ Type Check ═══");
        println!("  All checks passed.");
        println!();
    }

    // Generate assembly
    let mut code_generator = CodeGenerator::new();
    let assembly = code_generator.generate(&program);

    if verbose {
        println!("═══ Generated Assembly ═══");
        println!("{}", assembly);
    }

    // Derive output filenames from source
    let stem = source_path.trim_end_matches(".sol");
    let assembly_path = format!("{}.s", stem);
    let object_path = format!("{}.o", stem);
    let binary_path = stem.to_string();

    // Write assembly
    fs::write(&assembly_path, &assembly).unwrap_or_else(|error| {
        eprintln!("Error writing assembly: {}", error);
        std::process::exit(1);
    });

    // Assemble
    let status = Command::new("as")
        .args(["-o", &object_path, &assembly_path])
        .status()
        .expect("Failed to run assembler");

    if !status.success() {
        eprintln!("Assembly failed");
        std::process::exit(1);
    }

    // Link against libSystem (provides _puts, _main entry)
    let status = Command::new("ld")
        .args([
            "-o", &binary_path,
            &object_path,
            "-lSystem",
            "-syslibroot", "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk",
            "-e", "_main",
            "-arch", "arm64",
        ])
        .status()
        .expect("Failed to run linker");

    if !status.success() {
        eprintln!("Linking failed");
        std::process::exit(1);
    }

    // Clean up intermediate files
    let _ = fs::remove_file(&assembly_path);
    let _ = fs::remove_file(&object_path);

    println!("Compiled: {}", binary_path);
}
