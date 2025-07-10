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

#[derive(Debug)]
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
            umask: Mode::from_bits(0o0027).unwrap(),
            workdir: "/".into(),
        }
    }
}

impl Daemon {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn user<T: Into<Privileges>>(mut self, user: Option<T>) -> Self {
        self.user = user.map(|user| user.into());
        self
    }

    pub fn change_dir<P: AsRef<Path>>(mut self, workdir: Option<P>) -> Self {
        self.workdir = workdir
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or(Path::new("/"))
            .to_path_buf();

        self
    }

    pub fn pidfile<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.pidfile = path.map(|path| path.as_ref().to_path_buf());
        self
    }

    pub fn stderr<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.stderr = path
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or(Path::new("/dev/null"))
            .to_path_buf();

        self
    }

    pub fn stdout<P: AsRef<Path>>(mut self, path: Option<P>) -> Self {
        self.stdout = path
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or(Path::new("/dev/null"))
            .to_path_buf();

        self
    }

    pub fn umask(mut self, mode: Option<Mode>) -> Self {
        if let Some(mode) = mode {
            self.umask = mode;
        }

        self
    }

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

    #[must_use]
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
