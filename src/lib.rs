mod daemon;
mod fork;
mod fs;
mod pidfile;
mod user;

pub use daemon::{DEFAULT_UMASK, Daemon};
pub use fork::Parent;
pub use user::{Group, Privileges, User};

pub use nix::sys::stat::Mode;

pub fn options() -> Daemon {
    Daemon::new()
}
