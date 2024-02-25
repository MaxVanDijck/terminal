
use winit::{
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    event::{Event, WindowEvent},

};

use rustix_openpty::openpty;
use rustix::fd::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use rustix_openpty::rustix::termios;
use rustix_openpty::rustix::termios::{InputModes, OptionalActions};

use std::process::Command;

fn main() {
    let default_shell = std::env::var("SHELL")
            .expect("could not find default shell from $SHELL");
    println!("{default_shell}");
    let pty = openpty(None, None).expect("Creating pty failed");
    let master_fd = pty.controller.as_raw_fd();
    let slave_fd =  pty.user.as_raw_fd();

    if let Ok(mut termios) = termios::tcgetattr(&pty.controller) {
        // Set character encoding to UTF-8.
        termios.input_modes.set(InputModes::IUTF8, true);
        let _ = termios::tcsetattr(&pty.controller, OptionalActions::Now, &termios);
    }

    let cmd = Command::new(default_shell).spawn().expect("failed to spawn");
    let cmd = Command::new("ls").spawn().expect("failed to spawn");
    let cmd = Command::new("rg").spawn().expect("failed to spawn");




    let mut master_file = unsafe { File::from_raw_fd(slave_fd) };
    master_file.flush().expect("Failed to flush the command to pty");

    let master_file = unsafe { File::from_raw_fd(master_fd) };
    let reader = BufReader::new(master_file);
    for line in reader.lines() {
        match line {
            Ok(line) => println!("Read line: {}", line),
            Err(e) => println!("Error reading line: {}", e),
        }
    }




    // WINDOW STUFF
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().with_title("Terminal").build(&event_loop).unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let _ = event_loop.run(move |event, elwt| {
    match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            },
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. } ,
                ..
            } => {
                println!("{event:?}");
            },
            _ => ()
        }
    });
}
