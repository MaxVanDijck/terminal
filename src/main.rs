use winit::{
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    event::{Event, WindowEvent},

};

use std::os::fd::{AsRawFd, RawFd};
use std::process::Command;
use nix::unistd::ForkResult;
use nix::pty::forkpty;
use nix::unistd::read;

fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    // https://linux.die.net/man/7/pipe
    let mut read_buffer = [0; 65536];
    let read_result = read(fd, &mut read_buffer);
    match read_result {
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_e) => None,
    }
}

unsafe fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match forkpty(None, None) {
        Ok(fork_pty_res) => {
            let stdout_fd = fork_pty_res.master.as_raw_fd(); // primary
            if let ForkResult::Child = fork_pty_res.fork_result {
                // I'm the secondary part of the pty
                Command::new(&default_shell)
                    .spawn()
                    .expect("failed to spawn");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd
        }
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}

fn main() {
    let default_shell = std::env::var("SHELL")
            .expect("could not find default shell from $SHELL");
    println!("{default_shell}");

    let stdout_fd = unsafe {spawn_pty_with_shell(default_shell)};
    let mut read_buffer = vec![1];


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
                match read_from_fd(stdout_fd) {
                    Some(mut read_bytes) => {
                    read_buffer.append(&mut read_bytes);
                    println!("sum");
                }
                    None => {
                    let mut copy_buffer = vec![0; read_buffer.capacity()];
                    copy_buffer.copy_from_slice(&read_buffer);
                    println!("{:?}", String::from_utf8(copy_buffer).unwrap());
                    println!("non");
                }
                }
            },
            _ => ()
        }
    });
}
