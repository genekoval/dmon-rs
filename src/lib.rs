mod fork;
mod fs;
mod pidfile;
mod user;

pub use fork::Parent;
pub use user::{Group, User};

use nix::{
    sys::stat::{self, Mode},
    unistd::{Gid, Uid},
};
use std::{env, path::Path, process::exit};

#[derive(Debug)]
pub struct Daemon<'a> {
    user: Option<User>,
    group: Option<Group>,
    stdout: &'a Path,
    stderr: &'a Path,
    pidfile: Option<&'a Path>,
    umask: Mode,
    workdir: &'a Path,
}

impl<'a> Default for Daemon<'a> {
    fn default() -> Self {
        Self {
            user: None,
            group: None,
            stdout: fs::null(),
            stderr: fs::null(),
            pidfile: None,
            umask: Mode::from_bits(0o0027).unwrap(),
            workdir: fs::root(),
        }
    }
}

impl<'a> Daemon<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn chdir(mut self, workdir: Option<&'a Path>) -> Self {
        if let Some(workdir) = workdir {
            self.workdir = workdir;
        }

        self
    }

    pub fn group(mut self, group: &Option<Group>) -> Self {
        self.group = group.clone();
        self
    }

    pub fn permissions(mut self, perms: Option<&str>) -> Self {
        if let Some(perms) = perms {
            let mut perms = perms.trim().split(':');

            let user = perms.next().unwrap();

            self.user = match user.parse::<u32>().ok() {
                Some(uid) => Some(User::Id(Uid::from_raw(uid))),
                None => Some(User::Name(user.into())),
            };

            if let Some(group) = perms.next() {
                self.group = match group.parse::<u32>().ok() {
                    Some(gid) => Some(Group::Id(Gid::from_raw(gid))),
                    None => Some(Group::Name(group.into())),
                };
            }
        }

        self
    }

    pub fn pidfile(mut self, pidfile: Option<&'a Path>) -> Self {
        self.pidfile = pidfile;
        self
    }

    pub fn stderr(mut self, path: Option<&'a Path>) -> Self {
        if let Some(path) = path {
            self.stderr = path;
        }

        self
    }

    pub fn stdout(mut self, path: Option<&'a Path>) -> Self {
        if let Some(path) = path {
            self.stdout = path;
        }

        self
    }

    pub fn umask(mut self, mode: Option<Mode>) -> Self {
        if let Some(mode) = mode {
            self.umask = mode;
        }

        self
    }

    pub fn user(mut self, user: &Option<User>) -> Self {
        self.user = user.clone();
        self
    }

    fn prepare(self) -> Result<(), String> {
        // Pidfiles should be owned by the root user.
        // Write the pidfile before dropping privileges.
        if let Some(pidfile) = self.pidfile {
            pidfile::create(pidfile)?;
        }

        if let Some(user) = &self.user {
            user::drop_privileges(user, self.group.as_ref())?;
        }

        // Change the working directory after dropping privileges to ensure
        // the daemon user has access to it.
        env::set_current_dir(self.workdir).map_err(|err| {
            format!(
                "Failed to change working directory to '{}': {err}",
                self.workdir.display()
            )
        })?;

        stat::umask(self.umask);

        fs::redirect_stdin()?;
        fs::redirect_stdout(self.stdout)?;
        fs::redirect_stderr(self.stderr)?;

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

pub fn options() -> Daemon<'static> {
    Daemon::new()
}
