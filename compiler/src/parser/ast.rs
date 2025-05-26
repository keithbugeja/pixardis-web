use std::{cell::RefCell, rc::Rc};

pub trait AbstractSyntaxTreeVisitor
{
    fn visit_program(&mut self, node: &ProgramNode);
    fn visit_block(&mut self, node: &BlockNode);
    fn visit_unscoped_block(&mut self, node: &UnscopedBlockNode);
    fn visit_statement(&mut self, node: &StatementNode);
    fn visit_variable_declaration(&mut self, node: &VariableDeclarationNode);
    fn visit_array_declaration(&mut self, node: &ArrayDeclarationNode);
    fn visit_function_declaration(&mut self, node: &FunctionDeclarationNode);
    fn visit_formal_parameter(&mut self, node: &FormalParameterNode);
    fn visit_assignment(&mut self, node: &AssignmentNode);
    fn visit_expression(&mut self, node: &ExpressionNode);
    fn visit_print(&mut self, node: &PrintNode);
    fn visit_delay(&mut self, node: &ExpressionNode);
    fn visit_clear(&mut self, node: &ExpressionNode);
    fn visit_write(&mut self, node: &[ExpressionNode; 3]);
    fn visit_write_box(&mut self, node: &[ExpressionNode; 5]);
    fn visit_write_line(&mut self, node: &[ExpressionNode; 5]);
    fn visit_return(&mut self, node: &ExpressionNode);
    fn visit_if(&mut self, node: &IfNode);
    fn visit_while(&mut self, node: &WhileNode);
    fn visit_for(&mut self, node: &ForNode);
    fn visit_factor(&mut self, node: &FactorNode);
    fn visit_boolean_literal(&mut self, value: bool);
    fn visit_integer_literal(&mut self, value: i64);
    fn visit_float_literal(&mut self, value: f64);
    fn visit_colour_literal(&mut self, value: String);
    fn visit_width(&mut self);
    fn visit_height(&mut self);
    fn visit_random_int(&mut self, node: &Rc<ExpressionNode>);
    fn visit_read(&mut self, data: &[Rc<ExpressionNode>; 2]);
    fn visit_identifier(&mut self, value: String);
    fn visit_function_call(&mut self, node: &FunctionCallNode);
    fn visit_array_access(&mut self, node: &ArrayAccessNode);
    fn visit_subexpression(&mut self, node: &Rc<ExpressionNode>);
    fn visit_unary(&mut self, node: &Rc<ExpressionNode>);
}

// Program Node : this is the root node of the AST
#[derive(Debug, PartialEq, Clone)]
pub struct ProgramNode {
    pub statements: Vec<StatementNode>,
}

impl ProgramNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_program(self);
    }
}

// Block Node : this is a block of statements
#[derive(Debug, PartialEq, Clone)]
pub struct BlockNode {
    pub statements: Vec<StatementNode>,
}

impl BlockNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_block(self);
    }
}

// Block Node : this is a block of statements
#[derive(Debug, PartialEq, Clone)]
pub struct UnscopedBlockNode {
    pub statements: Vec<StatementNode>,
}

impl UnscopedBlockNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_unscoped_block(self);
    }
}

// Statement Node : enumerates all possible statements types in the language
#[derive(Debug, PartialEq, Clone)]
pub enum StatementNode {
    VariableDeclaration(VariableDeclarationNode),
    ArrayDeclaration(ArrayDeclarationNode),
    FunctionDeclaration(FunctionDeclarationNode),
    Assignment(AssignmentNode),
    Print(PrintNode),
    Delay(ExpressionNode),
    Write([ExpressionNode; 3]),
    WriteBox([ExpressionNode; 5]),
    WriteLine([ExpressionNode; 5]),
    Return(ExpressionNode),
    Block(BlockNode),
    UnscopedBlock(UnscopedBlockNode),
    If(IfNode),
    While(WhileNode),
    For(ForNode),
    Clear(ExpressionNode),
}

impl StatementNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        match self {
            StatementNode::VariableDeclaration(node) => visitor.visit_variable_declaration(node),
            StatementNode::ArrayDeclaration(node) => visitor.visit_array_declaration(node),
            StatementNode::FunctionDeclaration(node) => visitor.visit_function_declaration(node),
            StatementNode::Assignment(node) => visitor.visit_assignment(node),
            StatementNode::Print(node) => visitor.visit_print(node),
            StatementNode::Delay(node) => visitor.visit_delay(node),
            StatementNode::Write(node) => visitor.visit_write(node),
            StatementNode::WriteBox(node) => visitor.visit_write_box(node),
            StatementNode::WriteLine(node) => visitor.visit_write_line(node),
            StatementNode::Return(node) => visitor.visit_return(node),
            StatementNode::Block(node) => visitor.visit_block(node),
            StatementNode::UnscopedBlock(node) => visitor.visit_unscoped_block(node),
            StatementNode::If(node) => visitor.visit_if(node),
            StatementNode::While(node) => visitor.visit_while(node),
            StatementNode::For(node) => visitor.visit_for(node),
            StatementNode::Clear(node) => visitor.visit_clear(node),
        }
    }
}

// If Node : this is an if statement
#[derive(Debug, PartialEq, Clone)]
pub struct IfNode {
    pub condition: ExpressionNode,
    pub body: Rc<StatementNode>,
    pub else_body: Rc<Option<StatementNode>>,
    pub line: usize,
}

impl IfNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_if(self);
    }
}

// While Node : this is a while statement
#[derive(Debug, PartialEq, Clone)]
pub struct WhileNode {
    pub condition: ExpressionNode,
    pub body: Rc<StatementNode>,
    pub line: usize,
}

impl WhileNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_while(self);
    }
}

// For Node : this is a for statement
#[derive(Debug, PartialEq, Clone)]
pub struct ForNode {
    pub initialiser: Rc<Option<StatementNode>>,
    pub condition: Option<ExpressionNode>,
    pub increment: Rc<Option<StatementNode>>,
    pub body: Rc<StatementNode>,
    pub line: usize,
}

impl ForNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_for(self);
    }
}

// Array Declaration Node : this is an array declaration
#[derive(Debug, PartialEq, Clone)]
pub struct ArrayDeclarationNode {
    pub identifier: String,
    pub type_name: String,
    pub size: i64,
    pub initialiser: Option<Vec<ExpressionNode>>,
    pub line: usize,
}

impl ArrayDeclarationNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_array_declaration(self);
    }
}

// Array Access Node : this is an array access through indexing
#[derive(Debug, PartialEq, Clone)]
pub struct ArrayAccessNode{
    pub identifier: String,
    pub index: Rc<ExpressionNode>,
    pub line: usize,
}

impl ArrayAccessNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_array_access(self);
    }
}

// Variable Declaration Node : this is a variable declaration
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclarationNode {
    pub identifier: String,
    pub type_name: String,
    pub expression: ExpressionNode,
    pub line: usize,
}

impl VariableDeclarationNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_variable_declaration(self);
    }
}

// Assignment Node : this is an assignment
#[derive(Debug, PartialEq, Clone)]
pub struct AssignmentNode {
    pub identifier: String,
    pub array_index: Option<ExpressionNode>,
    pub expression: ExpressionNode,
    pub line: usize,
}

impl AssignmentNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_assignment(self);
    }
}

// Formal Parameter Node : this is a formal parameter declaration (x: type)
#[derive(Debug, PartialEq, Clone)]
pub struct FormalParameterNode {
    pub identifier: String,
    pub type_name: String,
    pub size: i64,
    pub line: usize,
}

impl FormalParameterNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_formal_parameter(self);
    }
}

// Function Declaration Node : this is a function declaration
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclarationNode {
    pub identifier: String,
    pub formal_parameters: Vec<FormalParameterNode>,
    pub return_type: String,
    pub return_size: i64,
    pub body: Rc<StatementNode>,
    pub line: usize,
}

impl FunctionDeclarationNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_function_declaration(self);
    }
}

// Function Call Node : this is a function call
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCallNode{
    pub identifier: String,
    pub arguments: Vec<ExpressionNode>,
    pub line: usize,
}

impl FunctionCallNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_function_call(self);
    }
}

// Print Node : this is the print function
#[derive(Debug, PartialEq)]
pub struct PrintNode{
    pub arg_expr: ExpressionNode,
    pub arg_type: RefCell<String>,
    pub line: usize,
}

impl Clone for PrintNode {
    fn clone(&self) -> PrintNode {       
        PrintNode {
            arg_expr: self.arg_expr.clone(),
            arg_type: RefCell::new(self.arg_type.borrow().clone()),
            line: self.line,
        }
    }
}

impl PrintNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_print(self);
    }
}

// Expression Node : factor <operator> <expression>
#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionNode {
    pub factor: FactorNode,
    pub operator: Option<String>,
    pub expression: Rc<Option<ExpressionNode>>,
    pub type_name: Option<String>,
    pub line: usize,
}

impl ExpressionNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        visitor.visit_expression(self);
    }
}

// Factor Node : this is a factor (literal, identifier, function call, subexpression)
#[derive(Debug, PartialEq, Clone)]
pub enum FactorNode {
    BooleanLiteral(bool),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    ColourLiteral(String),
    Width,
    Height,
    RandomInt(Rc<ExpressionNode>),
    Read([Rc<ExpressionNode>; 2]),
    Identifier(String),
    FunctionCall(FunctionCallNode),
    ArrayAccess(ArrayAccessNode),
    Subexpression(Rc<ExpressionNode>),
    Unary(Rc<ExpressionNode>),
}

impl FactorNode {
    pub fn accept(&self, visitor: &mut dyn AbstractSyntaxTreeVisitor) {
        match self {
            FactorNode::BooleanLiteral(value) => visitor.visit_boolean_literal(*value),
            FactorNode::IntegerLiteral(value) => visitor.visit_integer_literal(*value),
            FactorNode::FloatLiteral(value) => visitor.visit_float_literal(*value),
            FactorNode::ColourLiteral(value) => visitor.visit_colour_literal(value.clone()),
            FactorNode::Width => visitor.visit_width(),
            FactorNode::Height => visitor.visit_height(),
            FactorNode::RandomInt(node) => visitor.visit_random_int(node),
            FactorNode::Read(data) => visitor.visit_read(data),
            FactorNode::Identifier(value) => visitor.visit_identifier(value.clone()),
            FactorNode::FunctionCall(node) => visitor.visit_function_call(node),
            FactorNode::ArrayAccess(node) => visitor.visit_array_access(node),
            FactorNode::Subexpression(node) => visitor.visit_subexpression(node),
            FactorNode::Unary(node) => visitor.visit_unary(node),
        }
    }
}