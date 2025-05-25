use crate::common::{
    status::CompilationResult,
    logger::{
        Logger, 
        LoggerError
    }
};

use crate::lexer::{
    lexer::Lexer,
    token::TokenKind
};

use super::ast::*;

use std::rc::Rc;

pub struct Parser<'a> {
    lexer: Lexer <'a>,
    syntax_tree: Option<ProgramNode>,
    logger: &'a mut Logger<'a>,
    status: CompilationResult,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, logger: &'a mut Logger<'a>) -> Self { 
        Parser { 
            lexer: lexer, 
            syntax_tree: None,
            logger: logger,
            status: CompilationResult::Pending,
        } 
    }

    fn status_set(&mut self, status: CompilationResult) {
        self.status = status;
    }

    pub fn status(&self) -> CompilationResult {
        self.status.clone()
    }

    pub fn get_line_number(&self) -> usize {
        self.lexer.peek_token().unwrap().line.clone()
    }

    pub fn get_syntax_tree(&self) -> Option<ProgramNode> {
        self.syntax_tree.clone()
    }

    // parse lexer tokens into AST
    pub fn parse(&mut self) {
        // Set success flag (this will be cleared if any errors are encountered)
        self.status_set(CompilationResult::Success);

        let lexer_status = self.lexer.status();

        match lexer_status {
            CompilationResult::Pending => {
                self.logger.print_short_error(
                    LoggerError::Syntax, 
                    "Lexer has not been initialised.",
                );
                self.status_set(CompilationResult::Failure);
                return;
            },
            CompilationResult::Failure => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Lexer has encountered an error. Parsing aborted.",
                    self.get_line_number()
                );
                self.status_set(CompilationResult::Failure);
                return;
            },
            _ => {},
        }

        // Generate syntax tree. Assumes lexer has already been initialised 
        // and lexemes have been scanned.
        self.syntax_tree = self.parse_program();
    }

    // parse entire program
    pub fn parse_program(&mut self) -> Option<ProgramNode> {
        let mut statements = Vec::new();
    
        while let Some(statement) = self.parse_statement() {
            statements.push(statement);
        }
    
        Some(ProgramNode { statements })
    }

    // parse the expected token
    pub fn parse_token(&mut self, token: TokenKind) -> Result<(), ()> {
        if &self.lexer.peek_token().unwrap().kind == &token 
        {
            self.lexer.next_token();
            return Result::Ok(());
        }

        self.logger.print_error(
             LoggerError::Syntax, 
             format!("Invalid token found when expecting {:?}.", token).as_str(),
             self.get_line_number()
        );

        self.status_set(CompilationResult::Failure);

        // we could try to recover from the error here
        // Result::Ok(())
        
        Result::Err(())
    }

    // parse a series of statements enclosed in braces
    pub fn parse_statement_block(&mut self, is_unscoped_block: bool) -> Option<StatementNode> {
        let _ = self.parse_token(TokenKind::OpenBrace).ok()?;
        
        let mut statements = Vec::new();

        while let Some(statement) = self.parse_statement() {
            statements.push(statement);

            if self.lexer.peek_token().unwrap().kind == TokenKind::CloseBrace {
                self.lexer.next_token(); 
                break;
            }
        }

        if is_unscoped_block {
            return Some(StatementNode::UnscopedBlock(UnscopedBlockNode { statements }));
        }

        Some(StatementNode::Block(BlockNode { statements }))
    }

    // parse a statement
    pub fn parse_statement(&mut self) -> Option<StatementNode>{
        let token = self.lexer.peek_token();
        let kind = &token?.kind;
        let mut semicolon = true;

        let result = match kind 
        {
            TokenKind::Clear => {
                self.parse_clear()
            },
            TokenKind::Let => { 
                self.parse_variable_declaration()
            },
            TokenKind::Identifier(_) => { 
                self.parse_assignment()
            },
            TokenKind::Fun => {
                semicolon = false;
                self.parse_function_declaration()
            },
            TokenKind::Print => {
                self.parse_print()
            },
            TokenKind::Delay => {
                self.parse_delay()
            },
            TokenKind::Return => {
                self.parse_return()
            },
            TokenKind::Write => {
                self.parse_write()
            },
            TokenKind::WriteBox => {
                self.parse_write_box()
            },
            TokenKind::OpenBrace => {
                semicolon = false;
                self.parse_statement_block(false)
            },
            TokenKind::If => {
                semicolon = false;
                self.parse_if_else()
            },
            TokenKind::While => {
                semicolon = false;
                self.parse_while()
            },
            TokenKind::For => {
                semicolon = false;
                self.parse_for()
            },
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid statement found.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                None 
            },
        };

        // some statements end in a semicolon
        if semicolon {
            let _ = self.parse_token(TokenKind::SemiColon);
        }

        result
    }

    // parse a factor (literal, identifier, subexpression)
    pub fn parse_factor(&mut self) -> Option<FactorNode> {
        let mut advance_token = true;
        let token = self.lexer.peek_token();
        let kind = &token?.kind;
        let mut result = match kind
        {
            TokenKind::BooleanLiteral(b) => FactorNode::BooleanLiteral(b.clone()),
            TokenKind::IntegerLiteral(i) => FactorNode::IntegerLiteral(i.clone()),
            TokenKind::FloatLiteral(f) => FactorNode::FloatLiteral(f.clone()),
            TokenKind::ColourLiteral(c) => FactorNode::ColourLiteral(c.clone()),
            TokenKind::Width => FactorNode::Width,
            TokenKind::Height => FactorNode::Height,        
            TokenKind::Identifier(i) => FactorNode::Identifier(i.clone()),
            TokenKind::RandomInt => { 
                advance_token = false;
                self.lexer.next_token();

                match self.parse_expression() {
                    Some(expression) => FactorNode::RandomInt(Rc::new(expression)),
                    _ => return None,
                }
            },
            TokenKind::Read => { 
                advance_token = false;
                self.lexer.next_token();

                let expression_x = self.parse_expression()?;
                let _ = self.parse_token(TokenKind::Comma).ok()?;
                let expression_y = self.parse_expression()?;

                FactorNode::Read([Rc::new(expression_x), Rc::new(expression_y)])
            },
            TokenKind::OpenParen => {
                advance_token = false;

                match self.parse_subexpression() {
                    Some(expression) => FactorNode::Subexpression(Rc::new(expression)),
                    _ => return None,
                }
            },
            TokenKind::AdditiveOp(ref s) if s.as_str() == "-" => {
                advance_token = false;
                self.lexer.next_token();

                match self.parse_expression() {
                    Some(expression) => FactorNode::Unary(Rc::new(expression)),
                    _ => return None,
                }
            }
            TokenKind::UnaryOp => {
                advance_token = false;
                self.lexer.next_token();

                match self.parse_expression() {
                    Some(expression) => FactorNode::Unary(Rc::new(expression)),
                    _ => return None,
                }
            }
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Expected expression.",
                    self.get_line_number(),
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };
        
        // If we fetched an identifier, make sure it's not a function call
        match result { 
            FactorNode::Identifier(_) => {                 
                if self.lexer.peek_k_tokens(1).unwrap().kind == TokenKind::OpenParen {
                    advance_token = false;
                    result = match self.parse_function_call() {
                        Some(function_call) => FactorNode::FunctionCall(function_call),
                        _ => return None,
                    };
                }
            }
            _ => {}
        }

        // advance the token if we need to
        if advance_token {
            self.lexer.next_token();
        }

        Some(result)
    }

    // parse expression
    pub fn parse_expression(&mut self) -> Option<ExpressionNode> {
        let line_number = self.get_line_number();

        let factor = match self.parse_factor() {
            Some(factor) => factor,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid expression encountered.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None 
            },
        };

        let operator = match &self.lexer.peek_token().unwrap().kind {
            TokenKind::AdditiveOp(s) => s.clone(),
            TokenKind::MultiplicativeOp(s) => s.clone(),
            TokenKind::RelationalOp(s) => s.clone(),
            
            // Return the typecasting expression if we have an 'as' operator
            TokenKind::As => {
                self.lexer.next_token();

                let type_name = match &self.lexer.next_token().unwrap().kind {
                    TokenKind::Type(s) => s.clone(),
                    _ => {
                        self.logger.print_error(
                            LoggerError::Syntax, 
                            "Invalid variable declaration. Expected type.",
                            self.get_line_number()
                        );
                        return None
                    },
                };

                return Some(ExpressionNode {
                    factor: factor,
                    operator: Some(String::from("as")),
                    expression: Rc::new(None),
                    type_name: Some(type_name),
                    line: line_number,
                });
            }
            
            // Return the factor if we don't have an operator
            _ => return Some(ExpressionNode {
                factor: factor,
                operator: None,
                expression: Rc::new(None),
                type_name: None,
                line: line_number,
            }),
        };

        self.lexer.next_token();

        let expression = match self.parse_expression() {
            Some(expression) => Rc::<Option<ExpressionNode>>::new(Some(expression)),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid expression encountered.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);
       
                return None
            },
        };

        let expression_node = ExpressionNode {
            factor: factor,
            operator: Some(operator),
            expression: expression,
            type_name: None,
            line: line_number,
        };

        Some(expression_node)
    }

    // parse subexpression '(' + expression + ')'
    pub fn parse_subexpression(&mut self) -> Option<ExpressionNode> {
        let _ = self.parse_token(TokenKind::OpenParen).ok()?;
        
        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => return None,
        };

        let _ = self.parse_token(TokenKind::CloseParen).ok()?;

        Some(expression)
    }

    // parse single formal parameter for function signature
    pub fn parse_formal_parameter(&mut self) -> Option<FormalParameterNode> {
        let line_number = self.get_line_number();
        
        let identifier = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Identifier(s) => s.clone(),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid formal parameter. Identifier expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::Colon).ok()?;

        let type_name = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Type(s) => s.clone(),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid formal parameter. Type expected.",
                    self.get_line_number()
                );
                return None
            },
        };

        Some(FormalParameterNode {
            identifier: identifier,
            type_name: type_name,
            line: line_number,
        })
    }

    // parse formal parameter list in function signature
    pub fn parse_formal_parameter_list(&mut self) -> Option<Vec<FormalParameterNode>> {
        let mut formal_parameters = Vec::new();

        if self.lexer.peek_token().unwrap().kind == TokenKind::CloseParen {
            return Some(formal_parameters);
        }
        
        while let Some(formal_parameter) = self.parse_formal_parameter() {
            formal_parameters.push(formal_parameter);

            if self.lexer.peek_token().unwrap().kind != TokenKind::Comma {
                break;
            } else { 
                self.lexer.next_token(); 
            }
        }

        Some(formal_parameters)
    }

    // parse function declaration
    pub fn parse_function_declaration(&mut self) -> Option<StatementNode> {
        let line_number = self.get_line_number();
        
        let _ = self.parse_token(TokenKind::Fun).ok()?;

        let identifier = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Identifier(s) => s.clone(),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid function declaration. Function name expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::OpenParen).ok()?;

        let formal_parameters = match self.parse_formal_parameter_list() {
            Some(formal_parameters) => formal_parameters,
            None => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invlaid function declaration. Formal parameter list expected.",
                    self.get_line_number()
                );
                return None
            },
        };

        let _ = self.parse_token(TokenKind::CloseParen).ok()?;
        let _ = self.parse_token(TokenKind::Arrow).ok()?;

        let return_type = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Type(s) => s.clone(),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid function declaration. Expected return type.",
                    self.get_line_number()
                );       
                return None
            },
        };

        let body = match self.parse_statement_block(true) {
            Some(body) => Rc::new(body),
            _ => return None,
        };

        let function_declaration = FunctionDeclarationNode {
            identifier,
            formal_parameters,
            return_type,
            body,
            line: line_number,
        };

        Some(StatementNode::FunctionDeclaration(function_declaration))
    }

    // parse a call to a function with arguments
    pub fn parse_function_call(&mut self) -> Option<FunctionCallNode> {
        let line_number = self.get_line_number();
        
        let identifier = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Identifier(s) => s.clone(),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid function call. Expected function name.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::OpenParen).ok()?;

        let mut arguments = Vec::new();

        if self.lexer.peek_token().unwrap().kind == TokenKind::CloseParen 
        {
            self.lexer.next_token();

            return Some(FunctionCallNode {
                identifier,
                arguments,
                line: line_number,
            });
        }

        while let Some(expression) = self.parse_expression() 
        {
            arguments.push(expression);

            if self.lexer.peek_token().unwrap().kind != TokenKind::Comma {
                break;
            } else {
                self.lexer.next_token();
            }
        }

        let _ = self.parse_token(TokenKind::CloseParen).ok()?;

        Some(FunctionCallNode {
            identifier,
            arguments,
            line: line_number,
        })
    }

    // parse while statement
    pub fn parse_while(&mut self) -> Option<StatementNode> {
        let line_number = self.get_line_number();
        
        let _ = self.parse_token(TokenKind::While).ok()?;
        
        let expression = match self.parse_subexpression() {
            Some(expression) => expression,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid while statement declaration. Condition expects boolean expression.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let body = match self.parse_statement_block(false) {
            Some(body) => Rc::new(body),
            _ => return None,
        };

        Some(StatementNode::While(WhileNode {
            condition: expression,
            body,
            line: line_number,
        }))
    }

    // parse for loop
    pub fn parse_for(&mut self) -> Option<StatementNode> {
        let line_number = self.get_line_number();
        
        let _ = self.parse_token(TokenKind::For).ok()?;
        let _ = self.parse_token(TokenKind::OpenParen).ok()?;

        let initialiser = match self.lexer.peek_token().unwrap().kind {
            TokenKind::SemiColon => Rc::new(None),
            TokenKind::Identifier(_) => Rc::new(self.parse_assignment()),
            TokenKind::Let => Rc::new(self.parse_variable_declaration()),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid for statement declaration. Initialiser expects variable declaration or assignment.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::SemiColon).ok()?;

        let condition = match self.lexer.peek_token().unwrap().kind {
            TokenKind::SemiColon => None,
            _ => self.parse_expression(),
        };

        let _ = self.parse_token(TokenKind::SemiColon).ok()?;

        let increment = match self.lexer.peek_token().unwrap().kind {
            TokenKind::CloseParen => Rc::new(None),
            _ => Rc::new(self.parse_assignment()),
        };

        let _ = self.parse_token(TokenKind::CloseParen).ok()?;

        let body = match self.parse_statement_block(false) {
            Some(body) => Rc::new(body),
            _ => return None,
        };

        Some(StatementNode::For(ForNode {
            initialiser,
            condition,
            increment,
            body,
            line: line_number,
        }))
    }

    // parse if-else statement
    pub fn parse_if_else(&mut self) -> Option<StatementNode> {
        let line_number = self.get_line_number();
        
        let _ = self.parse_token(TokenKind::If).ok()?;

        let expression = match self.parse_subexpression() {
            Some(expression) => expression,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid if-else statement declaration. Condition expects boolean expression.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let if_block = match self.parse_statement_block(false) {
            Some(if_block) => Rc::new(if_block),
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid if-else statement declaration. Body expects statement block.",
                    self.get_line_number()
                );
       
                return None 
            },
        };

        let else_block;

        let _ = match self.lexer.peek_token().unwrap().kind {
            TokenKind::Else => {
                self.lexer.next_token();
                else_block = Rc::new(self.parse_statement_block(false));
            },
            _ => {
                else_block = Rc::new(None);
            },
        };

        Some(StatementNode::If(IfNode {
                condition: expression,
                body: if_block,
                else_body: else_block,
                line: line_number,
            }
        ))
    }

    // parse print statement
    pub fn parse_print(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::Print).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid __print statement declaration. Expression expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None 
            },
        };

        Some(StatementNode::Print(expression))
    }

    // parse delay statement
    pub fn parse_delay(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::Delay).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid __delay statement declaration. Expression expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        Some(StatementNode::Delay(expression))
    }

    // parse clear statement
    pub fn parse_clear(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::Clear).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => { 
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid __clear statement declaration. Expression expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        Some(StatementNode::Clear(expression))
    }

    // parse return statement
    pub fn parse_return(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::Return).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid return statement. Expression expected.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        Some(StatementNode::Return(expression))
    }

    // parse write statement
    pub fn parse_write(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::Write).ok()?;

        let expression_x = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_y = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_color = self.parse_expression()?;

        Some (StatementNode::Write([
            expression_x, 
            expression_y, 
            expression_color
        ]))
    }

    // parse write_box statement
    pub fn parse_write_box(&mut self) -> Option<StatementNode>{
        let _ = self.parse_token(TokenKind::WriteBox).ok()?;

        let expression_x = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_y = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_w = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_h = self.parse_expression()?;
        let _ = self.parse_token(TokenKind::Comma).ok()?;
        let expression_color = self.parse_expression()?;

        Some (StatementNode::WriteBox([
            expression_x, 
            expression_y, 
            expression_w, 
            expression_h, 
            expression_color
        ]))
    }

    // parse variable declaration
    pub fn parse_variable_declaration(&mut self) -> Option<StatementNode>{
        let line_number = self.get_line_number();
        
        let _ = self.parse_token(TokenKind::Let).ok()?;
        
        let identifier = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Identifier(s) => s.clone(),
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid variable declaration. Expected identifier.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::Colon).ok()?;

        let type_name = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Type(s) => s.clone(),
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid variable declaration. Expected type.",
                    self.get_line_number()
                );
                return None
            },
        };

        let _ = self.parse_token(TokenKind::Equals).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid variable declaration. Expected RHS expression.",
                    self.get_line_number()
                );
                return None
            },
        };

        let variable_declaration_node = VariableDeclarationNode {
            identifier: identifier,
            type_name: type_name,
            expression: expression,
            line: line_number,
        };

        Some(StatementNode::VariableDeclaration(variable_declaration_node))
    }

    // parse assignment
    pub fn parse_assignment(&mut self) -> Option<StatementNode>{
        let line_number = self.get_line_number();
        
        let identifier = match &self.lexer.next_token().unwrap().kind {
            TokenKind::Identifier(s) => s.clone(),
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid assignment. Expected identifier on LHS.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let _ = self.parse_token(TokenKind::Equals).ok()?;

        let expression = match self.parse_expression() {
            Some(expression) => expression,
            _ => {
                self.logger.print_error(
                    LoggerError::Syntax, 
                    "Invalid assignment. Expected expression on RHS.",
                    self.get_line_number()
                );

                self.status_set(CompilationResult::Failure);

                return None
            },
        };

        let assignment_node = AssignmentNode {
            identifier: identifier,
            expression: expression,
            line: line_number,
        };

        Some(StatementNode::Assignment(assignment_node))
    }
}