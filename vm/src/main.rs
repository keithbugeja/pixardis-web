mod pixardis;
mod machine;

use std::io;

use macroquad::prelude::*;

#[macroquad::main("Chroma VM (Pixardis Emulator)")]

async fn main() -> Result<(), io::Error> 
{
    // Parse command line arguments; place the results in a context object.
    let context = process_cmd_args();

    // Initialise VM
    let mut vm = PixardisVirtualMachine::new(context.width.unwrap(), context.height.unwrap());

    // Get desired log level
    let log_level = match context.log_level {
        Some(1) => PixardisLogLevel::Error,
        Some(2) => PixardisLogLevel::Full,
        _ => PixardisLogLevel::None,
    };

    // Set log level
    vm.log_level_set(log_level);

    // Get the file path from the context object
    let file_path = context.input.as_str();
    
    // Read source file
    let source = shared::io::read_file_to_string(&file_path)?;
    
    // Load program from source (text)
    vm.load_program_from_source(&source);
    
    loop {
        // Start execution when S is pressed
        if is_key_down(KeyCode::S) {
            break;
        }

        clear_background(WHITE);

        let text = "Hit [s] to execute loaded program.";
        let font_size = 30.;
        let text_size = measure_text(text, None, font_size as _, 1.0);

        draw_text(
            text,
            screen_width() / 2. - text_size.width / 2.,
            screen_height() / 2. + text_size.height / 2.,
            font_size,
            DARKGRAY,
        );

        next_frame().await
    }

    loop {
        // Quit when Q is pressed
        if is_key_down(KeyCode::Q) {
            break;
        }

        // Run for a given number of cycles
        let _ = vm.step(context.cycles.unwrap() as usize);

        // Draw the VM framebuffer
        let (width, height, colours) = vm.framebuffer();

        // Determine cell size (from screen size and framebuffer dimensions)
        let cell_edge_size = (screen_width() / width as f32).min(screen_height() / height as f32);
        
        for y in 0..height {
            for x in 0..width {
                let colour = colours[y * width + x];

                draw_rectangle(
                    x as f32 * cell_edge_size,
                    (height - y - 1) as f32 * cell_edge_size,
                    cell_edge_size,
                    cell_edge_size,
                    Color::from_hex(colour as u32),
                );
            }
        }

        next_frame().await
    }
    
    Ok(())
}

use clap::Parser as ClapParser;
use machine::executor::Executor;
use pixardis::pixardis::{PixardisVirtualMachine, PixardisLogLevel};

#[derive(clap::Parser, Debug)]
#[command(name = "chroma-vm")]
#[command(author = "Keith <bugeja.keith@gmail.com>")]
#[command(version = "0.1")]
#[command(about = "An emulator for the Pixardis (Pixel Art Display) VM.")]
#[command(long_about = 
"------------------------------------------------------------
     ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ███╗ █████╗ 
    ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗ ████║██╔══██╗
    ██║     ███████║██████╔╝██║   ██║██╔████╔██║███████║
    ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╔╝██║██╔══██║
    ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║
     ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝
                                                        
                      Virtual Machine
------------------------------------------------------------")]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    input: String,

    #[arg(short, long, help = "VM instruction cycles per frame [default = 250].", default_value = "250")]
    cycles: Option<u32>,

    #[arg(short = 'x', long, help = "VM display width [default = 64].", default_value = "64")]
    width: Option<usize>,

    #[arg(short = 'y', long, help = "VM display height [default = 48].", default_value = "48")]
    height: Option<usize>,

    #[arg(short = 'L', help = "Log level [default = 0].", default_value = "0")]
    log_level: Option<usize>,

    //#[arg(short, long, help = "Run VM in debug mode.")]
    //debug: Option<bool>,

    //#[arg(short, long, help = "Run VM in pure mode, without extensions.")]
    //pure: Option<bool>,
}

//
// Process compiler command line arguments
//
fn process_cmd_args() -> Args 
{
    let args = Args::parse();

    args
}