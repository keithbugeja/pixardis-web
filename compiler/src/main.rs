//! chroma compiler
//!
//! Compiles C-like code for the Pixardis virtual machine.
//!
//! TODO: [FIXES]
//! - Need to handle the unary operator properly (for non-integer values)
//! - Need to handle empty blocks (i.e. {})
//! - Separate logical and relational operators
//! 
//! TODO: [FEATURES]
//! - Add structs to the language
//! - Add proper variable scope determination (i.e. global, local, function)
//! - Add support for global variables

pub mod common;
pub mod lexer;
pub mod parser;
pub mod analysis;
pub mod codegen;

use common::{
    logger::{
        Logger, 
        LoggerMessage,
    }, 
    status::CompilationResult
};

use lexer::lexer::Lexer;

use parser::{
    parser::Parser, 
    ast::ProgramNode
};

use analysis::{
    semantic::SemanticAnalyser, 
    symbol::ScopeManager
};

use codegen::generator::CodeGenerator;
use codegen::optimiser::*;
use shared::pixardis::{
    PixardisInstruction, 
    pixardis_print_code, 
    pixardis_save_code
};

use std::io;
use std::process;

fn main() -> Result<(), io::Error> {

    // Parse command line arguments; place the results in a context object.
    let context = process_cmd_args();
    
    // Get the file path from the context object.
    let file_path = context.input.as_str();
    
    // Read source file
    let source = shared::io::read_file_to_string(&file_path)?;

    //
    // Initialise logger
    //
    let logger = Logger::new(source.as_str());

    //
    // Create scope manager
    //
    let mut scope_manager = ScopeManager::new();

    //
    // Perform lexical analysis    
    //
    let mut lexer_logger = logger.clone();
    let (lexer, status) = lexical_analysis(&source, &mut lexer_logger).unwrap();
    assert_stage(&logger, status, "Lexical Analysis");

    //
    // Perform parsing and build the syntax tree
    //
    let mut parser_logger = logger.clone();
    let (parser, status) = parse(lexer, &mut parser_logger).unwrap();
    assert_stage(&logger, status, "Parsing");


    //
    // Perform semantic analysis
    //
    let mut analysis_logger = logger.clone();
    let mut analysis_syntax_tree = parser.get_syntax_tree().unwrap();
    let (semantic_analyser, status) = semantic_analysis(&mut analysis_syntax_tree, &mut scope_manager, &mut analysis_logger).unwrap();
    assert_stage(&logger, status, "Semantic Analysis");


    //
    // Perform code generation
    //
    let mut codegen_logger = logger.clone();
    let mut codegen_syntax_tree = semantic_analyser.get_analysed_tree().unwrap(); //parser.get_syntax_tree().unwrap();
    let (program, status) = code_generation(&mut codegen_syntax_tree, &mut scope_manager, &mut codegen_logger).unwrap();
    assert_stage(&logger, status, "Code Generation");

    //
    // Perform code optimisation
    //
    let (optimised_program,status) = code_optimisation(&mut program.clone()).unwrap();
    assert_stage(&logger, status, "Code Optimisation");
    
    //
    // Write generated code to stdout or file
    //
    let show_line_number = context.line_prefix.map_or(false, |show| show);
    let show_scope = context.scope_prefix.map_or(false, |show| show);

    if let Some(output) = context.output {
        if let Err(error) = pixardis_save_code(&optimised_program, &output, show_line_number, show_scope) {
            logger.print_message(LoggerMessage::Error, 
                format!("Failed writing to '{}', error '{}'", output, error).as_str());    
        }
    } else {
        pixardis_print_code(&optimised_program, show_line_number, show_scope);
    }

    Ok(())
}

///
/// Lexical Analysis
/// 
pub fn lexical_analysis<'a>(source: &'a str, logger: &'a mut Logger<'a>) -> Result<(Lexer<'a>, CompilationResult),()> {
    let mut lexer = Lexer::new(source, logger);

    lexer.scan();

    let status = lexer.status().clone();

    Ok((lexer, status))
}

///
/// Parsing
/// 
pub fn parse<'a>(lexer: Lexer<'a>, logger: &'a mut Logger<'a>) -> Result<(Parser<'a>, CompilationResult),()> {
    let mut parser = Parser::new(lexer, logger);

    parser.parse();

    let status = parser.status().clone();

    Ok((parser, status))
}

///
/// Semantic Analysis
/// 
pub fn semantic_analysis<'a>(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Result<(SemanticAnalyser<'a>, CompilationResult),()> {
    let mut semantic_analyser = SemanticAnalyser::new(syntax_tree, scope_manager, logger);
    semantic_analyser.analyse();

    let status = semantic_analyser.status().clone();

    Ok((semantic_analyser, status))
}

///
/// Code Generation
/// 
pub fn code_generation<'a>(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Result<(Vec<(usize, PixardisInstruction)>, CompilationResult), ()>{
    let mut code_generator = CodeGenerator::new(syntax_tree, scope_manager, logger);
    code_generator.generate();

    Ok((code_generator.program_code(), code_generator.status()))
}

///
/// Code Optimisation
/// 
pub fn code_optimisation(code: &mut Vec::<(usize, PixardisInstruction)>) -> Result<(Vec<(usize, PixardisInstruction)>, CompilationResult), ()> {
    Ok((optimise_code_pixardis(code), CompilationResult::Success))
}

///
/// Helper function to assert stage completed successfully
/// 
pub fn assert_stage(logger: &Logger, status: CompilationResult, stage: &str) {
    match status {
        CompilationResult::Success => { 
            logger.print_message(LoggerMessage::Info, format!("{} Complete.", stage).as_str());
        },
        CompilationResult::Warning => {
            logger.print_message(LoggerMessage::Warning, format!("{} Complete with Warnings.", stage).as_str());
        },
        CompilationResult::Failure => {
            logger.print_message(LoggerMessage::Error, format!("{} Failed.", stage).as_str());
            process::exit(1);
        },
        CompilationResult::Pending => {
            logger.print_message(LoggerMessage::Warning, format!("{} Pending.", stage).as_str());
        },
    }
}

//
// Handle command line input using clap
//
use clap::Parser as ClapParser;

#[derive(clap::Parser, Debug)]
#[command(name = "chroma")]
#[command(author = "Keith <bugeja.keith@gmail.com>")]
#[command(version = "0.1")]
#[command(about = "A compiler for the Pixardis (Pixel Art Display) VM.")]
#[command(long_about = 
"------------------------------------------------------------
     ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ███╗ █████╗ 
    ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗ ████║██╔══██╗
    ██║     ███████║██████╔╝██║   ██║██╔████╔██║███████║
    ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╔╝██║██╔══██║
    ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║
     ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝
                                                        
                        Compiler
------------------------------------------------------------")]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    input: String,

    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    #[arg(short, long, help = "Prefixes instructions with line numbers.")]
    line_prefix: Option<bool>,

    #[arg(short, long, help = "Prefixes instructions with scope id.")]
    scope_prefix: Option<bool>,

    //#[arg(short, long, help = "Generate debug information.")]
    //debug: Option<bool>,
}

//
// Process compiler command line arguments
//
fn process_cmd_args() -> Args 
{
    let args = Args::parse();

    args
}