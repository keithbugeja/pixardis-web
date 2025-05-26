use std::cell::RefCell;

use crate::{parser::ast::ProgramNode, common::{logger::{Logger, LoggerError}, status::CompilationResult}};
use super::symbol::{ScopeManager, SymbolEntry, SymbolType};

pub struct SemanticAnalyser<'a> {    
    syntax_tree: &'a mut ProgramNode,
    analysed_tree: Option<RefCell<ProgramNode>>,
    pub scope_manager: &'a mut ScopeManager,
    pub logger: &'a mut Logger<'a>,
    pub type_stack: Vec<SymbolType>,
    status: CompilationResult,
}

impl<'a> SemanticAnalyser<'a> {
    pub fn new(syntax_tree: &'a mut ProgramNode, scope_manager: &'a mut ScopeManager, logger: &'a mut Logger<'a>) -> Self { 
        SemanticAnalyser { 
            syntax_tree,
            analysed_tree: None,
            scope_manager,
            logger,
            type_stack: Vec::<SymbolType>::new(),
            status: CompilationResult::Pending, } 
    }

    pub fn get_analysed_tree(&self) -> Option<ProgramNode> {
        if let Some(analysed_tree) = &self.analysed_tree {
            Some(analysed_tree.borrow().clone())
        } else {
            None
        }
    }

    pub fn status_set(&mut self, status: CompilationResult) {
        self.status = status;
    }

    pub fn status(&self) -> CompilationResult {
        self.status.clone()
    }

    pub fn analyse(&mut self) {       
        // Set success flag (this will be cleared if any errors are encountered)
        self.status_set(CompilationResult::Success);

        // Start semantic analysis. This step assumes the syntax tree has already
        // has already been generated.
        let root_node = self.syntax_tree.clone();
        root_node.accept(self);

        // Save modified tree
        self.analysed_tree = Some(RefCell::new(root_node));
    }

    pub fn push_type(&mut self, symbol_type: SymbolType) {
        self.type_stack.push(symbol_type);
    }

    pub fn pop_type(&mut self) -> Option<SymbolType> {
        self.type_stack.pop()
    }

    pub fn add_variable_to_current_scope(&mut self, name: String, symbol: SymbolEntry) {
        if let Some(scope) = self.scope_manager.current_mut() {
            scope.insert(name, symbol);
        }
    }

    pub fn get_variable_type(&mut self, name: &str) -> Option<SymbolType> {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            Some(symbol.symbol_type.clone())
        } else {
            None
        }
    }

    pub fn get_array_type(&mut self, name: &str) -> Option<SymbolType> {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            match symbol.symbol_type.clone() {
                SymbolType::Array(t, _) => Some((*t).clone()),
                _ => None,
            }
        }  else {
            None
        } 
    }

    pub fn get_function_return_type(&mut self, name: &str) -> Option<SymbolType> {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            symbol.return_type.clone()
        } else {
            None
        }
    }

    pub fn get_function_argument_types(&mut self, name: &str) -> Option<Vec<SymbolEntry>> {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            symbol.params.clone()
        } else {
            None
        }
    }

    pub fn get_scope_return_type(&mut self) -> Option<SymbolType> {
        let mut scope_id = Some(self.scope_manager.current().unwrap().scope_id());
        
        while let Some(_) = scope_id {
            let scope = self.scope_manager.get(scope_id.unwrap()).unwrap();

            if scope.is_function() {
                return scope.return_type();
            }
            
            scope_id = self.scope_manager.get(scope_id.unwrap()).unwrap().parent_scope_id();            
        }

        None
    }

    pub fn is_function_declaration_scope(&mut self) -> bool {
        if let Some(current_scope) = self.scope_manager.current() {
            return current_scope.is_function();
        }

        false
    }

    pub fn assert_type(&mut self, expected_type: SymbolType, message: &str, line_number: usize) {
        if let Some(actual_type) = self.type_stack.pop() {
            if actual_type != expected_type {
                self.logger.print_error(
                    LoggerError::Type,                     
                    &format!("Mismatching types in {}; expected {}, got {}.", message, expected_type.to_string(), actual_type.to_string()),
                    line_number,
                );

                self.status_set(CompilationResult::Failure);
            }
        }
    }

    pub fn assert_variable_type(&mut self, name: &str, line_number: usize) {
        let rhs_type = self.type_stack.pop().unwrap();
        if !self.check_variable_type(&name, rhs_type.clone()) {
            self.logger.print_error(
                LoggerError::Type, 
                format!("Mismatching types, trying to assign '{}' value to '{}'.", rhs_type.to_string(), &name).as_str(),
                line_number,
            );

            self.status_set(CompilationResult::Failure);
        }
    }

    pub fn assert_array_type(&mut self, name: &str, line_number: usize) {
        let rhs_type = self.type_stack.pop().unwrap();
        if !self.check_array_type(&name, rhs_type.clone()) {
            self.logger.print_error(
                LoggerError::Type, 
                format!("Mismatching types, trying to assign '{}' value to array '{}'.", rhs_type.to_string(), &name).as_str(),
                line_number,
            );

            self.status_set(CompilationResult::Failure);
        }
    }

    // Should add a precondition that symbol exists, otherwise
    // an undeclared variable/array error will result in a type error

    pub fn check_array_type(&mut self, name: &str, symbol_type: SymbolType) -> bool {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            match symbol.symbol_type.clone() {
                SymbolType::Array(t, _) => *t == symbol_type,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn check_variable_type(&mut self, name: &str, symbol_type: SymbolType) -> bool {
        if let Some((_, _, symbol)) = self.scope_manager.find_symbol(name) {
            symbol.symbol_type == symbol_type
        } else {
            false
        }
    }

    pub fn check_variable_exists_in_current_scope(&mut self, name: &str) -> bool {     
        let scope = self.scope_manager.current().unwrap();
        
        scope.exists(name)
    }

    pub fn check_variable_exists_in_scope(&mut self, name: &str, scope_id: Option<usize>) -> bool {
        let scope = self.scope_manager.get(scope_id.unwrap()).unwrap();

        scope.exists(name)
    }

    pub fn check_variable_exists(&mut self, name: &str) -> bool {
        if let Some(_) = self.scope_manager.find_symbol(name) {
            true
        } else {
            false
        }
    }

    pub fn enter_scope(&mut self) {
        let _ = self.scope_manager.open(false, None);
    }

    pub fn enter_function_scope(&mut self, return_type: Option<SymbolType>) {
        let _ = self.scope_manager.open(true, return_type);
    }

    pub fn exit_scope(&mut self) {        
        let _ = self.scope_manager.close();
    }
}