use clap::Parser;
use dmon::{
    nix::{
        sys::stat::Mode,
        unistd::{chown, mkfifo},
    },
    user::Privileges,
};
use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind},
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Starts a server that listens on a named pipe for commands
///
/// The named pipe will appear in the process's working directory under the
/// name "daemon.pipe". You can communicate with the server by passing
/// commands into the pipe. Every line sent is a separate command.
///
/// Command output is written to the file "daemon.out". Errors and diagnostic
/// information is written to "daemon.err". Both files are located in the
/// working directory.
#[derive(Debug, Parser)]
#[command(max_term_width = 80)]
struct Cli {
    /// Run the server as a daemon process
    #[arg(short, long)]
    daemon: bool,

    /// Daemon process owner and optional group
    #[arg(short, long, value_name = "OWNER:[GROUP]", requires = "daemon")]
    user: Option<String>,

    /// Daemon working directory
    #[arg(
        short = 'w',
        long,
        value_name = "DIRECTORY",
        default_value = Self::default_work_dir().into_os_string(),
        requires = "daemon",
    )]
    workdir: PathBuf,

    /// Path to the pidfile
    #[arg(short, long, value_name = "FILE", requires = "daemon")]
    pidfile: Option<PathBuf>,
}

impl Cli {
    fn default_work_dir() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(env!("CARGO_PKG_NAME"));
        path
    }
}

// A Parent wrapper that prints errors to stderr.
#[derive(Debug, Default)]
struct Parent(dmon::Parent);

impl Parent {
    fn notify(&mut self, message: &str) {
        if let Err(err) = self.0.notify(message) {
            eprintln!("failed to send error message to parent process: {err}");
        }
    }

    fn success(&mut self) {
        if let Err(err) = self.0.success() {
            eprintln!(
                "failed to notify parent process of successful start: {err}"
            );
        }
    }
}

impl From<dmon::Parent> for Parent {
    fn from(value: dmon::Parent) -> Self {
        Self(value)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let user = cli.user.as_deref().map(Privileges::from);
    let work_dir = cli.workdir.as_path();
    let pidfile = cli.pidfile.as_deref();

    let mut parent: Parent = if cli.daemon {
        if let Err(err) = create_dir(work_dir, user.as_ref()) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }

        dmon::options()
            .user(user)
            .pidfile(pidfile)
            .working_directory(Some(work_dir))
            .stdout(Some("daemon.out"))
            .stderr(Some("daemon.err"))
            .daemonize()
            .into()
    } else {
        Default::default()
    };

    eprintln!("server process started");

    let code = match run_server(&mut parent) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            parent.notify(&err);
            ExitCode::FAILURE
        }
    };

    if let Some(pidfile) = pidfile {
        remove_file(pidfile);
    }

    code
}

fn run_server(parent: &mut Parent) -> Result<(), String> {
    const FIFO: &str = "daemon.pipe";

    mkfifo(FIFO, Mode::S_IRWXU).map_err(|err| {
        format!("failed to create named pipe '{FIFO}': {err}")
    })?;

    parent.success();

    let fifo = File::open(FIFO)
        .map_err(|err| format!("failed to open '{FIFO}' for reading: {err}"))?;

    let mut fifo = BufReader::new(fifo);
    let mut line = String::new();

    loop {
        line.clear();

        let line = match fifo.read_line(&mut line) {
            Ok(_) => line.trim(),
            Err(err) => {
                eprintln!("failed to read from pipe: {err}");
                continue;
            }
        };

        let mut split = line.splitn(2, ' ');

        let command = split.next().unwrap().trim();
        let args = split.next().unwrap_or_default().trim();

        match command {
            "print" => println!("{args}"),
            "quit" => break,
            "" => (),
            _ => eprintln!("unknown command: {command}"),
        }
    }

    eprintln!("server process shutting down");
    drop(fifo);
    remove_file(FIFO);

    Ok(())
}

fn create_dir(path: &Path, user: Option<&Privileges>) -> Result<(), String> {
    match fs::create_dir(path) {
        Ok(()) => {
            if let Some(user) = user {
                let (user, group) = user.get()?;

                chown(path, Some(user.uid), Some(group.gid)).map_err(|err| {
                    format!(
                        "failed to change ownership of directory '{}': {err}",
                        path.display()
                    )
                })?;
            }

            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(format!(
            "failed to created directory '{}': {err}",
            path.display()
        )),
    }
}

fn remove_file<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    if let Err(err) = fs::remove_file(path)
        && err.kind() != ErrorKind::NotFound
    {
        eprintln!("failed to remove file '{}': {err}", path.display());
    }
}
