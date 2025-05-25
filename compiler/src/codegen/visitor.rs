use crate::parser::ast::*;
use super::generator::CodeGenerator;
use shared::pixardis::PixardisInstruction;

impl AbstractSyntaxTreeVisitor for CodeGenerator<'_> {
    fn visit_program(&mut self, node: &ProgramNode) {
        // We assume scope with id 0 is the global scope
        self.reset_scope();

        // No need to emit the entry point label since the
        // code organisation step adds it when global scope
        // instructions are grouped together.
        self.emit_code(PixardisInstruction::Label("main".to_string()));

        // This stub enables us to bypass the halt check in the VM
        // initialiser. It is required because the VM expects the
        // program to end with a halt instruction before it runs
        // into a label.
        self.emit_code(PixardisInstruction::PushImmediate("4".to_string()));
        self.emit_code(PixardisInstruction::Jump);
        self.emit_code(PixardisInstruction::Halt);

        let symbol_table = self.symbol_table().unwrap();
        let count = symbol_table.size();
        
        self.emit_code(PixardisInstruction::PushImmediate(count.to_string()));
        self.emit_code(PixardisInstruction::FrameOpen);

        for statement in &node.statements {            
            statement.accept(self);
        }

        self.emit_code(PixardisInstruction::FrameClose);
        self.emit_code(PixardisInstruction::Halt);
    }

    fn visit_block(&mut self, node: &BlockNode) {
        self.next_scope();

        let symbol_table = self.symbol_table().unwrap();
        let count = symbol_table.size();

        self.emit_code(PixardisInstruction::PushImmediate(count.to_string()));
        self.emit_code(PixardisInstruction::FrameOpen);

        for statement in &node.statements {
            statement.accept(self);
        }

        self.emit_code(PixardisInstruction::FrameClose);
        self.previous_scope(); 
    }

    fn visit_unscoped_block(&mut self, node: &UnscopedBlockNode) {
        // We need to use alloc to extend the stack frame
        let symbol_table = self.symbol_table().unwrap();
        let count = symbol_table.size();

        // TODO:
        // Need to subtract parameter count from count
        self.emit_code(PixardisInstruction::PushImmediate(count.to_string()));
        self.emit_code(PixardisInstruction::Allocate);

        for statement in &node.statements {
            statement.accept(self);
        }
    }

    fn visit_statement(&mut self, node: &StatementNode) {
        node.accept(self);
    }

    fn visit_variable_declaration(&mut self, node: &VariableDeclarationNode) {
        // Evaluate expression for initialiser
        node.expression.accept(self);

        // Store expression result onto stack
        let symbol_table = self.symbol_table().unwrap();
        let symbol = symbol_table.get(&node.identifier).unwrap();

        self.emit_code(PixardisInstruction::PushImmediate(symbol.offset.unwrap().to_string()));
        self.emit_code(PixardisInstruction::PushImmediate("0".to_string()));
        self.emit_code(PixardisInstruction::Store);
    }

    fn visit_function_declaration(&mut self, node: &FunctionDeclarationNode) {                
        // Functions are encapsulated with jumps 
        // to prevent execution of function code 
        // without explicit calls
        let patch_function_block_address = self.current_instruction_index();

        // If condition is true, jump to body
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::Jump);
        
        // Create symbols for formal parameter list
        for parameter in &node.formal_parameters {
            parameter.accept(self);
        }

        // Emit label for function entry point
        self.emit_code(PixardisInstruction::Label(node.identifier.clone()));

        // Enter function scope (VM does this automatically)
        self.next_scope();

        // Process function body
        node.body.accept(self);

        // Exit function scope
        self.previous_scope();

        // Patch function prefix to never allow code to 'fall' into function execution.
        let offset_function_block_address = (self.current_instruction_index() - patch_function_block_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_function_block_address), patch_function_block_address);

    }

    fn visit_formal_parameter(&mut self, _node: &FormalParameterNode) {
    }

    fn visit_assignment(&mut self, node: &AssignmentNode) {
        // Evaluate expression
        node.expression.accept(self);
        
        let symbol = self.scope_manager.find_symbol(&node.identifier).unwrap();
        
        let frame = symbol.1.clone().to_string();
        let offset = symbol.2.offset.unwrap().clone().to_string();

        self.emit_code(PixardisInstruction::PushImmediate(offset));
        self.emit_code(PixardisInstruction::PushImmediate(frame));
        self.emit_code(PixardisInstruction::Store);
    }

    fn visit_expression(&mut self, node: &ExpressionNode) {
        // Expression rhs and lhs are traversed in reverse order due to the way 
        // the VM stack works w.r.t. the order of operands
        
        // rhs goes next so that the expression is pushed onto the stack
        if let Some(expression) = &node.expression.as_ref() {
            expression.accept(self);
        }

        // factor (lhs) accepts visitor first so that the type is pushed onto the stack
        node.factor.accept(self);

        // Evaluate operator
        if let Some(operator) = &node.operator {
            match operator.as_str() {
                "+" | "||" | "or" => self.emit_code(PixardisInstruction::Add),
                "-" => self.emit_code(PixardisInstruction::Subtract),
                "*" | "&&" | "and" => self.emit_code(PixardisInstruction::Multiply),
                "/" => self.emit_code(PixardisInstruction::Divide),
                "==" => self.emit_code(PixardisInstruction::Equal),
                "!=" => { 
                    self.emit_code(PixardisInstruction::Equal);
                    self.emit_code(PixardisInstruction::PushImmediate("1".to_string()));
                    self.emit_code(PixardisInstruction::Subtract);
                },
                "<" => self.emit_code(PixardisInstruction::LessThan),
                ">" => self.emit_code(PixardisInstruction::GreaterThan),
                "<=" => self.emit_code(PixardisInstruction::LessEqual),
                ">=" => self.emit_code(PixardisInstruction::GreaterEqual),
                _ => (),
            }
        } 
    }
    
    fn visit_print(&mut self, node: &ExpressionNode) {
        node.accept(self);
        self.emit_code(PixardisInstruction::Print);
    }

    fn visit_delay(&mut self, node: &ExpressionNode) {
        node.accept(self);
        self.emit_code(PixardisInstruction::Delay);
    }

    fn visit_clear(&mut self, node: &ExpressionNode) {
        node.accept(self);
        self.emit_code(PixardisInstruction::Clear);
    }

    fn visit_write(&mut self, node: &[ExpressionNode; 3]) {
        node[2].accept(self);
        node[1].accept(self);
        node[0].accept(self);
        self.emit_code(PixardisInstruction::Write);
    }

    fn visit_write_box(&mut self, node: &[ExpressionNode; 5]) {
        node[4].accept(self);
        node[3].accept(self);
        node[2].accept(self);
        node[1].accept(self);
        node[0].accept(self);
        self.emit_code(PixardisInstruction::WriteBox);
    }

    fn visit_return(&mut self, node: &ExpressionNode) {
        node.accept(self);

        self.push_scope();

        // Pop operands till we reach function frame
        while self.is_function_declaration_scope() == false {
            self.emit_code(PixardisInstruction::FrameClose);
            self.previous_scope();
        }

        self.pop_scope();

        self.emit_code(PixardisInstruction::Return);
    }

    fn visit_if(&mut self, node: &IfNode) {
        // Condition expression should be a boolean        
        node.condition.accept(self);

        let patch_if_block_address = self.current_instruction_index();

        // If condition is true, jump to body
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::ConditionalJump);

        let patch_else_block_address = self.current_instruction_index();

        // Else jump to else block if it exists
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::Jump);

        // Patch if block address
        let offset_if_block_address = (self.current_instruction_index() -  patch_if_block_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_if_block_address), patch_if_block_address);

        // Accept body
        node.body.accept(self);

        // Else block address start
        let mut offset_else_block_address = (self.current_instruction_index() - patch_else_block_address) as i64;

        // ... and else block if it exists
        if let Some(else_body) = &node.else_body.as_ref() {
            let patch_block_end_address = self.current_instruction_index();
            self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
            self.emit_code(PixardisInstruction::Jump);

            offset_else_block_address = (self.current_instruction_index() - patch_else_block_address) as i64;

            else_body.accept(self);

            let offset_block_end_address = (self.current_instruction_index() - patch_block_end_address) as i64;
            self.emit_code_patch(PixardisInstruction::PushOffset(offset_block_end_address), patch_block_end_address);
        }

        // Patch conditional jump
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_else_block_address), patch_else_block_address);
    }

    fn visit_while(&mut self, node: &WhileNode) {
        // Each loop iteration will run the condition expression
        let patch_condition_address = self.current_instruction_index();

        // Condition should be a boolean
        node.condition.accept(self);

        // If successful, jump to body
        let patch_while_block_address = self.current_instruction_index();

        // If condition is true, jump to body
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::ConditionalJump);

        // If unsuccessful, jump to end
        let patch_block_end_address = self.current_instruction_index();

        // End of while block
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::Jump);

        let offset_while_block_address = (self.current_instruction_index() - patch_while_block_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_while_block_address), patch_while_block_address);

        node.body.accept(self);

        // If unsuccessful, jump to end
        let offset_condition_address = patch_condition_address as i64 - self.current_instruction_index() as i64;

        // End of while block (jumps to condition)
        self.emit_code(PixardisInstruction::PushOffset(offset_condition_address as i64));
        self.emit_code(PixardisInstruction::Jump);

        // Patch jump if condition is false
        let offset_block_end_address = (self.current_instruction_index() - patch_block_end_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_block_end_address), patch_block_end_address);
    
    }

    fn visit_for(&mut self, node: &ForNode) {

        // Create a new scope
        self.next_scope();
        
        let symbol_table = self.symbol_table().unwrap();
        let count = symbol_table.size();

        self.emit_code(PixardisInstruction::PushImmediate(count.to_string()));
        self.emit_code(PixardisInstruction::FrameOpen);

        // Initialser
        if let Some(initialiser) = &node.initialiser.as_ref() {
            initialiser.accept(self);
        }

        // Each loop iteration will run the condition expression
        let patch_condition_address = self.current_instruction_index();

        if let Some(condition) = &node.condition {
            condition.accept(self);
        }

        // If successful, jump to body
        let patch_for_block_address = self.current_instruction_index();

        // If condition is true, jump to body
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::ConditionalJump);

        // If unsuccessful, jump to end
        let patch_block_end_address = self.current_instruction_index();

        // End of while block
        self.emit_code(PixardisInstruction::PushOffset(self.current_instruction_index() as i64));
        self.emit_code(PixardisInstruction::Jump);

        let offset_for_block_address = (self.current_instruction_index() - patch_for_block_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_for_block_address), patch_for_block_address);

        // Body
        node.body.accept(self);

        // Increment
        if let Some(increment) = node.increment.as_ref() {
            increment.accept(self);
        }

        // If unsuccessful, jump to end
        let offset_condition_address = patch_condition_address as i64 - self.current_instruction_index() as i64;

        // End of while block (jumps to condition)
        self.emit_code(PixardisInstruction::PushOffset(offset_condition_address as i64));
        self.emit_code(PixardisInstruction::Jump);

        // Patch jump if condition is false
        let offset_block_end_address = (self.current_instruction_index() - patch_block_end_address) as i64;
        self.emit_code_patch(PixardisInstruction::PushOffset(offset_block_end_address), patch_block_end_address); 

        // Close the frame
        self.emit_code(PixardisInstruction::FrameClose);
        self.previous_scope(); 
    }

    fn visit_factor(&mut self, node: &FactorNode) {
        node.accept(self)
    }

    fn visit_boolean_literal(&mut self, value: bool) {
        match value {
            true => self.emit_code(PixardisInstruction::PushImmediate("1".to_string())),
            false => self.emit_code(PixardisInstruction::PushImmediate("0".to_string())),
        }
    }

    fn visit_integer_literal(&mut self, value: i64) {
        self.emit_code(PixardisInstruction::PushImmediate(value.to_string()));
    }

    fn visit_float_literal(&mut self, value: f64) {
        self.emit_code(PixardisInstruction::PushImmediate(value.to_string()));
    }

    fn visit_colour_literal(&mut self, value: String) {
        self.emit_code(PixardisInstruction::PushImmediate(value));
    }

    fn visit_width(&mut self) {
        self.emit_code(PixardisInstruction::Width);
    }

    fn visit_height(&mut self) {
        self.emit_code(PixardisInstruction::Height);
    }

    fn visit_random_int(&mut self, node: &std::rc::Rc<ExpressionNode>) {
        node.accept(self);
        self.emit_code(PixardisInstruction::RandomInt);
    }

    fn visit_read(&mut self, node: &[std::rc::Rc<ExpressionNode>; 2]) {       
        node[1].accept(self);
        node[0].accept(self);
        self.emit_code(PixardisInstruction::Read);
    }

    fn visit_identifier(&mut self, value: String) {
        let symbol = self.scope_manager.find_symbol(value.as_str()).unwrap();
        
        let frame:i64 = symbol.1.clone().to_string().parse().unwrap();
        let offset = symbol.2.offset.clone().unwrap() as i64;

        self.emit_code(PixardisInstruction::PushIndexed([offset, frame]));
    }

    fn visit_function_call(&mut self, node: &FunctionCallNode) {
        let mut arg_count = 0;

        let arguments: Vec<_> = node.arguments.iter().collect();
        arguments.into_iter().rev().for_each(|arg| {
            arg.accept(self);
            arg_count += 1;
        });

        self.emit_code(PixardisInstruction::PushImmediate(arg_count.to_string()));
        self.emit_code(PixardisInstruction::PushLabel(node.identifier.clone()));
        self.emit_code(PixardisInstruction::Call);
    }

    fn visit_subexpression(&mut self, node: &std::rc::Rc<ExpressionNode>) {
        node.accept(self);
    }

    fn visit_unary(&mut self, node: &std::rc::Rc<ExpressionNode>) {
        node.accept(self);

        self.emit_code(PixardisInstruction::PushImmediate("0".to_string()));
        self.emit_code(PixardisInstruction::Subtract);
        
        // self.emit_code(PixardisInstruction::Not);
    }
}