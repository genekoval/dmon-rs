use nix::{
    fcntl::{OFlag, open},
    sys::stat::Mode,
    unistd::{dup2_stderr, dup2_stdout},
};
use std::{os::fd::OwnedFd, path::Path};

pub fn null() -> &'static Path {
    Path::new("/dev/null")
}

pub fn root() -> &'static Path {
    Path::new("/")
}

pub fn open_file(file: &Path) -> Result<OwnedFd, String> {
    open(
        file,
        OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_EXCL,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
    )
    .map_err(|err| {
        format!(
            "failed to open file for redirection '{}': {err}",
            file.display()
        )
    })
}

pub fn redirect_stdout(file: &Path) -> Result<(), String> {
    let fd = open_file(file)?;

    dup2_stdout(fd).map_err(|err| {
        format!("failed to redirect stdout to '{}': {err}", file.display())
    })
}

pub fn redirect_stderr(file: &Path) -> Result<(), String> {
    let fd = open_file(file)?;

    dup2_stderr(fd).map_err(|err| {
        format!("failed to redirect stderr to '{}': {err}", file.display())
    })
}
