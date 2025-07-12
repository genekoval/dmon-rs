use nix::{
    fcntl::{self, OFlag},
    sys::stat::Mode,
    unistd::{dup2_stderr, dup2_stdin, dup2_stdout},
};
use std::{os::fd::OwnedFd, path::Path};

enum Rw {
    ReadOnly,
    WriteOnly,
}

use Rw::*;

pub fn redirect_stdin() -> Result<(), String> {
    let file = Path::new("/dev/null");

    open(file, ReadOnly)
        .and_then(|fd| dup2_stdin(fd).map_err(|err| err.to_string()))
        .map_err(|err| {
            format!("failed to redirect stdin to '{}': {err}", file.display())
        })
}

pub fn redirect_stdout(file: &Path) -> Result<(), String> {
    open(file, WriteOnly)
        .and_then(|fd| dup2_stdout(fd).map_err(|err| err.to_string()))
        .map_err(|err| {
            format!("failed to redirect stdout to '{}': {err}", file.display())
        })
}

pub fn redirect_stderr(file: &Path) -> Result<(), String> {
    open(file, WriteOnly)
        .and_then(|fd| dup2_stderr(fd).map_err(|err| err.to_string()))
        .map_err(|err| {
            format!("failed to redirect stderr to '{}': {err}", file.display())
        })
}

fn open(file: &Path, rw: Rw) -> Result<OwnedFd, String> {
    let flags = match rw {
        Rw::ReadOnly => OFlag::O_RDONLY,
        Rw::WriteOnly => OFlag::O_WRONLY | OFlag::O_APPEND,
    };

    fcntl::open(
        file,
        flags | OFlag::O_CREAT,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
    )
    .map_err(|err| format!("failed to open file: {err}"))
}
