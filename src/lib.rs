//! A library for building daemon processes.
//!
//! # Example
//!
//! ```no_run
//! use dmon::nix::sys::stat::Mode;
//!
//! dmon::options()
//!     .user(Some("mydaemon".parse().unwrap()))
//!     .working_directory(Some("/var/lib/mydaemon"))
//!     .pidfile(Some("/run/mydaemon.pid"))
//!     .stdout(Some("mydaemon.out"))
//!     .stderr(Some("mydaemon.err"))
//!     .umask(Some(Mode::from_bits(0o0077).unwrap()))
//!     .daemonize()
//!     .success()
//!     .unwrap();
//! ```

pub mod user;

mod daemon;
mod fork;
mod fs;
mod pidfile;

pub use daemon::{DEFAULT_UMASK, Daemon};
pub use fork::Parent;

pub use nix;

/// Returns a new Daemon object.
///
/// This function is equivalent to [`Daemon::new`] but avoids the need to import
/// `Daemon`.
///
/// # Examples
///
/// ```no_run
/// let mut parent = dmon::options()
///                   .working_directory(Some("/tmp/mydaemon"))
///                   .daemonize();
/// ```
#[must_use]
pub fn options() -> Daemon {
    Daemon::new()
}
