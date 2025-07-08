use nix::unistd::{self, Gid, Uid};
use std::{
    env,
    ffi::CString,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug)]
pub enum User {
    Id(Uid),
    Name(String),
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(uid) => write!(f, "user with ID ({uid})"),
            Self::Name(name) => write!(f, "user '{name}'"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Group {
    Id(Gid),
    Name(String),
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(gid) => write!(f, "group with ID ({gid})"),
            Self::Name(name) => write!(f, "group '{name}'"),
        }
    }
}

pub fn drop_privileges(
    user: &User,
    group: Option<&Group>,
) -> Result<(), String> {
    let user = find_user(user)?;
    let group = match group {
        Some(group) => find_group(group)?,
        None => find_group(&Group::Id(user.gid))?,
    };

    let name = CString::new(user.name.as_str())
        .expect("User names can only contain valid ASCII characters");

    unistd::initgroups(&name, group.gid).map_err(|err| {
        format!(
            "Failed to set supplementary group list for user '{}': {err}",
            user.name
        )
    })?;

    unistd::setgid(group.gid).map_err(|err| {
        format!("Failed to set group to '{}': {err}", group.name)
    })?;

    unistd::setuid(user.uid).map_err(|err| {
        format!("Failed to set user to '{}': {err}", user.name)
    })?;

    set_env(&user);

    Ok(())
}

fn find_group(group: &Group) -> Result<unistd::Group, String> {
    match group {
        Group::Id(gid) => unistd::Group::from_gid(*gid),
        Group::Name(name) => unistd::Group::from_name(name),
    }
    .map_err(|err| format!("{group}: {err}"))?
    .ok_or_else(|| format!("{group} does not exist"))
}

fn find_user(user: &User) -> Result<unistd::User, String> {
    match user {
        User::Id(uid) => unistd::User::from_uid(*uid),
        User::Name(name) => unistd::User::from_name(name),
    }
    .map_err(|err| format!("{user}: {err}"))?
    .ok_or_else(|| format!("{user} does not exist"))
}

fn set_env(user: &unistd::User) {
    unsafe { env::set_var("USER", &user.name) };
    unsafe { env::set_var("HOME", &user.dir) };
    unsafe { env::set_var("SHELL", &user.shell) };
}
