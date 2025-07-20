use crate::{
    fork::{self, Parent},
    fs, pidfile,
    user::Privileges,
};

use nix::sys::stat::{self, Mode};
use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
};

/// The default file mode creation mask value of `027`.
pub const DEFAULT_UMASK: Mode = Mode::from_bits(0o0027).unwrap();

/// Options which can be used to configure the daemon process.
///
/// This builder exposes the ability to configure various properties and enable
/// common setup actions such as setting the working directory and creating a
/// PID file.
///
/// Generally speaking, when using `Daemon`, you'll first call [`Daemon::new`],
/// then chain calls to methods to set each option before ultimately calling
/// [`Daemon::daemonize`] to fork the process and create the daemon. All code
/// after the call to [`Daemon::daemonize`] will be run inside the daemon
/// process. This method will give you a [`Parent`] that you can use to notify
/// the original process of any immediate errors you encounter while setting up.
///
/// All of the configuration methods take values wrapped in [`Option`]. This
/// is to make [`Daemon`] compatible with various external configuration
/// methods such as command-line arguments and config files. Passing [`None`]
/// to these methods resets them back to the default values. See the individual
/// methods to find out what their default values are.
///
/// Although there is no option for configuring stdin redirection, the standard
/// input stream will be redirected to `/dev/null` (any attempts to read
/// from stdin will receive an immediate EOF).
///
/// # Examples
///
/// ```no_run
/// use dmon::Daemon;
///
/// struct Config {
///     user: Option<String>,
/// }
///
/// let config = Config {
///     user: Some("daemon".into()),
/// };
///
/// let mut parent = Daemon::new()
///                   .pidfile(Some("/run/mydaemon.pid"))
///                   .working_directory(Some("/var/lib/mydaemon"))
///                   .user(config.user.as_deref())
///                   .stdout(Some("mydaemon.out"))
///                   .stderr(Some("mydaemon.err"))
///                   .daemonize();
///
/// // Perform setup like listening on a port or socket, creating a file, etc...
///
/// parent.success().unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Daemon {
    user: Option<Privileges>,
    stdout: PathBuf,
    stderr: PathBuf,
    pidfile: Option<PathBuf>,
    umask: Mode,
    workdir: PathBuf,
}

impl Default for Daemon {
    fn default() -> Self {
        Self {
            user: None,
            stdout: "/dev/null".into(),
            stderr: "/dev/null".into(),
            pidfile: None,
            umask: DEFAULT_UMASK,
            workdir: "/".into(),
        }
    }
}

impl Daemon {
    /// Creates a new daemon configuration with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the user and group the daemon will run as.
    ///
    /// If this configuration value is present, the process will drop its
    /// privileges to the given user and group. Note that this will normally
    /// require the parent process to be started as the root user.
    ///
    /// By default, no value is present and the daemon will run with the same
    /// privileges as the original process.
    pub fn user<T: Into<Privileges>>(mut self, user: Option<T>) -> Self {
        self.user = user.map(|user| user.into());
        self
    }

    /// Changes the daemon's working directory to the specified path.
    ///
    /// This is done after dropping privileges. Pass `.` to this method to avoid
    /// changing the working directory.
    ///
    /// By default, this will be the root directory (`/`).
    pub fn working_directory<P: AsRef<Path>>(
        mut self,
        workdir: Option<P>,
    ) -> Self {
        self.workdir = workdir
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or(Path::new("/"))
            .to_path_buf();

        self
    }

    /// Requests the daemon to create a PID file.
    ///
    /// The daemon will write its process ID and a trailing newline to the
    /// specified file. The file must not already exist. This file
    /// is created before dropping privileges. If a relative path is given,
    /// it will be relative to the parent's starting directory.
    ///
    /// By default, no PID file is created.
    pub fn pidfile<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.pidfile = path.map(|path| path.as_ref().to_path_buf());
        self
    }

    /// Redirects the daemon's standard output stream to the specified file.
    ///
    /// The file will be created if it does not exist and will be appended to
    /// if it does. Relative paths are relative to the daemon's
    /// working directory.
    ///
    /// By default, stdout is redirected to `/dev/null` (writes will succeed
    /// and be immediately discarded).
    pub fn stdout<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.stdout = path
            .map(|path| path.as_ref().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/dev/null"));

        self
    }

    /// Redirects the daemon's standard error stream to the specified file.
    ///
    /// The file will be created if it does not exist and will be appended to
    /// if it does. Relative paths are relative to the daemon's
    /// working directory.
    ///
    /// By default, stderr is redirected to `/dev/null` (writes will succeed
    /// and be immediately discarded).
    pub fn stderr<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.stderr = path
            .map(|path| path.as_ref().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/dev/null"));

        self
    }

    /// Sets the daemon process's file mode creation mask.
    ///
    /// See `umask(2)` for more information.
    ///
    /// The default value is [`DEFAULT_UMASK`].
    pub fn umask(mut self, mode: Option<Mode>) -> Self {
        self.umask = mode.unwrap_or(DEFAULT_UMASK);
        self
    }

    /// Applies the configuration to the daemon process.
    fn prepare(self) -> Result<(), String> {
        // Pidfiles should be owned by the root user.
        // Write the pidfile before dropping privileges.
        if let Some(pidfile) = self.pidfile {
            pidfile::create(&pidfile)?;
        }

        if let Some(user) = self.user {
            user.drop_privileges()?;
        }

        // Change the working directory after dropping privileges to ensure
        // the daemon user has access to it.
        env::set_current_dir(&self.workdir).map_err(|err| {
            format!(
                "failed to change working directory to '{}': {err}",
                self.workdir.display()
            )
        })?;

        stat::umask(self.umask);

        fs::redirect_stdin()?;
        fs::redirect_stdout(&self.stdout)?;
        fs::redirect_stderr(&self.stderr)?;

        Ok(())
    }

    /// Creates the daemon by forking the process.
    ///
    /// After the daemon process is created, the configuration is applied in
    /// the following order:
    ///
    /// 1. The PID file is created.
    /// 1. Privileges are dropped.
    /// 1. The working directory is changed.
    /// 1. The umask is set.
    /// 1. Standard input is redirected to `/dev/null`.
    /// 1. Standard output is redirected.
    /// 1. Standard error is redirected.
    ///
    /// All of the code that follows this call is run by the daemon. The
    /// original parent process waits for the child to prepare itself and is
    /// terminated by calling [`exit`] upon receiving the child's status
    /// via the returned [`Parent`] object. It is left up to the caller to
    /// decide when the daemon can be considered "ready". This could be
    /// immediately after forking or after some specific actions have been
    /// taken, such as listening on a port.
    ///
    /// # Safety
    ///
    /// This function is unsafe to call from a multithreaded environment.
    pub fn daemonize(self) -> Parent {
        let parent = fork::fork();

        if let Err(err) = self.prepare() {
            eprintln!("{err}");
            exit(1);
        }

        parent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user() {
        let daemon = Daemon::new().user(Some("root"));
        let (user, group) = daemon.user.unwrap().get().unwrap();

        assert_eq!(0, user.uid.as_raw());
        assert_eq!(0, group.gid.as_raw());
    }
}
