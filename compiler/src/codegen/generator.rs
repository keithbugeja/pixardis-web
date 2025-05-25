use shared::pixardis::{
    PixardisInstruction, 
    pixardis_print_code
};

use crate::common::logger::Logger;
use crate::common::status::CompilationResult;
use crate::parser::ast::ProgramNode;
use crate::analysis::symbol::*;

pub struct CodeGenerator<'a> {
    syntax_tree: &'a mut ProgramNode,
    pub scope_manager: &'a mut ScopeManager,
    pub scope_index: usize,
    scope_stack: Vec<usize>,
    pub program_code: Vec<(usize, PixardisInstruction)>,
    pub instruction_index: usize,
    pass: usize,
    pub logger: &'a mut Logger<'a>,
    emit_debug: bool,
    status: CompilationResult,
 }

impl<'a> CodeGenerator<'a> {
    pub fn new(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Self { 
        CodeGenerator { 
            syntax_tree,
            scope_manager: scope_manager,
            scope_index: 0,
            scope_stack: Vec::<usize>::new(),
            program_code: Vec::<(usize, PixardisInstruction)>::new(),
            instruction_index: 0,
            pass: 0,
            logger,
            emit_debug: false, 
            status: CompilationResult::Pending,
        } 
    }

    fn status_set(&mut self, status: CompilationResult) {
        self.status = status;
    }

    pub fn status(&self) -> CompilationResult {
        self.status.clone()
    }

    pub fn generate(&mut self) {
        // Set success flag (this will be cleared if any errors are encountered)
        self.status_set(CompilationResult::Success);
        
        // Generate code for the syntax tree; note that this step assumes
        // the semantic analysis step has already been run and the 
        // code generator has been constructed with a valid scope manager.
        let root_node = self.syntax_tree.clone();
        
        self.pass_set(0);
        root_node.accept(self);

        // No need for a second pass (at the moment)
        // self.pass_set(1);
        // root_node.accept(self);

        // Organise code to group global scope instructions together and 
        // label the entry point (.main).
        self.relocate_code();
    }

    pub fn program_code(&self) -> Vec<(usize, PixardisInstruction)> {
        self.program_code.clone()
    }

    pub fn emit_code(&mut self, code: PixardisInstruction) {
        self.instruction_index += 1;
        self.program_code.push((self.scope_id(), code.clone()));
    }

    pub fn emit_code_patch(&mut self, code: PixardisInstruction, index: usize) {
        self.program_code[index].1 = code.clone();
    }

    pub fn relocate_code(&mut self) {
        // Removed relocation code since it was broken.
    }

    pub fn print_code(&self, show_line_numbers: bool, show_scope: bool) {
        pixardis_print_code(&self.program_code, show_line_numbers, show_scope);
    }
    

    pub fn pass(&self) -> usize {
        self.pass.clone()
    }

    pub fn pass_set(&mut self, pass: usize) {
        self.pass = pass;
    }


    pub fn current_instruction_index(&self) -> usize {
        self.instruction_index.clone()
    }

    pub fn symbol_table(&self) -> Option<&SymbolTable> {
        self.scope_manager.current()
    }


    pub fn reset_scope(&mut self) {
        self.scope_index = 0;
        self.scope_stack.clear();
        let _ = self.scope_manager.activate(0);
    }
    
    pub fn push_scope(&mut self) {
        self.scope_stack.push(self.scope_id());
    }

    pub fn pop_scope(&mut self) {
        if let Some(scope_index) = self.scope_stack.pop() {
            let _ = self.scope_manager.activate(scope_index);
        }
    }

    pub fn next_scope(&mut self) {
        self.scope_index += 1;

        let scope_id = self.scope_index;
        let _ = self.scope_manager.activate(scope_id);
    }

    pub fn previous_scope(&mut self) {
        if let Some(scope_id) = self.parent_scope_id() {
            let _ = self.scope_manager.activate(scope_id);
        }
    }

    pub fn scope_id(&self) -> usize {
        self.scope_manager.current().unwrap().scope_id()
    }

    pub fn parent_scope_id(&self) -> Option<usize> {
        self.scope_manager.current().unwrap().parent_scope_id()
    }

    pub fn is_function_declaration_scope(&mut self) -> bool {
        if let Some(current_scope) = self.scope_manager.current() {
            return current_scope.is_function();
        }

        false
    }

}
