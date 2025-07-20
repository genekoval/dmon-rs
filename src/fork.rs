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

/// The write end of a pipe to the original parent process.
///
/// The daemon can send at most one message to the parent, after which the pipe
/// will be closed and the parent process terminated. Sending nothing and
/// dropping the `Parent` object will result in an EOF on the parent's end,
/// which is equivalent to sending an empty error message. If the daemon
/// encounters a fatal error during setup, it can send a custom error message
/// to the parent with [`Parent::notify`]. This message will be printed to the
/// parent process's stderr. If setup succeeds, [`Parent::success`] should be
/// called, which sends a very specific message to the parent. The message is
/// akin to a simple "ok", so there is very little chance of an error being
/// misunderstood as success. Upon receiving a success message, the parent
/// process exits with code zero without printing anything.
///
/// Objects created by [`Parent::default`] do not contain any pipe handles. They
/// can be used to simplify code for programs that can be run as a daemon or in
/// the foreground.
///
/// # Examples
///
/// ```no_run
/// use dmon::Parent;
/// use std::process::ExitCode;
///
/// fn main() -> ExitCode {
///     let daemon = true;
///
///     let mut parent = if daemon {
///         dmon::options().daemonize()
///     } else {
///         Default::default()
///     };
///
///     match run_server(&mut parent) {
///         Ok(()) => ExitCode::SUCCESS,
///         Err(err) => {
///             eprintln!("{err}");
///             parent.notify(&err).unwrap();
///             ExitCode::FAILURE
///         }
///     }
/// }
///
/// fn run_server(parent: &mut Parent) -> Result<(), String> {
///     // Listen on a port or socket...
///     // Returning an Err here will write the error to the parent process.
///
///     parent.success().unwrap();
///
///     // Run the server...
///
///     Ok(())
/// }
/// ````
#[derive(Debug, Default)]
#[must_use = "dropping `Parent` without calling `success` indicates failure"]
pub struct Parent(Option<File>);

impl Parent {
    fn new(fd: OwnedFd) -> Self {
        Self(Some(fd.into()))
    }

    /// Returns true if the parent process is waiting for a message.
    pub fn is_waiting(&self) -> bool {
        self.0.is_some()
    }

    /// Writes the specified message to the parent process and closes the pipe.
    ///
    /// This method should be called if the daemon encounters a fatal error
    /// during setup. The message will be displayed to the user, and the
    /// parent process will exit with a non-zero code.
    ///
    /// It is safe to call this method after the pipe is closed or when there
    /// is no parent process at all. Such calls are no-ops and immediately
    /// returns [`Ok`].
    pub fn notify(&mut self, message: &str) -> io::Result<()> {
        let Some(mut pipe) = self.0.take() else {
            return Ok(());
        };

        pipe.write_all(message.as_bytes())?;

        Ok(())
    }

    /// Tells the parent process that the daemon started successfully and closes
    /// the pipe.
    ///
    /// This method should be called as soon as the daemon is considered up and
    /// running. Upon receiving a success message, the parent process will exit
    /// with code zero without printing anything to the user.
    ///
    /// It is safe to call this method after the pipe is closed or when there
    /// is no parent process at all. Such calls are no-ops and immediately
    /// returns [`Ok`].
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
