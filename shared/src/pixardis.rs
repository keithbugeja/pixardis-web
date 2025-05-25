use std::io::Write;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixardisInstruction {
    Label(String),
    PushImmediate(String),
    PushLabel(String),
    PushOffset(i64),
    PushIndexed([i64; 2]),
    Store,
    Nop,
    Drop,
    Duplicate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Increment,
    Decrement,
    Maximum,
    Minimum,
    RandomInt,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equal,
    Jump,
    ConditionalJump,
    Call,
    Return,
    Halt,
    FrameOpen,
    FrameClose,
    Allocate,
    Delay,
    Write,
    WriteBox,
    Read,
    Clear,
    Width,
    Height,
    Print,
}

pub fn pixardis_instruction_from_string(instruction: String) -> PixardisInstruction {
    
    // Let's make some preliminary processing of the instruction string
    // to remove comments and trim whitespace.
    let instruction_filtered:Vec<&str> = 
        instruction.splitn(2, "//").next().unwrap().trim().split_whitespace().collect();

    // Next we discriminate the instruction on the basis of the number of arguments.
    if instruction_filtered.len() == 1 
    {
        match instruction_filtered[0] {
            "st" => PixardisInstruction::Store,
            "nop" => PixardisInstruction::Nop,
            "drop" | "pop" => PixardisInstruction::Drop,
            "dup" => PixardisInstruction::Duplicate,
            "not" => PixardisInstruction::Not,
            "add" => PixardisInstruction::Add,
            "sub" => PixardisInstruction::Subtract,
            "mul" => PixardisInstruction::Multiply,
            "div" => PixardisInstruction::Divide,
            "inc" => PixardisInstruction::Increment,
            "dec" => PixardisInstruction::Decrement,
            "max" => PixardisInstruction::Maximum,
            "min" => PixardisInstruction::Minimum,
            "irnd" => PixardisInstruction::RandomInt,
            "lt" => PixardisInstruction::LessThan,
            "le" => PixardisInstruction::LessEqual,
            "gt" => PixardisInstruction::GreaterThan,
            "ge" => PixardisInstruction::GreaterEqual,
            "eq" => PixardisInstruction::Equal,
            "jmp" => PixardisInstruction::Jump,
            "cjmp2" => PixardisInstruction::ConditionalJump,
            "call" => PixardisInstruction::Call,
            "ret" => PixardisInstruction::Return,
            "halt" => PixardisInstruction::Halt,
            "oframe" => PixardisInstruction::FrameOpen,
            "cframe" => PixardisInstruction::FrameClose,
            "alloc" => PixardisInstruction::Allocate,
            "delay" => PixardisInstruction::Delay,
            "pixel" | "write" => PixardisInstruction::Write,
            "pixelr" | "writebox"  => PixardisInstruction::WriteBox,
            "read" => PixardisInstruction::Read,
            "clear" => PixardisInstruction::Clear,
            "width" => PixardisInstruction::Width,
            "height" => PixardisInstruction::Height,
            "print" => PixardisInstruction::Print,
            value => {
                let mut instruction = PixardisInstruction::Nop;

                let pattern = Regex::new(r"^\.(?P<label>[a-zA-Z][a-zA-Z0-9_]*)$").unwrap();
                if let Some(label) = pattern.captures(value) {
                    instruction = PixardisInstruction::Label(label["label"].to_string());
                }

                instruction
            },
        }
    } else {
        // Here we match the rest of the instructions that have arguments.
        match instruction_filtered.as_slice() {
            // Push has a number of variants:
            //
            // push n       - Push immediate value
            // push .label  - Push label
            // push #PC±n   - Push program counter +/- n
            // push [i:s]   - Push value onto scope stack
            ["push", value] => {
                let mut instruction = PixardisInstruction::Nop;
                
                let pattern = Regex::new(
                    r"^(?P<colour>#([0-9a-fA-F]{6}))|(?P<number>-?\d+(?:\.\d+)?)|\.(?P<label>[a-zA-Z][a-zA-Z0-9_]*)|(#PC(?P<offset>[+-]\d+))|(\[(?P<index>\d+):(?P<scope>\d+)\])$"            
                ).unwrap();
            
                for captures in pattern.captures_iter(value.clone().trim()) {
                    // push number
                    if let Some(num) = captures.name("number") {
                        instruction = PixardisInstruction::PushImmediate(num.as_str().to_string());
                    }
                    if let Some(num) = captures.name("colour") {
                        instruction = PixardisInstruction::PushImmediate(num.as_str().to_string());
                    }
                    // push .label
                    else if let Some(label) = captures.name("label") {
                        instruction = PixardisInstruction::PushLabel(label.as_str().to_string());
                    } 
                    // push #PC±offset
                    else if let Some(offset) = captures.name("offset") {
                        let offset_value = offset.as_str().parse::<i64>().unwrap();
                        instruction = PixardisInstruction::PushOffset(offset_value);
                    } 
                    // push [index:scope]
                    else if let (Some(num1), Some(num2)) = (captures.name("index"), captures.name("scope")) {
                        let index_value = num1.as_str().parse::<i64>().unwrap();
                        let scope_value = num2.as_str().parse::<i64>().unwrap();
                        instruction = PixardisInstruction::PushIndexed([index_value, scope_value]);
                    }
                }

                instruction
            }
            _ => PixardisInstruction::Nop,
        }
    }
}

pub fn pixardis_instruction_to_string(instruction: PixardisInstruction) -> String {
    match instruction {
        PixardisInstruction::Label(s) => format!(".{}", s),
        PixardisInstruction::PushImmediate(s) => format!("push {}", s),
        PixardisInstruction::PushLabel(s) => format!("push .{}", s),
        PixardisInstruction::PushOffset(n) => {
            if n > 0 {
                format!("push #PC+{}", n)
            } else {
                format!("push #PC{}", n)
            }
        }
        PixardisInstruction::PushIndexed([index, frame]) => format!("push [{}:{}]", index, frame),
        PixardisInstruction::Store => String::from("st"),
        PixardisInstruction::Nop => String::from("nop"),
        PixardisInstruction::Not => String::from("not"),
        PixardisInstruction::Drop => String::from("drop"),
        PixardisInstruction::Duplicate => String::from("dup"),
        PixardisInstruction::Add => String::from("add"),
        PixardisInstruction::Subtract => String::from("sub"),
        PixardisInstruction::Multiply => String::from("mul"),
        PixardisInstruction::Divide => String::from("div"),
        PixardisInstruction::Increment => String::from("inc"),
        PixardisInstruction::Decrement => String::from("dec"),
        PixardisInstruction::Maximum => String::from("max"),
        PixardisInstruction::Minimum => String::from("min"),
        PixardisInstruction::RandomInt => String::from("irnd"),
        PixardisInstruction::LessThan => String::from("lt"),
        PixardisInstruction::LessEqual => String::from("le"),
        PixardisInstruction::GreaterThan => String::from("gt"),
        PixardisInstruction::GreaterEqual => String::from("ge"),
        PixardisInstruction::Equal => String::from("eq"),
        PixardisInstruction::Jump => String::from("jmp"),
        PixardisInstruction::ConditionalJump => String::from("cjmp2"),
        PixardisInstruction::Call => String::from("call"),
        PixardisInstruction::Return => String::from("ret"),
        PixardisInstruction::Halt => String::from("halt"),
        PixardisInstruction::FrameOpen => String::from("oframe"),
        PixardisInstruction::FrameClose => String::from("cframe"),
        PixardisInstruction::Allocate => String::from("alloc"),
        PixardisInstruction::Delay => String::from("delay"),
        PixardisInstruction::Write => String::from("pixel"),
        PixardisInstruction::WriteBox => String::from("pixelr"),
        PixardisInstruction::Read => String::from("read"),
        PixardisInstruction::Clear => String::from("clear"),
        PixardisInstruction::Width => String::from("width"),
        PixardisInstruction::Height => String::from("height"),
        PixardisInstruction::Print => String::from("print"),
    }
}

pub fn pixardis_instruction_to_string_ex(instruction: PixardisInstruction, line: Option<usize>, scope: Option<usize>) -> String {
    let prefix;
        
    if line.is_some() && scope.is_some() {
        prefix = format!("{:10}", format!("[{:4}|{:4}] ", scope.unwrap(), line.unwrap()));
    } else if line.is_some() {
        prefix = format!("{:10}", format!("[{}]", line.unwrap()));
    } else if scope.is_some() {
        prefix = format!("{:10}", format!("[{}]", scope.unwrap()));
    } else {
        prefix = String::from("");
    }
         
    format!("{}{}", prefix, pixardis_instruction_to_string(instruction.clone()))
}

pub fn pixardis_save_code(code: &Vec<(usize, PixardisInstruction)>, filename: &str, show_line_numbers: bool, show_scope: bool) -> std::io::Result<()> {
    let mut file = std::fs::File::create(filename)?;
    
    let mut line = None;
    let mut scope = None;

    for (index, instruction) in code.iter().enumerate() 
    {
        if show_line_numbers == true {
            line = Some(index);
        }

        if show_scope == true {
            scope = Some(instruction.0);
        }
        
        file.write_all(
            format!("{}\n", pixardis_instruction_to_string_ex(instruction.1.clone(), line, scope)).
            as_bytes())?;
    }

    file.flush()?;

    Ok(())
}

pub fn pixardis_print_code(code: &Vec<(usize, PixardisInstruction)>, show_line_numbers: bool, show_scope: bool) {
    for (index, instruction) in code.iter().enumerate() 
    {
        let prefix;
        
        if show_line_numbers && show_scope {
            prefix = format!("{:10}", format!("[{:4}|{:4}] ", instruction.0, index));
        } else if show_line_numbers {
            prefix = format!("{:10}", format!("[{}]", index));
        } else if show_scope {
            prefix = format!("{:10}", format!("[{}]", instruction.0));
        } else {
            prefix = String::from("");
        }
             
        println!("{}{}", prefix, pixardis_instruction_to_string(instruction.1.clone()));
    }
}