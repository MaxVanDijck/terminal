
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
use std::process::{Child, Command, Stdio};
use std::io::{Error, ErrorKind, Read, Result};
use libc::{self, c_int, TIOCSCTTY};
use std::os::unix::process::CommandExt;

struct ShellUser {
    user: String,
    home: String,
    shell: String,
}

macro_rules! die {
    ($($arg:tt)*) => {{
        std::process::exit(1);
    }}
}

fn set_controlling_terminal(fd: c_int) {
    let res = unsafe {
        // TIOSCTTY changes based on platform and the `ioctl` call is different
        // based on architecture (32/64). So a generic cast is used to make sure
        // there are no issues. To allow such a generic cast the clippy warning
        // is disabled.
        #[allow(clippy::cast_lossless)]
        libc::ioctl(fd, TIOCSCTTY as _, 0)
    };

    if res < 0 {
        die!("ioctl TIOCSCTTY failed: {}", Error::last_os_error());
    }
}

fn main() {
    // Setup Pseudoterminal
    let pty = openpty(None, None).expect("Creating pty failed");
    let master_fd = pty.controller.as_raw_fd();
    let slave_fd =  pty.user.as_raw_fd();

    if let Ok(mut termios) = termios::tcgetattr(&pty.controller) {
        // Set character encoding to UTF-8.
        termios.input_modes.set(InputModes::IUTF8, true);
        let _ = termios::tcsetattr(&pty.controller, OptionalActions::Now, &termios);
    }

    // Init shell
    // Hardcode for testing
    let user = ShellUser {user: "maxvandijck".to_string(), home: "/Users/maxvandijck".to_string(), shell: "/bin/zsh".to_string()};
    let shell_name = user.shell.rsplit('/').next().unwrap();
    let exec = format!("exec -a -{} {}", shell_name, user.shell);
    let mut binding = Command::new("/usr/bin/login");
    
    binding.args(["-flp", &user.user, "/bin/zsh", "-c", &exec]);

    binding.stdin(unsafe { Stdio::from_raw_fd(slave_fd) });
    binding.stderr(unsafe { Stdio::from_raw_fd(slave_fd) });
    binding.stdout(unsafe { Stdio::from_raw_fd(slave_fd) });

    binding.env("USER", user.user);
    binding.env("HOME", user.home);

    unsafe {binding.pre_exec(move || {
        // Create a new process group.
        let err = libc::setsid();
        if err == -1 {
            return Err(Error::new(ErrorKind::Other, "Failed to set session id"));
        }

        set_controlling_terminal(slave_fd);

        // No longer need slave/master fds.
        libc::close(slave_fd);
        libc::close(master_fd);

        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
        libc::signal(libc::SIGHUP, libc::SIG_DFL);
        libc::signal(libc::SIGINT, libc::SIG_DFL);
        libc::signal(libc::SIGQUIT, libc::SIG_DFL);
        libc::signal(libc::SIGTERM, libc::SIG_DFL);
        libc::signal(libc::SIGALRM, libc::SIG_DFL);
        Ok(())
    });
    };   

    binding.current_dir("/Users/maxvandijck".to_string());




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
