use nix::unistd::{self, ForkResult, setsid};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::fd::OwnedFd,
    process::exit,
};

const SUCCESS: &str = "OK";

struct Pipe {
    read: OwnedFd,
    write: OwnedFd,
}

impl Pipe {
    fn new() -> Self {
        match unistd::pipe() {
            Ok((read, write)) => Self { read, write },
            Err(err) => {
                eprintln!("failed to create interprocess channel: {err}");
                exit(1);
            }
        }
    }

    fn read(self) -> OwnedFd {
        self.read
    }

    fn write(self) -> OwnedFd {
        self.write
    }
}

#[derive(Default)]
#[must_use = "dropping `Parent` without calling `success` indicates failure"]
pub struct Parent(Option<File>);

impl Parent {
    fn new(fd: OwnedFd) -> Self {
        Self(Some(fd.into()))
    }

    pub fn is_waiting(&self) -> bool {
        self.0.is_some()
    }

    pub fn notify(&mut self, message: &str) -> io::Result<()> {
        let Some(mut pipe) = self.0.take() else {
            return Ok(());
        };

        pipe.write_all(message.as_bytes())?;

        Ok(())
    }

    pub fn success(&mut self) -> io::Result<()> {
        self.notify(SUCCESS)
    }
}

struct Child(File);

impl Child {
    fn read(mut self) -> String {
        let mut message = String::new();

        if let Err(err) = self.0.read_to_string(&mut message) {
            eprintln!("failed to read message from daemon process: {err}");
            exit(1);
        }

        message
    }

    fn wait(self) -> ! {
        match self.read().as_str() {
            SUCCESS => exit(0),
            "" => eprintln!("daemon failed to start"),
            message => eprintln!("daemon failed to start: {message}"),
        }

        exit(1);
    }
}

impl From<OwnedFd> for Child {
    fn from(fd: OwnedFd) -> Self {
        Self(fd.into())
    }
}

fn parent(pipe: Pipe) -> ! {
    Child::from(pipe.read()).wait();
}

fn child(pipe: Pipe) -> Parent {
    let pipe = pipe.write();

    if setsid().is_err() {
        eprintln!("already process group leader");
        exit(1);
    }

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { .. }) => exit(0),
        Ok(ForkResult::Child) => Parent::new(pipe),
        Err(err) => {
            eprintln!("failed to fork off for the second time: {err}");
            exit(1);
        }
    }
}

pub fn fork() -> Parent {
    let pipe = Pipe::new();

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { .. }) => parent(pipe),
        Ok(ForkResult::Child) => child(pipe),
        Err(err) => {
            eprintln!("failed to fork off for the first time: {err}");
            exit(1);
        }
    }
}
