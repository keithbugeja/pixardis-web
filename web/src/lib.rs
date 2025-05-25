use shared::pixardis::{PixardisInstruction, pixardis_print_code};
use wasm_bindgen::prelude::*;
use std::sync::Mutex;
use std::collections::VecDeque;

// // Global error collector
// static ERROR_COLLECTOR: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

// // Custom eprintln! replacement
// macro_rules! capture_eprintln {
//     ($($arg:tt)*) => {
//         {
//             let message = format!($($arg)*);
//             ERROR_COLLECTOR.lock().unwrap().push_back(message.clone());
//             // Also log to console for debugging
//             web_sys::console::error_1(&message.into());
//         }
//     };
// }

// Import VM modules from the vm crate
#[path = "../../vm/src/pixardis/mod.rs"]
mod pixardis;

#[path = "../../vm/src/machine/mod.rs"] 
mod machine;

// Import compiler modules directly
#[path = "../../compiler/src/common/mod.rs"]
mod common;

#[path = "../../compiler/src/lexer/mod.rs"]
mod lexer;

#[path = "../../compiler/src/parser/mod.rs"]
mod parser;

#[path = "../../compiler/src/analysis/mod.rs"]
mod analysis;

#[path = "../../compiler/src/codegen/mod.rs"]
mod codegen;

use common::logger::{get_captured_errors, clear_captured_errors};

// Use compiler modules
use common::{
    logger::{Logger, LoggerMessage}, 
    status::CompilationResult
};

use lexer::lexer::Lexer;
use parser::{parser::Parser, ast::ProgramNode};
use analysis::{symbol::ScopeManager, semantic::SemanticAnalyser};
use codegen::generator::CodeGenerator;
use codegen::optimiser::*;

// VM modules
use machine::executor::Executor;
use pixardis::pixardis::{PixardisVirtualMachine, PixardisLogLevel};

// Copy the compilation functions from compiler/main.rs
pub fn lexical_analysis<'a>(source: &'a str, logger: &'a mut Logger<'a>) -> Result<(Lexer<'a>, CompilationResult), ()> {
    let mut lexer = Lexer::new(source, logger);
    lexer.scan();
    let status = lexer.status().clone();
    Ok((lexer, status))
}

pub fn parse<'a>(lexer: Lexer<'a>, logger: &'a mut Logger<'a>) -> Result<(Parser<'a>, CompilationResult), ()> {
    let mut parser = Parser::new(lexer, logger);
    parser.parse();
    let status = parser.status().clone();
    Ok((parser, status))
}

pub fn semantic_analysis<'a>(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Result<CompilationResult, ()> {
    let mut semantic_analyser = SemanticAnalyser::new(syntax_tree, scope_manager, logger);
    semantic_analyser.analyse();
    Ok(semantic_analyser.status())
}

pub fn code_generation<'a>(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Result<(Vec<(usize, PixardisInstruction)>, CompilationResult), ()> {
    let mut code_generator = CodeGenerator::new(syntax_tree, scope_manager, logger);
    code_generator.generate();
    Ok((code_generator.program_code(), code_generator.status()))
}

pub fn code_optimisation(code: &mut Vec<(usize, PixardisInstruction)>) -> Result<(Vec<(usize, PixardisInstruction)>, CompilationResult), ()> {
    Ok((optimise_code_pixardis(code), CompilationResult::Success))
}

#[wasm_bindgen]
pub fn compile_pixardis_source_with_errors(source: &str) -> JsValue {
    // Clear any previous errors
    clear_captured_errors();
    
    let result = std::panic::catch_unwind(|| {
        compile_pixardis_source(source)
    });
    
    // Get the captured error messages
    let captured_errors = get_captured_errors();
    
    match result {
        Ok(Ok(assembly)) => {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "success": true,
                "assembly": assembly,
                "errors": captured_errors
            })).unwrap()
        },
        Ok(Err(error)) => {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "success": false,
                "assembly": "",
                "errors": if captured_errors.is_empty() { vec![error] } else { captured_errors }
            })).unwrap()
        },
        Err(_) => {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "success": false,
                "assembly": "",
                "errors": if captured_errors.is_empty() { vec!["Internal compiler error".to_string()] } else { captured_errors }
            })).unwrap()
        }
    }
}

#[wasm_bindgen]
pub fn compile_pixardis_source(source: &str) -> Result<String, String> {
    let mut logger = Logger::new(source);
    let mut scope_manager = ScopeManager::new();

    // Lexical analysis
    let mut lexer_logger = logger.clone();
    let (lexer, status) = lexical_analysis(source, &mut lexer_logger)
        .map_err(|_| "Lexical analysis failed")?;
    
    if matches!(status, CompilationResult::Failure) {
        return Err("Lexical analysis failed".to_string());
    }

    // Parsing
    let mut parser_logger = logger.clone();
    let (parser, status) = parse(lexer, &mut parser_logger)
        .map_err(|_| "Parsing failed")?;
        
    if matches!(status, CompilationResult::Failure) {
        return Err("Parsing failed".to_string());
    }

    // Semantic analysis
    let mut analysis_logger = logger.clone();
    let mut analysis_syntax_tree = parser.get_syntax_tree()
        .ok_or("Failed to get syntax tree")?; // Changed from map_err to ok_or
    let status = semantic_analysis(&mut analysis_syntax_tree, &mut scope_manager, &mut analysis_logger)
        .map_err(|_| "Semantic analysis failed")?;
        
    if matches!(status, CompilationResult::Failure) {
        return Err("Semantic analysis failed".to_string());
    }

    // Code generation
    let mut codegen_logger = logger.clone();
    let mut codegen_syntax_tree = parser.get_syntax_tree()
        .ok_or("Failed to get syntax tree for codegen")?; // Changed from map_err to ok_or
    let (program, status) = code_generation(&mut codegen_syntax_tree, &mut scope_manager, &mut codegen_logger)
        .map_err(|_| "Code generation failed")?;
        
    if matches!(status, CompilationResult::Failure) {
        return Err("Code generation failed".to_string());
    }

    // Code optimization
    let (optimised_program, _status) = code_optimisation(&mut program.clone())
        .map_err(|_| "Code optimization failed")?;

    // Convert to assembly string
    let assembly = instructions_to_assembly_string(&optimised_program);
    
    Ok(assembly)
}

// Helper function to convert instructions to assembly string
fn instructions_to_assembly_string(instructions: &[(usize, PixardisInstruction)]) -> String {
    use shared::pixardis::pixardis_instruction_to_string; // Import the proper function
    
    let mut assembly = String::new();
    
    for (_index, instruction) in instructions {
        // Use the REAL instruction formatter, not Debug
        assembly.push_str(&format!("{}\n", pixardis_instruction_to_string(instruction.clone())));
    }
    
    assembly
}

#[wasm_bindgen]
pub struct WebVM {
    vm: PixardisVirtualMachine,
}

#[wasm_bindgen]
impl WebVM {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> WebVM {
        let mut vm = PixardisVirtualMachine::new(width, height);
        vm.log_level_set(PixardisLogLevel::None);
        WebVM { vm }
    }
    
    pub fn load_program(&mut self, assembly: &str) {
        self.vm.load_program_from_source(assembly);
    }
    
    pub fn step(&mut self, steps: usize) {
        let _ = self.vm.step(steps);
    }
    
    pub fn get_framebuffer(&self) -> Vec<u8> {
        let (width, height, colors) = self.vm.framebuffer();
        let mut rgb_data = Vec::with_capacity(width * height * 3);
        
        for color in colors {
            rgb_data.push(((color >> 16) & 0xFF) as u8); // R
            rgb_data.push(((color >> 8) & 0xFF) as u8);  // G
            rgb_data.push((color & 0xFF) as u8);         // B
        }
        
        rgb_data
    }
}

// Convenience functions for JS
#[wasm_bindgen]
pub fn create_vm(width: usize, height: usize) -> WebVM {
    WebVM::new(width, height)
}

#[wasm_bindgen]
pub fn step_vm(vm: &mut WebVM, steps: usize) {
    vm.step(steps);
}

#[wasm_bindgen]
pub fn get_vm_framebuffer(vm: &WebVM) -> Vec<u8> {
    vm.get_framebuffer()
}

#[wasm_bindgen]
pub fn load_vm_program(vm: &mut WebVM, assembly: &str) {
    vm.load_program(assembly);
}

// #[macroquad::main("Pixardis Web IDE")]
// async fn main() {
//     let mut source_code = include_str!("../compiler/examples/bounce.ps").to_string();
//     let mut compiled_code = include_str!("../compiler/examples/bounce.pad").to_string();
//     let mut compilation_status = "Press F8 to compile".to_string();
//     let mut debug_info = Vec::<String>::new();

//     // Simple editor state
//     let mut cursor_line = 0;
//     let mut cursor_col = 0;
//     let mut scroll_offset = 0;

//     // Create VM
//     let mut vm = PixardisVirtualMachine::new(64, 48);
//     vm.log_level_set(PixardisLogLevel::None);
//     vm.load_program_from_source(&compiled_code);
    
//     loop {
//         // F8 compilation (keep this working!)
//         if is_key_pressed(KeyCode::F8) {
//             compilation_status = "Compiling...".to_string();
//             debug_info.clear();
            
//             match compile_pixardis_source(&source_code) {
//                 Ok(new_compiled_code) => {
//                     compiled_code = new_compiled_code;
//                     compilation_status = "✅ Compiled successfully!".to_string();
                    
//                     // Reload VM with new code
//                     vm = PixardisVirtualMachine::new(64, 48);
//                     vm.log_level_set(PixardisLogLevel::None);
//                     vm.load_program_from_source(&compiled_code);
//                 },
//                 Err(error) => {
//                     compilation_status = format!("❌ Error: {}", error);
//                 }
//             }
//         }

//         // Drawing
//         clear_background(Color::from_rgba(34, 34, 34, 255));

//         let screen_w = screen_width();
//         let screen_h = screen_height();
        
//         // Left panel - Simple text editor
//         let editor_width = screen_w * 0.5;
//         draw_rectangle(10.0, 10.0, editor_width - 20.0, screen_h - 80.0, Color::from_rgba(40, 40, 40, 255));
        
//         draw_text("📝 Pixardis Code Editor", 20.0, 30.0, 20.0, WHITE);
//         draw_text("F8: Compile | Arrow keys: Navigate | Type to edit", 20.0, screen_h - 50.0, 14.0, GRAY);
//         draw_text(&compilation_status, 20.0, screen_h - 30.0, 14.0, 
//                  if compilation_status.starts_with("✅") { GREEN } else { RED });

//         // Draw the code with cursor
//         for (i, line) in lines.iter().enumerate().skip(scroll_offset).take(25) {
//             let y = 60.0 + (i - scroll_offset) as f32 * 16.0;
//             if y >= screen_h - 100.0 { break; }
            
//             // Line number
//             draw_text(&format!("{:3}:", i + 1), 25.0, y, 12.0, GRAY);
            
//             // Line content
//             draw_text(line, 60.0, y, 12.0, WHITE);
            
//             // Cursor
//             if i == cursor_line {
//                 let cursor_x = 60.0 + cursor_col as f32 * 7.0;
//                 draw_line(cursor_x, y - 12.0, cursor_x, y + 2.0, 2.0, YELLOW);
//             }
//         }

//         // Right panel - VM (unchanged)
//         let vm_x = editor_width + 10.0;
//         let vm_width = screen_w - vm_x - 10.0;
//         let vm_height = screen_h - 20.0;
        
//         draw_rectangle(vm_x, 10.0, vm_width, vm_height, BLACK);
//         draw_text("Pixardis VM (64x48)", vm_x + 10.0, 30.0, 20.0, WHITE);
        
//         let _ = vm.step(100);
        
//         let (width, height, colours) = vm.framebuffer();
//         let cell_size = (vm_width / width as f32).min((vm_height - 100.0) / height as f32);
        
//         let start_x = vm_x + (vm_width - width as f32 * cell_size) / 2.0;
//         let start_y = 50.0 + (vm_height - 100.0 - height as f32 * cell_size) / 2.0;
        
//         for y in 0..height {
//             for x in 0..width {
//                 let colour = colours[y * width + x];
//                 let r = ((colour >> 16) & 0xFF) as u8;
//                 let g = ((colour >> 8) & 0xFF) as u8;
//                 let b = (colour & 0xFF) as u8;
                
//                 draw_rectangle(
//                     start_x + x as f32 * cell_size,
//                     start_y + (height - y - 1) as f32 * cell_size,
//                     cell_size,
//                     cell_size,
//                     Color::from_rgba(r, g, b, 255)
//                 );
//             }
//         }

//         next_frame().await;
//     }
// }