use crate::machine::{
    architecture::{
        Operand,
        VirtualMachine, 
        VirtualMachineError, VirtualMachineState,
    }, 
    executor::Executor
};

// use macroquad::time::get_time;
use shared::pixardis::PixardisInstruction;

use instant::Instant;

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

#[allow(dead_code)]
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
    start_time: Instant,
    #[cfg(target_arch = "wasm32")]
    print_buffer: Vec<String>,
}

impl PixardisVirtualMachine {
    pub fn new(width: usize, height: usize) -> PixardisVirtualMachine {
        PixardisVirtualMachine {
            virtual_machine: VirtualMachine::new(),
            display: PixardisDisplay::new(width, height),
            log_level: PixardisLogLevel::None,
            start_time: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            print_buffer: Vec::new(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    // Add methods to manage the print buffer
    pub fn get_print_output(&self) -> &Vec<String> {
        &self.print_buffer
    }

    #[cfg(target_arch = "wasm32")]
    pub fn clear_print_output(&mut self) {
        self.print_buffer.clear();
    }

    #[cfg(target_arch = "wasm32")]
    fn add_print_output(&mut self, text: String) {
        self.print_buffer.push(text);
    }        

    // Add the get_time function
    fn get_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
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

            /*
                PixardisInstruction::PushIndexedOffset(index) - Reads a value from a specified memory location, where the location is dynamically calculated using a base offset, an additional offset from the stack, and a frame index.

                Steps:
                1. Pop an operand from the stack to serve as the additional offset.
                2. Validate that this popped operand is an integer, which will be used to calculate the final memory location.
                3. Calculate the value offset by adding the base offset (index[0] as usize) with the additional offset popped from the stack.
                4. Read the value from memory at the calculated offset within the specified frame (index[1] as usize).
                5. Push the retrieved value onto the operand stack.

                This instruction allows for dynamic access to elements within a data structure or memory segment by calculating an element's offset at runtime. It is particularly useful for operations on arrays or complex data structures where elements are accessed based on runtime calculations or conditions. The base offset and frame are specified by the 'index' parameter, with the actual offset dynamically adjusted by an operand value.
            */

            PixardisInstruction::PushIndexedOffset(index) => {
                // frame = index[1], offset = index[0] + pop() as int;
                let operand_offset = self.virtual_machine.operand_pop()?;
                let offset = match operand_offset {
                    Operand::Integer(offset) => {
                        offset as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOffset)? },
                };

                let value_offset = (index[0] as usize) + offset;
                let value = self.virtual_machine.memory_read(index[1] as usize, value_offset)?;
                self.virtual_machine.operand_push(value);
            },

            /*
                PixardisInstruction::PushArray(index) - Pushes a sequence of values from a specified memory area onto the operand stack in reverse order.

                Steps:
                1. Pop the count operand from the stack, which indicates the number of elements to be pushed onto the stack.
                2. Validate the count operand to ensure it is a positive integer representing the number of elements to push.
                3. Iterate over the specified range in reverse order, starting from the highest offset to zero.
                4. For each offset, read a value from the specified memory location (using the base frame index and the base offset plus the current offset).
                5. Push each retrieved value onto the operand stack.

                This instruction enables the virtual machine to load elements from a specific memory region into the operand stack. The specified memory area is defined by a frame (index[1]) and a base offset (index[0]), with the count determining the number of elements to be transferred. The operation is performed in reverse sequential order, meaning the last element of the specified memory area is pushed first onto the stack, effectively reversing the original order of the elements as they are pushed onto the stack.
            */

            PixardisInstruction::PushArray(index) => {
                // LIFO to reverse sequential order
                let operand = self.virtual_machine.operand_pop()?;
                let count = match operand {
                    Operand::Integer(count) if count > 0 => {
                        count as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                for offset in (0..count).rev() {
                    let value = self.virtual_machine.memory_read(index[1] as usize, index[0] as usize + offset)?;
                    self.virtual_machine.operand_push(value);
                }
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

            /*
                PixardisInstruction::StoreArray - Stores a sequence of values into a specified memory frame and offset.

                Steps:
                1. Pop the frame operand from the stack, which specifies the memory frame where the array elements will be stored.
                2. Validate the frame operand to ensure it is an integer representing the memory frame index.
                3. Pop the offset operand from the stack, which determines the starting position in the memory frame for storing the elements.
                4. Validate the offset operand to ensure it is an integer representing the starting position within the specified memory frame.
                5. Pop the count operand from the stack, which indicates the number of elements to be stored in the array.
                6. Validate the count operand to ensure it is an integer representing the number of elements to store.
                7. Iterate over the count, popping values from the operand stack in reverse order (LIFO) and writing each value into the memory frame at the specified offset plus the current index.

                This instruction enables the virtual machine to store elements from the operand stack into a specific region of memory, defined by a frame and an offset. It treats the top of the stack as the first element of the array to be stored, following a Last-In-First-Out (LIFO) approach to retrieving elements from the stack and storing them in sequential order in memory, starting at the specified offset.
            */

            PixardisInstruction::StoreArray => {
                // LIFO to sequential order                
                // expects count, frame, offset
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

                let operand_count = self.virtual_machine.operand_pop()?;
                let count = match operand_count {
                    Operand::Integer(count) => {
                        count as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidCount)? },
                };

                for index in 0..count {
                    let value = self.virtual_machine.operand_pop()?;
                    self.virtual_machine.memory_write(frame, offset + index, value)?;
                }
            },

            PixardisInstruction::Nop => {},

            PixardisInstruction::Drop => {
                self.virtual_machine.operand_pop()?;
            },

            PixardisInstruction::Duplicate => {
                self.virtual_machine.operand_dup()?;
            },

            PixardisInstruction::DuplicateArray => {
                let operand = self.virtual_machine.operand_pop()?;
                let count = match operand {
                    Operand::Integer(count) => {
                        count as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidCount)? },
                };

                for _ in 0..count {
                    self.virtual_machine.operand_dup()?;
                }
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

            PixardisInstruction::Modulo => {
                let operand_a = self.virtual_machine.operand_pop()?;
                let operand_b = self.virtual_machine.operand_pop()?;

                let result = match (operand_a, operand_b) {
                    (Operand::Unsigned(a), Operand::Unsigned(b)) => {
                        if b == 0 {
                            Err(VirtualMachineError::DivisionByZero)?
                        }

                        Operand::Unsigned(a % b)
                    },
                    (Operand::Integer(a), Operand::Integer(b)) => {
                        if b == 0 {
                            Err(VirtualMachineError::DivisionByZero)?
                        }

                        Operand::Integer(a % b)
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

            /*
                PixardisInstruction::ReturnArray - Handles returning an array from a function.

                Steps:
                1. Pop the operand indicating the number of elements in the array to be returned.
                2. Validate the popped operand to ensure it's an integer, representing the array size.
                3. Initialize an empty vector to store the array elements temporarily.
                4. Pop each element of the array from the operand stack, filling the temporary vector.
                   The elements are popped in reverse order to match the function's return order.
                5. Close the current memory frame, cleaning up any local variables and temporary data.
                6. Pop the return address from the address stack to know where to return execution after the function call.
                7. Push the elements of the temporary array vector back onto the operand stack in reverse order.
                   This ensures the caller receives the array elements in the correct order.
                8. Set the program counter to the return address, effectively returning control to the caller.

                This instruction is pivotal for functions that return arrays, facilitating the transfer
                of array data from the callee back to the caller, ensuring proper stack and memory management,
                and maintaining the execution flow by jumping back to the correct return address.
            */

            PixardisInstruction::ReturnArray => {
                // Read number of elements to return
                let operand = self.virtual_machine.operand_pop()?;
                
                let array_size = match operand {
                    Operand::Integer(array_size) => {
                        array_size as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                // Read array from operand stack
                let mut array = Vec::<Operand>::new();
                for _ in 0..array_size {
                    let element = self.virtual_machine.operand_pop()?;
                    // array.insert(0, element);
                    array.push(element);
                }

                // Close memory stack frame
                self.virtual_machine.memory_frame_close();

                // Pop return address from address stack
                let return_address = self.virtual_machine.address_pop()?;

                // Push return values onto operand stack
                for _ in 0..array_size {
                    let element = array.pop().unwrap();
                    self.virtual_machine.operand_push(element);
                }

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

            PixardisInstruction::Delay => {
                let operand = self.virtual_machine.operand_pop()?;
                let _delay = match operand {
                    Operand::Integer(delay) => {
                        delay as u64
                    },
                    _ => { Err(VirtualMachineError::InvalidDelay)? },
                };

                self.delay(_delay)?;
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

            PixardisInstruction::WriteLine => {
                let operand = self.virtual_machine.operand_pop()?;
                let x0 = match operand {
                    Operand::Integer(x0) => {
                        x0 as usize
                    },
                    Operand::Real(x0) => {
                        x0 as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let y0 = match operand {
                    Operand::Integer(y0) => {
                        y0 as usize
                    },
                    Operand::Real(y0) => {
                        y0 as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let x1 = match operand {
                    Operand::Integer(x1) => {
                        x1 as usize
                    },
                    Operand::Real(x1) => {
                        x1 as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let operand = self.virtual_machine.operand_pop()?;
                let y1 = match operand {
                    Operand::Integer(y1) => {
                        y1 as usize
                    },
                    Operand::Real(y1) => {
                        y1 as usize
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

                let _ = self.display.write_line(x0, y0, x1, y1, c);
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
                    Operand::Integer(value) => {
                        value as u64
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
                        format!("unsigned :: {}", value.to_string())
                    },
                    Operand::Integer(value) => {
                        format!("int :: {}", value.to_string())
                    },
                    Operand::Real(value) => {
                        format!("real :: {}", value.to_string())
                    },
                };

                // For web targets, store in buffer; for native, print to console
                #[cfg(target_arch = "wasm32")]
                {
                    self.add_print_output(value);
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                {
                    println!("{}", value);
                }
            },

            /*
                PixardisInstruction::PrintArray - Prints the elements of an array from the operand stack.

                Steps:
                1. Pop the operand from the stack which indicates the size of the array to be printed.
                2. Validate the popped operand to ensure it represents an integer, which is the array size.
                3. Initialize an empty vector to collect the string representations of the array elements.
                4. Iterate over the number of elements specified by the array size:
                a. Pop each element from the operand stack.
                b. Match the type of each operand (Unsigned, Integer, Real) and format it accordingly.
                c. Push the formatted string representation of each element into the vector.
                5. Join the collected string representations with a comma separator and enclose them in brackets.
                6. Print the resulting string to the console.

                This instruction allows the virtual machine to print the contents of an array in a readable format,
                showcasing the type and value of each element. It's particularly useful for debugging purposes or
                when array content needs to be visualized during execution.
            */

            PixardisInstruction::PrintArray => {
                let operand = self.virtual_machine.operand_pop()?;
                
                let array_size = match operand {
                    Operand::Integer(array_size) => {
                        array_size as usize
                    },
                    _ => { Err(VirtualMachineError::InvalidOperand)? },
                };

                let mut values = Vec::new();

                for _ in 0..array_size {
                    let operand = self.virtual_machine.operand_pop()?;
                
                    let value = match operand {
                        Operand::Unsigned(value) => format!("unsigned :: {}", value),
                        Operand::Integer(value) => format!("int :: {}", value),
                        Operand::Real(value) => format!("real :: {}", value),
                    };
                
                    values.push(value);
                }
                
                // Print in stack order
                let output = format!("[{}]", values.join(", "));

                // For web targets, store in buffer; for native, print to console
                #[cfg(target_arch = "wasm32")]
                {
                    self.add_print_output(output);
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                {
                    println!("{}", output);
                }              
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
    
    fn run(&mut self) -> Result<(), VirtualMachineError> { 
        while self.step(1).is_ok() { };

        Ok(())
    }

    fn step(&mut self, cycles: usize) -> Result<(), VirtualMachineError> {
        // Don't change state to running when delayed
        match self.virtual_machine.state() {
            VirtualMachineState::Delayed(_, _) => { }, // if delayed, don't change state
            _ => {
                self.virtual_machine.state_set(VirtualMachineState::Running);
            }
        } 

        for _ in 0..cycles {
            // If VM is delayed, check if delay has expired
            if let VirtualMachineState::Delayed(time_stamp, cooldown) = self.virtual_machine.state() {
                let elapsed = self.get_time() - time_stamp;
                
                if elapsed < cooldown {
                    continue;
                } 
            
                self.virtual_machine.state_set(VirtualMachineState::Running);
            }
                        
            // Return current instruction
            let instruction = self.virtual_machine.instruction_get_current()?;

            // Increment program counter
            self.virtual_machine.program_counter_increment();

            // Execute instruction
            let result = self.execute_instruction(instruction.clone());
            
            // Report an error if an exception is thrown
            if result.is_err() {
                let error = result.err().unwrap();

                match self.log_level() {
                    PixardisLogLevel::None => { },
                    _ => {  
                        println!("Error: {:?}", error);
                        println!("@ ==> [{}] : {:?}", self.virtual_machine.program_counter(), instruction.clone());
                    }
                }
                
                self.virtual_machine.state_set(VirtualMachineState::Stopped);

                // For WASM targets, return the error
                #[cfg(target_arch = "wasm32")]
                {                    
                    return Err(error);
                }

                // For non-WASM targets, exit the process
                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::process::exit(1);
                }
            } 

            // Provide output if log level is set to full
            match self.log_level() {
                PixardisLogLevel::Full => {println!("[{}] : {:?}", self.virtual_machine.program_counter(), instruction.clone())},
                _ => { },
            }

            // self.virtual_machine.print_operand_stack();
        }

        // Don't change state to paused when delayed
        match self.virtual_machine.state() {
            VirtualMachineState::Delayed(_, _) => { }, // if delayed, don't change state
            _ => {
                self.virtual_machine.state_set(VirtualMachineState::Paused);
            }
        } 

        Ok(())
    }

    fn stop(&mut self) -> Result<(), VirtualMachineError> { Ok(() )}

    fn reset(&mut self) -> Result<(), VirtualMachineError> { Ok(()) }

    fn delay(&mut self, millis: u64) -> Result<(), VirtualMachineError> {
        let sleep_time = millis as f64 / 1000.0;
        let time_stamp = self.get_time();

        self.virtual_machine.state_set(VirtualMachineState::Delayed(time_stamp, sleep_time));

        Ok(())
    }
}