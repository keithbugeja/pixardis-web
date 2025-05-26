use shared::pixardis::PixardisInstruction;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum VirtualMachineError {
    StackUnderflow,
    InvalidMemoryAccess,
    InvalidStackFrame,
    InvalidAddress,
    InvalidLabel,
    InvalidOffset,
    InvalidFrame,
    InvalidFrameSize,
    InvalidOperand,
    InvalidCount,
    InvalidArgumentCount,
    InvalidDelay,
    DivisionByZero,
    InstructionError,
    TrapHalt,
}

pub struct AddressStack {
    stack: Vec<usize>,
}

#[allow(dead_code)]
impl AddressStack {
    pub fn new() -> AddressStack {
        AddressStack {
            stack: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, address: usize) {
        self.stack.push(address);
    }

    pub fn pop(&mut self) -> Result<usize,VirtualMachineError> {
        if self.stack.len() > 0 {
            return Ok(self.stack.pop().unwrap());
        }

        Err(VirtualMachineError::StackUnderflow)
    }

    pub fn peek(&self) -> Result<usize,VirtualMachineError> {
        if self.stack.len() > 0 {
            return Ok(self.stack.last().unwrap().clone());
        }

        Err(VirtualMachineError::StackUnderflow)
    }
}

#[derive(Debug, Clone)]
pub enum Operand {
    Unsigned(u64),
    Integer(i64),
    Real(f64),
}

#[derive(Debug, Clone)]
pub struct OperandStack {
    stack: Vec<Operand>,
}

#[allow(dead_code)]
impl OperandStack {
    pub fn new() -> OperandStack {
        OperandStack {
            stack: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, operand: Operand) {
        self.stack.push(operand);
    }

    pub fn pop(&mut self) -> Result<Operand,VirtualMachineError> {
        if self.stack.len() > 0 {
            return Ok(self.stack.pop().unwrap());
        }

        Err(VirtualMachineError::StackUnderflow)
    }

    pub fn peek(&self) -> Result<Operand,VirtualMachineError> {
        if self.stack.len() > 0 {
            let operand = self.stack.last().unwrap().clone();
            return Ok(operand);
        }

        Err(VirtualMachineError::StackUnderflow)
    }

    pub fn dup(&mut self) -> Result<(),VirtualMachineError> {
        let operand = self.peek()?;
        self.push(operand);

        Ok(())
    }
}

#[derive(Debug)]
pub struct StackFrame {
    stack: Vec<Operand>,
}

#[allow(dead_code)]
impl StackFrame {
    pub fn new(size: usize) -> StackFrame {
        StackFrame {
            stack: vec![Operand::Integer(0); size],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    pub fn size(&self) -> usize {
        self.stack.len()
    }

    pub fn alloc(&mut self, size: usize) -> Result<usize,VirtualMachineError> {
        let offset = self.stack.len();
        self.stack.resize(self.stack.len() + size, Operand::Integer(0));

        Ok(offset)
    }

    pub fn read(&self, offset: usize) -> Result<Operand,VirtualMachineError> {
        if offset < self.stack.len() {
            if let Some(operand) = self.stack.get(offset) {
                return Ok(operand.clone());
            }
        }
        
        Err(VirtualMachineError::InvalidMemoryAccess)
    }

    pub fn write(&mut self, offset: usize, operand: Operand) -> Result<(),VirtualMachineError> {
        if offset < self.stack.len() {
            self.stack[offset] = operand.clone();
            return Ok(());
        }
        
        Err(VirtualMachineError::InvalidMemoryAccess)
    }
}

pub struct Memory {
    stack: Vec<StackFrame>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            stack: Vec::new(),
        }
    }

    fn stack_frame_to_index(&self, frame: usize) -> Result<usize, VirtualMachineError> {
        let frame_index = self.stack.len() - frame - 1;

        if frame_index >= self.stack.len() {
            return Err(VirtualMachineError::InvalidStackFrame);
        }

        Ok(frame_index)
    }

    pub fn frame_open(&mut self, allocation: usize) {
        self.stack.push(StackFrame::new(allocation));
    }

    pub fn frame_close(&mut self) {
        if self.stack.len() > 0 {
            self.stack.pop();
        }
    }

    pub fn frame_alloc(&mut self, size: usize) -> Result<(), VirtualMachineError> {
        if let Some(stack_frame) = self.stack.last_mut() {
            stack_frame.alloc(size)?;
        }

        Ok(())
    }

    pub fn read(&self, frame: usize, offset: usize) -> Result<Operand,VirtualMachineError> { 
        let frame_index = self.stack_frame_to_index(frame)?;
        if let Some(stack_frame) = self.stack.get(frame_index) {
            return Ok(stack_frame.read(offset)?);
        }

        Err(VirtualMachineError::InvalidMemoryAccess)
    }

    pub fn write(&mut self, frame: usize, offset: usize, operand: Operand) -> Result<(),VirtualMachineError> {
        let frame_index = self.stack_frame_to_index(frame)?;
        if let Some(stack_frame) = self.stack.get_mut(frame_index) {
            return Ok(stack_frame.write(offset, operand)?);
        }

        Err(VirtualMachineError::InvalidMemoryAccess)
    }
}

type Instruction = PixardisInstruction;

#[derive(Debug, Clone)]
pub enum VirtualMachineState {
    Running,
    Paused,
    Stopped,
    Delayed(f64, f64),
}

pub struct VirtualMachine {
    memory: Memory,
    address_stack: AddressStack,
    operand_stack: OperandStack,

    program: Vec<PixardisInstruction>,
    program_counter: usize,
    address_map: HashMap<String, usize>,

    state: VirtualMachineState,
}

#[allow(dead_code)]
impl VirtualMachine
{
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            memory: Memory::new(),
            address_stack: AddressStack::new(),
            operand_stack: OperandStack::new(),

            program: Vec::new(),
            program_counter: 0,
            address_map: HashMap::new(),

            state: VirtualMachineState::Stopped,
        }
    }

    // pub fn print_operand_stack(&self) {
    //     println!("operand_stack: {:?}", self.operand_stack);
    // }

    pub fn state(&self) -> VirtualMachineState {
        self.state.clone()
    }

    pub fn state_set(&mut self, state: VirtualMachineState) {
        self.state = state;
    }

    /*
     *
     */
    pub fn random_integer(&mut self, value: i64) -> i64 {
        // self.random_number_generator.gen_range(0..value)
        fastrand::i64(0..value)
    }

    /*
     * program counter and execution sub-system
     */

    pub fn program_counter(&self) -> usize {
        self.program_counter
    }

    pub fn program_counter_set_absolute(&mut self, program_counter: usize) {
        self.program_counter = program_counter;
    }

    pub fn program_counter_set_relative(&mut self, offset: i64) {
        self.program_counter = (self.program_counter as i64 + offset) as usize;
    }

    pub fn program_counter_increment(&mut self) {
        self.program_counter += 1;
    }

    /*
     * memory sub-system
     */

    pub fn memory_frame_open(&mut self, size: usize) {
        self.memory.frame_open(size);
    }

    pub fn memory_frame_close(&mut self) {
        self.memory.frame_close();
    }

    pub fn memory_frame_alloc(&mut self, size: usize) -> Result<(), VirtualMachineError> {
        self.memory.frame_alloc(size)?;

        Ok(())
    }

    pub fn memory_write(&mut self, frame: usize, offset: usize, operand: Operand) -> Result<(),VirtualMachineError> {
        Ok(self.memory.write(frame, offset, operand)?)
    }

    pub fn memory_read(&self, frame: usize, offset: usize) -> Result<Operand,VirtualMachineError> {
        Ok(self.memory.read(frame, offset)?)
    }

    /*
     * Operand stack subsystem
     */

    pub fn operand_push(&mut self, operand: Operand) {
        self.operand_stack.push(operand);
    }

    pub fn operand_push_label(&mut self, label: &str) -> Result<(),VirtualMachineError> {
        if let Some(address) = self.address_map.get(label) {
            self.operand_stack.push(Operand::Integer(address.clone() as i64));
            return Ok(());
        }

        Err(VirtualMachineError::InvalidLabel)
    }

    pub fn operand_pop(&mut self) -> Result<Operand,VirtualMachineError> {
        Ok(self.operand_stack.pop()?)
    }

    pub fn operand_dup(&mut self) -> Result<(),VirtualMachineError> {
        Ok(self.operand_stack.dup()?)
    }

    /*
     * Address stack 
     */

    pub fn address_push(&mut self, address: usize) {
        self.address_stack.push(address);
    }

    pub fn address_pop(&mut self) -> Result<usize,VirtualMachineError> {
        Ok(self.address_stack.pop()?)
    }

    pub fn address_label_set(&mut self, label: &str, address: usize) {
        self.address_map.insert(label.to_string(), address);
    }

    /*
     * Program subsystem
     */

    pub fn program_load(&mut self, program: Vec<Instruction>) {
        self.program = program;
    }

    pub fn program_set_entry_point(&mut self, entry_point: usize) {
        self.program_counter_set_absolute(entry_point);
    }

    /*
     * Instructions
     */
    pub fn instruction_get_current(&self) -> Result<Instruction,VirtualMachineError> {
        if let Some(instruction) = self.program.get(self.program_counter) {
            return Ok(instruction.clone());
        }

        Err(VirtualMachineError::InstructionError)
    }
}

struct _InstructionDebugInfo {
    instruction: Instruction,
    symbol: Option<String>,
    breakpoint: Option<String>,
    line_number: Option<usize>,
    scope: Option<usize>,
}