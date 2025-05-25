use crate::machine::{
    architecture::{
        Operand,
        VirtualMachine, 
        VirtualMachineError, VirtualMachineState,
    }, 
    executor::Executor
};

use shared::pixardis::PixardisInstruction;

#[derive(Debug, Clone)]
pub enum PixardisLogLevel {
    None,
    Error,
    Full,
}

pub struct PixardisDisplay
{
    width: usize,
    height: usize,

    display_buffer: Vec<u64>,
}

impl PixardisDisplay {
    pub fn new(width: usize, height: usize) -> PixardisDisplay {
        PixardisDisplay {
            width: width,
            height: height,

            display_buffer: vec![0; width * height],
        }
    }

    // Display width
    pub fn width(&self) -> usize {
        self.width
    }

    // Display height
    pub fn height(&self) -> usize {
        self.height
    }

    // Display framebuffer
    pub fn framebuffer(&self) -> &Vec<u64> {
        &self.display_buffer
    }

    // Clear framebuffer
    pub fn clear(&mut self, value: u64) {
        for index in 0..self.display_buffer.len() {
            self.display_buffer[index] = value;
        }
    }

    // Read pixel from framebuffer
    pub fn read_pixel(&self, x: usize, y: usize) -> Result<u64, VirtualMachineError> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            return Ok(self.display_buffer[index]);
        }

        Err(VirtualMachineError::InvalidMemoryAccess)
    }

    // Write pixel to framebuffer
    pub fn write_pixel(&mut self, x: usize, y: usize, value: u64) -> Result<(), VirtualMachineError> {
        // println!("write_pixel({}, {}, {})", x, y, value);
        
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.display_buffer[index] = value;
            
            return Ok(());
        }

        Err(VirtualMachineError::InvalidMemoryAccess)
    }

    // Draw a box on framebuffer
    pub fn write_box(&mut self, x: usize, y: usize, width: usize, height: usize, value: u64) -> Result<(), VirtualMachineError> {
        for y_offset in 0..height {
            for x_offset in 0..width {
                let x_index = x + x_offset;
                let y_index = y + y_offset;

                if x_index < self.width && y_index < self.height {
                    let index = y_index * self.width + x_index;
                    self.display_buffer[index] = value;
                }
            }
        }

        Ok(())
    }

    pub fn write_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, value: u64) -> Result<(), VirtualMachineError> {
        let dx = x1 as isize - x0 as isize;
        let dy = y1 as isize - y0 as isize;

        let mut x = x0 as isize;
        let mut y = y0 as isize;

        let mut step_x = 1;
        let mut step_y = 1;

        if dx < 0 {
            step_x = -1;
        }

        if dy < 0 {
            step_y = -1;
        }

        let dx = dx.abs();
        let dy = dy.abs();

        let mut error = dx - dy;

        loop {
            self.write_pixel(x as usize, y as usize, value)?;

            if x == x1 as isize && y == y1 as isize {
                break;
            }

            let error2 = error * 2;

            if error2 > -dy {
                error -= dy;
                x += step_x;
            }

            if error2 < dx {
                error += dx;
                y += step_y;
            }
        }

        Ok(())
    }
}

pub struct PixardisVirtualMachine
{
    virtual_machine: VirtualMachine,
    display: PixardisDisplay,
    log_level: PixardisLogLevel,
}

impl PixardisVirtualMachine {
    pub fn new(width: usize, height: usize) -> PixardisVirtualMachine {
        PixardisVirtualMachine {
            virtual_machine: VirtualMachine::new(),
            display: PixardisDisplay::new(width, height),
            log_level: PixardisLogLevel::None,
        }
    }

    //
    // Convert a string to an operand
    //
    fn operand_from_string(&self, operand: &str) -> Operand {
        // Operand is a real number
        if operand.contains(".") {
            Operand::Real(operand.parse::<f64>().unwrap())
        } else {
            // Operand is a hex colour
            if operand.starts_with("#") && operand.len() == 7 {
                let hex_digits = &operand[1..]; // Remove the '#' character
                let rgb_value = u64::from_str_radix(hex_digits, 16);
            
                match rgb_value {
                    Ok(value) => Operand::Unsigned(value),
                    Err(_) => Operand::Unsigned(0xFF00FF)        // Push false colour (error)
                }
            } else {
                // Operand is an integer
                Operand::Integer(operand.parse::<i64>().unwrap())
            }
        }
    }

    //
    // Load program from source (text)
    //
    pub fn load_program_from_source(&mut self, source: &str) {
        // Split the string using newlines (\n)
        let source_lines: Vec<&str> = source.split('\n').collect();

        let mut pixardis_program = Vec::<PixardisInstruction>::new();

        for line in source_lines {
            let instruction = shared::pixardis::pixardis_instruction_from_string(line.to_string());
            match instruction.clone() {
                PixardisInstruction::Label(label) => {
                    let current_instruction_index = pixardis_program.len();

                    self.virtual_machine.address_label_set(&label, current_instruction_index);
                    
                    if label == ".main" {
                        self.virtual_machine.program_set_entry_point(current_instruction_index);
                    }
                },
                _ => { },
            }

            pixardis_program.push(instruction);
        }

        // Load program into virtual machine
        self.virtual_machine.program_load(pixardis_program);
    }

    //
    // Execute a single instruction
    //
    pub fn execute_instruction(&mut self, instruction: PixardisInstruction) -> Result<(), VirtualMachineError> {
        match instruction.clone() {
            PixardisInstruction::Label(_) => { },

            PixardisInstruction::PushImmediate(value) => { 
                let operand = self.operand_from_string(value.as_str());
                self.virtual_machine.operand_push(operand); 
            },

            PixardisInstruction::PushLabel(label) => {
                self.virtual_machine.operand_push_label(label.as_str())?;            
            },

            PixardisInstruction::PushOffset(offset) => {
                let address = self.virtual_machine.program_counter() as i64 + offset - 1;
                self.virtual_machine.operand_push(Operand::Integer(address));
            },

            PixardisInstruction::PushIndexed(index) => {
                // frame = index[1], offset = index[0];
                let value = self.virtual_machine.memory_read(index[1] as usize, index[0] as usize)?;
                self.virtual_machine.operand_push(value);
            },

            PixardisInstruction::Store => {
                let operand_frame = self.virtual_machine.operand_pop()?;
                let frame = match operand_frame {
                    Operand::Integer(frame) => {
                        frame as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidFrame)? },
                };

                let operand_offset = self.virtual_machine.operand_pop()?;
                let offset = match operand_offset {
                    Operand::Integer(offset) => {
                        offset as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOffset)? },
                };

                let value = self.virtual_machine.operand_pop()?;

                self.virtual_machine.memory_write(frame, offset, value)?;
            },

            PixardisInstruction::Nop => {},

            PixardisInstruction::Drop => {
                self.virtual_machine.operand_pop()?;
            },

            PixardisInstruction::Duplicate => {
                self.virtual_machine.operand_dup()?;
            },

            PixardisInstruction::Not => {
                let operand = self.virtual_machine.operand_pop()?;
                let result = match operand {
                    Operand::Unsigned(value) => {
                        Operand::Unsigned(!value)
                    },
                    Operand::Integer(value) => {
                        Operand::Integer(!value)
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Add => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a.clone(), operand_b.clone()) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(a + b)
                    },                    
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(a + b)
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Real(a + b)
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Real(a + b as f64)
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Real(a as f64 + b)
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Subtract => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(a - b)
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(a - b)
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Real(a - b)
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Real(a - b as f64)
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Real(a as f64 - b)
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Multiply => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(a * b)
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(a * b)
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Real(a * b)
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Real(a * b as f64)
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Real(a as f64 * b)
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Divide => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        if b == 0 {
                            Err(VirtualMachineError::DivisionByZero)?
                        }

                        Operand::Unsigned(a / b)
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        if b == 0 {
                            Err(VirtualMachineError::DivisionByZero)?
                        }

                        Operand::Integer(a / b)
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        if b.abs() < f64::EPSILON {
                            Err(VirtualMachineError::DivisionByZero)?
                        }
                        
                        Operand::Real(a / b)
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        if b == 0 {
                            Err(VirtualMachineError::DivisionByZero)?
                        }

                        Operand::Real(a / b as f64)
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        if b.abs() < f64::EPSILON {
                            Err(VirtualMachineError::DivisionByZero)?
                        }
                        
                        Operand::Real(a as f64 / b)
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Increment => {
                let operand = self.virtual_machine.operand_pop()?;
                let result = match operand {
                    Operand::Unsigned(value) => {
                        Operand::Unsigned(value + 1)
                    },
                    Operand::Integer(value) => {
                        Operand::Integer(value + 1)
                    },
                    Operand::Real(value) => {
                        Operand::Real(value + 1.0)
                    },                    
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Decrement => {
                let operand = self.virtual_machine.operand_pop()?;
                let result = match operand {
                    Operand::Unsigned(value) => {
                        Operand::Unsigned(value - 1)
                    },
                    Operand::Integer(value) => {
                        Operand::Integer(value - 1)
                    },
                    Operand::Real(value) => {
                        Operand::Real(value - 1.0)
                    },                    
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Maximum => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(a.max(b))
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(a.max(b))
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Real(a.max(b))
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Real(a.max(b as f64))
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Real((a as f64).max(b))
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Minimum => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(a.min(b))
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(a.min(b))
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Real(a.min(b))
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Real(a.min(b as f64))
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Real((a as f64).min(b))
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::RandomInt => {
                let operand = self.virtual_machine.operand_pop()?;

                let result = match operand {
                    Operand::Integer(upper) => {
                        let value = self.virtual_machine.random_integer(upper);

                        Operand::Integer(value)
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::LessThan => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(if a < b { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(if a < b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Integer(if a < b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Integer(if a < b as f64 { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Integer(if (a as f64) < b { 1 } else { 0 })
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::LessEqual => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(if a <= b { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(if a <= b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Integer(if a <= b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Integer(if a <= b as f64 { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Integer(if (a as f64) <= b { 1 } else { 0 })
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::GreaterThan => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(if a > b { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(if a > b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Integer(if a > b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Integer(if a > b as f64 { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Integer(if (a as f64) > b { 1 } else { 0 })
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::GreaterEqual => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(if a >= b { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(if a >= b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Integer(if a >= b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Integer(if a >= b as f64 { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Integer(if (a as f64) >= b { 1 } else { 0 })
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Equal => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        Operand::Unsigned(if a == b { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        Operand::Integer(if a == b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Real(b)) => {
                        Operand::Integer(if a == b { 1 } else { 0 })
                    },
                    (Operand::Real(a), Operand::Integer(b)) => {
                        Operand::Integer(if a == b as f64 { 1 } else { 0 })
                    },
                    (Operand::Integer(a), Operand::Real(b)) => {
                        Operand::Integer(if (a as f64) == b { 1 } else { 0 })
                    },
                    (_, _) => { Err(VirtualMachineError::InvalidOperand)? },
                };
                self.virtual_machine.operand_push(result);
            },

            PixardisInstruction::Jump => {
                let operand = self.virtual_machine.operand_pop()?;

                let address = match operand {
                    Operand::Integer(address) => {
                        address as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidAddress)? },
                };

                self.virtual_machine.program_counter_set_absolute(address);
            },

            PixardisInstruction::ConditionalJump => {
                let operand = self.virtual_machine.operand_pop()?;
                let address = match operand {
                    Operand::Integer(address) => {
                        address as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidAddress)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let condition = match operand {
                    Operand::Unsigned(condition) => {
                        condition as i64
                    },
                    Operand::Integer(condition) => {
                        condition
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                if condition != 0 {
                    self.virtual_machine.program_counter_set_absolute(address);
                }
            },

            PixardisInstruction::Call => {
                // Read address of subroutine
                let operand = self.virtual_machine.operand_pop()?;
                let address = match operand {
                    Operand::Integer(address) => {
                        address as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidAddress)? },
                };

                // Read number of arguments
                let operand = self.virtual_machine.operand_pop()?;
                let param_count = match operand {
                    Operand::Integer(param_count) => {
                        param_count as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidArgumentCount)? },
                };

                // Read arguments from operand stack
                let mut param_copy = param_count.clone();
                let mut param_buffer = Vec::<Operand>::new();

                while param_copy > 0 {
                    let operand = self.virtual_machine.operand_pop()?;
                    param_buffer.push(operand);
                    param_copy -= 1;
                }

                // Open a new memory stack frame
                self.virtual_machine.memory_frame_open(param_count);

                // Copy arguments
                for (index, operand) in param_buffer.iter().enumerate() {
                    self.virtual_machine.memory_write(0, index, operand.clone())?;
                }

                // Push return address onto address stack
                self.virtual_machine.address_push(self.virtual_machine.program_counter() as usize);

                // Jump to subroutine
                self.virtual_machine.program_counter_set_absolute(address);
            },

            PixardisInstruction::Return => {
                // Read return value
                let operand = self.virtual_machine.operand_pop()?;

                // Close memory stack frame
                self.virtual_machine.memory_frame_close();

                // Pop return address from address stack
                let return_address = self.virtual_machine.address_pop()?;

                // Push return value onto operand stack
                self.virtual_machine.operand_push(operand);

                // Jump to return address
                self.virtual_machine.program_counter_set_absolute(return_address);
            },

            PixardisInstruction::Halt => { 
                Err(VirtualMachineError::TrapHalt)? 
            },

            PixardisInstruction::FrameOpen => {
                let operand = self.virtual_machine.operand_pop()?;
                let frame_size = match operand {
                    Operand::Integer(frame_size) => {
                        frame_size as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidFrameSize)? },
                };

                self.virtual_machine.memory_frame_open(frame_size);
            },

            PixardisInstruction::FrameClose => {
                self.virtual_machine.memory_frame_close();
            },

            PixardisInstruction::Allocate => {
                let operand = self.virtual_machine.operand_pop()?;
                let frame_size = match operand {
                    Operand::Integer(frame_size) => {
                        frame_size as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidFrameSize)? },
                };

                let _ = self.virtual_machine.memory_frame_alloc(frame_size);
            },

            // TODO: Implement
            PixardisInstruction::Delay => {
                let operand = self.virtual_machine.operand_pop()?;
                let _delay = match operand {
                    Operand::Integer(delay) => {
                        delay as u64
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                // self.virtual_machine.delay(delay);
            },

            PixardisInstruction::Write => {
                let operand = self.virtual_machine.operand_pop()?;
                let x = match operand {
                    Operand::Integer(x) => {
                        x as usize
                    },
                    Operand::Real(x) => {
                        x as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let y = match operand {
                    Operand::Integer(y) => {
                        y as usize
                    },
                    Operand::Real(y) => {
                        y as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let c = match operand {
                    Operand::Unsigned(c) => {
                        c as u64
                    },
                    Operand::Integer(c) => {
                        c as u64
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let _ = self.display.write_pixel(x, y, c);
            },

            PixardisInstruction::WriteBox => {
                let operand = self.virtual_machine.operand_pop()?;
                let x = match operand {
                    Operand::Integer(x) => {
                        x as usize
                    },
                    Operand::Real(x) => {
                        x as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let y = match operand {
                    Operand::Integer(y) => {
                        y as usize
                    },
                    Operand::Real(y) => {
                        y as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let w = match operand {
                    Operand::Integer(w) => {
                        w as usize
                    },
                    Operand::Real(w) => {
                        w as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let h = match operand {
                    Operand::Integer(h) => {
                        h as usize
                    },
                    Operand::Real(h) => {
                        h as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let c = match operand {
                    Operand::Unsigned(c) => {
                        c as u64
                    },
                    Operand::Integer(c) => {
                        c as u64
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let _ = self.display.write_box(x, y, w, h, c);
            },

            PixardisInstruction::Read => {
                let operand = self.virtual_machine.operand_pop()?;
                let x = match operand {
                    Operand::Integer(x) => {
                        x as usize
                    },
                    Operand::Real(x) => {
                        x as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let y = match operand {
                    Operand::Integer(y) => {
                        y as usize
                    },
                    Operand::Real(y) => {
                        y as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let value = self.display.read_pixel(x, y)?;
                self.virtual_machine.operand_push(Operand::Unsigned(value));
            },

            PixardisInstruction::Clear => {
                let operand = self.virtual_machine.operand_pop()?;
                let value = match operand {
                    Operand::Unsigned(value) => {
                        value
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                self.display.clear(value);
            },

            PixardisInstruction::Width => {
                self.virtual_machine.operand_push(Operand::Integer(self.display.width() as i64));
            },

            PixardisInstruction::Height => {
                self.virtual_machine.operand_push(Operand::Integer(self.display.height() as i64));
            },
            
            PixardisInstruction::Print => {
                let operand = self.virtual_machine.operand_pop()?;
                let value = match operand {
                    Operand::Unsigned(value) => {
                        format!("Ux{}", value.to_string())
                    },
                    Operand::Integer(value) => {
                        format!("Ix{}", value.to_string())
                    },
                    Operand::Real(value) => {
                        format!("Rx{}", value.to_string())
                    },
                };

                println!("{}", value);
            },

            // Just in case we get an instruction we don't recognise
            // _ => { },
        }

        Ok(())
    }

    //
    // Returns the display framebuffer
    //
    pub fn framebuffer(&self) -> (usize, usize, &Vec<u64>) {
        (self.display.width(), self.display.height(), self.display.framebuffer())
    }

    //
    // Set VM log level
    //
    pub fn log_level_set(&mut self, log_level: PixardisLogLevel) {
        self.log_level = log_level;
    }

    pub fn log_level(&self) -> PixardisLogLevel {
        self.log_level.clone()
    }
}

///
/// Implements the Executor trait for the Pixardis virtual machine
///
impl Executor for PixardisVirtualMachine {    
    
    fn run(&mut self) -> Result<(), VirtualMachineError> 
    { 
        while self.step(1).is_ok() { };

        Ok(())
    }

    fn step(&mut self, cycles: usize) -> Result<(), VirtualMachineError> 
    {
        self.virtual_machine.state_set(VirtualMachineState::Running);

        for _ in 0..cycles {
            // Return current instruction
            let instruction = self.virtual_machine.instruction_get_current()?;

            // Increment program counter
            self.virtual_machine.program_counter_increment();

            // Execute instruction
            let result = self.execute_instruction(instruction.clone());
            
            // Report an error if an exception is thrown
            if result.is_err() {
                match self.log_level() {
                    PixardisLogLevel::None => { },
                    _ => {  
                        println!("Error: {:?}", result.err().unwrap());
                        println!("@ ==> [{}] : {:?}", self.virtual_machine.program_counter(), instruction.clone());
                    }
                }
                
                self.virtual_machine.state_set(VirtualMachineState::Stopped);

                std::process::exit(1);
            } else {
                match self.log_level() {
                    PixardisLogLevel::Full => {println!("[{}] : {:?}", self.virtual_machine.program_counter(), instruction.clone())},
                    _ => { },
                }
            }
        }

        self.virtual_machine.state_set(VirtualMachineState::Paused);

        Ok(())
    }

    fn stop(&mut self) -> Result<(), VirtualMachineError> { Ok(() )}

    fn reset(&mut self) -> Result<(), VirtualMachineError> { Ok(()) }
}