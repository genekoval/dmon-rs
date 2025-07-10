mod daemon;
mod fork;
mod fs;
mod pidfile;
mod user;

pub use daemon::Daemon;
pub use fork::Parent;
pub use user::{Group, Privileges, User};

pub fn options() -> Daemon {
    Daemon::new()
}
