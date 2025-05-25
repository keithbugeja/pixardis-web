use crate::common::logger::LoggerError;
use crate::common::status::CompilationResult;
use crate::parser::ast::AbstractSyntaxTreeVisitor;
use super::semantic::SemanticAnalyser;
use super::symbol::SymbolType;
use super::symbol::SymbolEntry;

impl AbstractSyntaxTreeVisitor for SemanticAnalyser<'_> {
    fn visit_program(&mut self, node: &crate::parser::ast::ProgramNode) {
        self.enter_scope();

        for statement in &node.statements {            
            statement.accept(self);
        }

        self.exit_scope();
    }

    fn visit_block(&mut self, node: &crate::parser::ast::BlockNode) {
        self.enter_scope();

        for statement in &node.statements {
            statement.accept(self);
        }

        self.exit_scope();
    }

    fn visit_unscoped_block(&mut self, node: &crate::parser::ast::UnscopedBlockNode) {
        for statement in &node.statements {
            statement.accept(self);
        }
    }

    fn visit_statement(&mut self, node: &crate::parser::ast::StatementNode) {
        node.accept(self);
    }

    fn visit_variable_declaration(&mut self, node: &crate::parser::ast::VariableDeclarationNode) {
        // Check if variable already exists in current scope
        if self.check_variable_exists_in_current_scope(&node.identifier) {
            self.logger.print_error(
                LoggerError::Semantic, 
                format!("Variable '{}' already exists in current scope.", node.identifier).as_str(),
                node.line,
            );

            self.status_set(CompilationResult::Failure);
        } else {
            self.add_variable_to_current_scope(
                node.identifier.clone(), 
                SymbolEntry {
                    name: node.identifier.clone(), 
                    symbol_type: SymbolType::from_string(node.type_name.as_str()).unwrap(),
                    params: None, 
                    return_type: None, 
                    offset: None,
                }
            );
        }
        
        // Evaluate expression for initialiser
        node.expression.accept(self);

        // Check if initialiser expression type matches variable type
        self.assert_variable_type(&node.identifier, node.line);

    }

    fn visit_function_declaration(&mut self, node: &crate::parser::ast::FunctionDeclarationNode) {
        // Create parameter array
        let mut parameters = Vec::<SymbolEntry>::new();

        // Create symbols for formal parameter list
        for parameter in &node.formal_parameters {
            parameter.accept(self);

            parameters.push(SymbolEntry {
                name: parameter.identifier.clone(),
                symbol_type: SymbolType::from_string(parameter.type_name.as_str()).unwrap(),
                params: None,
                return_type: None,
                offset: None,
            });
        }
        
        // Check if function already exists in current scope
        if self.check_variable_exists_in_current_scope(&node.identifier) {
            self.logger.print_error(
                LoggerError::Semantic, 
                format!("Function '{}' already exists in current scope.", node.identifier).as_str(),
                node.line, 
            );
            self.status_set(CompilationResult::Failure);
        } else {
            // Add function to scope
            self.add_variable_to_current_scope(node.identifier.clone(), 
            SymbolEntry {
                name: node.identifier.clone(), 
                symbol_type: SymbolType::Function,
                params: Some(parameters.clone()),
                return_type: SymbolType::from_string(node.return_type.as_str()), 
                offset: None,
            });
        }

        // Create new scope for function body
        self.enter_function_scope(SymbolType::from_string(node.return_type.as_str()));

        for parameter in parameters {
            self.add_variable_to_current_scope(parameter.name.clone(), parameter);
        }

        // Add parameters to function scope
        node.body.accept(self);

        self.exit_scope();
    }

    fn visit_formal_parameter(&mut self, node: &crate::parser::ast::FormalParameterNode) {        
        // Strictly speaking this is not required since the type is checked during parsing
        // and a syntax error would thrown if the type were invalid
        if let None = SymbolType::from_string(&node.type_name) {
            self.logger.print_error(
                LoggerError::Type,
                &format!("Invalid type '{}' encountered.", node.type_name),
                node.line,
            );
            self.status_set(CompilationResult::Failure);
        }
    }
    
    fn visit_assignment(&mut self, node: &crate::parser::ast::AssignmentNode) {
        // Make sure variable has been declared before assignment
        if !self.check_variable_exists(&node.identifier) {
            self.logger.print_error(
                LoggerError::Semantic, 
                format!("Variable '{}' used but not declared.", node.identifier).as_str(),
                node.line,
            );
            self.status_set(CompilationResult::Failure);
        }

        // Evaluate expression
        node.expression.accept(self);

        self.assert_variable_type(&node.identifier, node.line);
    }

    fn visit_expression(&mut self, node: &crate::parser::ast::ExpressionNode) {
        // factor (lhs) accepts visitor first so that the type is pushed onto the stack
        node.factor.accept(self);

        // pop lhs from stack
        let mut lhs_type = self.pop_type().unwrap();

        // if there is an operator, then there must be a rhs
        let rhs_type;

        if let Some(expression) = &node.expression.as_ref() {
            expression.accept(self);
        }

        if let Some(operator) = &node.operator {
            if operator == "as" {
                lhs_type = SymbolType::from_string(&node.type_name.clone().unwrap().as_str()).unwrap();
            } else {           
                rhs_type = self.pop_type().unwrap();

                if lhs_type != rhs_type {
                    self.logger.print_error(
                        LoggerError::Type, 
                        format!("Mismatching types found in operation '{}' {} '{}'.", lhs_type.to_string(), operator, rhs_type.to_string()).as_str(),
                        node.line,
                    );
                    self.status_set(CompilationResult::Failure);
                }

                match operator.as_str() {
                    "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "and" | "||" | "or" => lhs_type = SymbolType::Bool,
                    _ => (),
                }
            }
        } 

        // Push resulting type on stack        
        self.push_type(lhs_type);
        
    }
    
    fn visit_print(&mut self, node: &crate::parser::ast::ExpressionNode) {
        node.accept(self);

        // We're fine with printing any type
        let _ = self.pop_type();
    }

    fn visit_delay(&mut self, node: &crate::parser::ast::ExpressionNode) {
        // Delay only takes an integer argument
        node.accept(self);
        self.assert_type(SymbolType::Int, "__delay", node.line);
    }
    
    fn visit_clear(&mut self, node: &crate::parser::ast::ExpressionNode) {
        // Clear takes a colour typed argument
        node.accept(self);
        self.assert_type(SymbolType::Colour, "__clear", node.line);
    }

    fn visit_write(&mut self, node: &[crate::parser::ast::ExpressionNode; 3]) {
        // first argument is x position (int)
        node[0].accept(self);
        self.assert_type(SymbolType::Int, "__write", node[0].line);

        // second argument is y position (int)
        node[1].accept(self);
        self.assert_type(SymbolType::Int, "__write", node[1].line);
        
        // third argument is colour (colour)
        node[2].accept(self);
        self.assert_type(SymbolType::Colour, "__write", node[2].line);
    }

    fn visit_write_box(&mut self, node: &[crate::parser::ast::ExpressionNode; 5]) {
        // first argument is x position (int)
        node[0].accept(self);
        self.assert_type(SymbolType::Int, "__write_box", node[0].line);

        // second argument is y position (int)
        node[1].accept(self);
        self.assert_type(SymbolType::Int, "__write_box", node[1].line);
        
        // third argument is width (int)
        node[2].accept(self);
        self.assert_type(SymbolType::Int, "__write_box", node[2].line);

        // fourth argument is height (int)
        node[3].accept(self);
        self.assert_type(SymbolType::Int, "__write_box", node[3].line);

        // fifth argument is colour (colour)
        node[4].accept(self);
        self.assert_type(SymbolType::Colour, "__write_box", node[4].line);
    }

    fn visit_return(&mut self, node: &crate::parser::ast::ExpressionNode) {
        // We're fine with returning any type
        node.accept(self);

        let expected_return_type = self.get_scope_return_type();
        self.assert_type(expected_return_type.unwrap(), "return", node.line);
    }

    fn visit_if(&mut self, node: &crate::parser::ast::IfNode) {
        // Condition expression should be a boolean        
        node.condition.accept(self);
        self.assert_type(SymbolType::Bool, "if", node.line);

        // Accept body
        node.body.accept(self);

        // ... and else block if it exists
        if let Some(else_body) = &node.else_body.as_ref() {
            else_body.accept(self);
        }
    }

    fn visit_while(&mut self, node: &crate::parser::ast::WhileNode) {
        // Condition should be a boolean
        node.condition.accept(self);
        self.assert_type(SymbolType::Bool, "while", node.line);

        node.body.accept(self);
    }

    fn visit_for(&mut self, node: &crate::parser::ast::ForNode) {
        self.enter_scope();
        
        if let Some(initialiser) = &node.initialiser.as_ref() {
            initialiser.accept(self);
        }

        if let Some(condition) = &node.condition {
            condition.accept(self);
            self.assert_type(SymbolType::Bool, "for", node.line);
        }

        if let Some(increment) = node.increment.as_ref() {
            increment.accept(self);
        }

        node.body.accept(self);

        self.exit_scope();
    }

    fn visit_factor(&mut self, node: &crate::parser::ast::FactorNode) {
        node.accept(self)
    }

    fn visit_boolean_literal(&mut self, _value: bool) {
        self.push_type(SymbolType::Bool);
    }

    fn visit_integer_literal(&mut self, _value: i64) {
        self.push_type(SymbolType::Int);
    }

    fn visit_float_literal(&mut self, _value: f64) {
        self.push_type(SymbolType::Float);
    }

    fn visit_colour_literal(&mut self, _value: String) {
        self.push_type(SymbolType::Colour);
    }

    fn visit_width(&mut self) {
        self.push_type(SymbolType::Int);
    }

    fn visit_height(&mut self) {
        self.push_type(SymbolType::Int);
    }

    fn visit_random_int(&mut self, node: &std::rc::Rc<crate::parser::ast::ExpressionNode>) {
        node.accept(self);
        self.assert_type(SymbolType::Int, "random_int", node.line);
        self.push_type(SymbolType::Int);
    }

    fn visit_read(&mut self, node: &[std::rc::Rc<crate::parser::ast::ExpressionNode>; 2]) {       
        // first argument is x position (int)
        node[0].accept(self);
        self.assert_type(SymbolType::Int, "__read", node[0].line);

        // second argument is y position (int)
        node[1].accept(self);
        self.assert_type(SymbolType::Int, "__read", node[1].line);
        
        // Return type is colour
        self.push_type(SymbolType::Colour);
    }

    fn visit_identifier(&mut self, value: String) {
        let symbol = self.get_variable_type(&value);

        if symbol.is_none() {
            self.logger.print_short_error (
                LoggerError::Semantic, 
                format!("Variable '{}' used but not declared.", value).as_str(),
            );            

            self.status_set(CompilationResult::Failure);

            // Recover from error
            self.push_type(SymbolType::Undefined);
        } else {
            self.push_type(symbol.unwrap());
        }
    }

    fn visit_function_call(&mut self, node: &crate::parser::ast::FunctionCallNode) {
        // Make sure function has been declared
        if self.check_variable_type(&node.identifier, SymbolType::Function) == false {
            self.logger.print_error(
                LoggerError::Semantic, 
                format!("Function '{}' has not been declared", node.identifier).as_str(),
                node.line,
            );
            self.status_set(CompilationResult::Failure);
        }

        if let Some(arg_types) = self.get_function_argument_types(&node.identifier) 
        {
            let arg_count = arg_types.len();
            let provided_arg_count = node.arguments.len();
        
            // Make sure correct number of arguments have been provided
            if arg_count != provided_arg_count {
                self.logger.print_error(
                    LoggerError::Semantic,
                    &format!(
                        "Function '{}' expects {} argument(s), {} provided.",
                        node.identifier, arg_count, provided_arg_count
                    ),
                    node.line,
                );
                self.status_set(CompilationResult::Failure);
            }
        
            // Typecheck arguments
            for (i, (argument, argument_type)) in node.arguments.iter().zip(arg_types.iter()).enumerate() {
                argument.accept(self);
        
                self.assert_type(
                    argument_type.symbol_type.clone(),
                    &format!("In function {}, argument {},", node.identifier, i),
                    node.line,
                );
            }
        }
        
        // Typecheck return type
        let return_type = self.get_function_return_type(&node.identifier);
        // If function has no return type, push Undefined to stack (for error recovery)
        if return_type == None {
            self.push_type(SymbolType::Undefined);
        } else {
            self.push_type(return_type.unwrap());
        }
    }

    fn visit_subexpression(&mut self, node: &std::rc::Rc<crate::parser::ast::ExpressionNode>) {
        node.accept(self);
    }

    fn visit_unary(&mut self, node: &std::rc::Rc<crate::parser::ast::ExpressionNode>) {
        node.accept(self);
    }
}