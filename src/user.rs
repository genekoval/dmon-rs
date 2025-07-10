use nix::{
    libc::{gid_t, uid_t},
    unistd::{self, Gid, Uid},
};
use std::{
    env,
    ffi::CString,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug)]
pub enum User {
    Uid(Uid),
    Name(String),
}

impl User {
    pub fn get(&self) -> Result<unistd::User, String> {
        match self {
            Self::Uid(uid) => unistd::User::from_uid(*uid),
            Self::Name(name) => unistd::User::from_name(name),
        }
        .map_err(|err| format!("{self}: {err}"))?
        .ok_or_else(|| format!("{self} does not exist"))
    }

    pub fn uid(&self) -> Uid {
        match self {
            Self::Uid(uid) => *uid,
            Self::Name(name) => panic!("expected uid; found name '{name}'"),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Uid(uid) => panic!("expected name; found uid {uid}"),
            Self::Name(name) => name,
        }
    }
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uid(uid) => write!(f, "user with ID ({uid})"),
            Self::Name(name) => write!(f, "user '{name}'"),
        }
    }
}

impl From<&str> for User {
    fn from(value: &str) -> Self {
        match value.parse::<uid_t>().ok() {
            Some(uid) => Self::Uid(uid.into()),
            None => Self::Name(value.into()),
        }
    }
}

impl From<Uid> for User {
    fn from(value: Uid) -> Self {
        Self::Uid(value)
    }
}

impl From<uid_t> for User {
    fn from(value: uid_t) -> Self {
        Self::Uid(value.into())
    }
}

#[derive(Clone, Debug)]
pub enum Group {
    Gid(Gid),
    Name(String),
}

impl Group {
    pub fn get(&self) -> Result<unistd::Group, String> {
        match self {
            Self::Gid(gid) => unistd::Group::from_gid(*gid),
            Self::Name(name) => unistd::Group::from_name(name),
        }
        .map_err(|err| format!("{self}: {err}"))?
        .ok_or_else(|| format!("{self} does not exist"))
    }

    pub fn gid(&self) -> Gid {
        match self {
            Self::Gid(gid) => *gid,
            Self::Name(name) => panic!("expected gid; found name '{name}'"),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Gid(gid) => panic!("expected name; found gid {gid}"),
            Self::Name(name) => name,
        }
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gid(gid) => write!(f, "group with ID ({gid})"),
            Self::Name(name) => write!(f, "group '{name}'"),
        }
    }
}

impl From<&str> for Group {
    fn from(value: &str) -> Self {
        match value.parse::<gid_t>().ok() {
            Some(gid) => Self::Gid(gid.into()),
            None => Self::Name(value.into()),
        }
    }
}

impl From<Gid> for Group {
    fn from(value: Gid) -> Self {
        Self::Gid(value)
    }
}

impl From<gid_t> for Group {
    fn from(value: gid_t) -> Self {
        Self::Gid(value.into())
    }
}

#[derive(Clone, Debug)]
pub struct Privileges {
    pub user: User,
    pub group: Option<Group>,
}

impl Privileges {
    pub fn get(&self) -> Result<(unistd::User, unistd::Group), String> {
        let user = self.user.get()?;

        let group = match &self.group {
            Some(group) => group.get()?,
            None => Group::Gid(user.gid).get()?,
        };

        Ok((user, group))
    }

    pub fn drop_privileges(&self) -> Result<(), String> {
        let (user, group) = self.get()?;

        let name = CString::new(user.name.as_str())
            .expect("user names can only contain valid ASCII characters");

        unistd::initgroups(&name, group.gid).map_err(|err| {
            format!(
                "failed to set supplementary group list for user '{}': {err}",
                user.name
            )
        })?;

        unistd::setgid(group.gid).map_err(|err| {
            format!("failed to set group to '{}': {err}", group.name)
        })?;

        unistd::setuid(user.uid).map_err(|err| {
            format!("failed to set user to '{}': {err}", user.name)
        })?;

        set_env(&user);

        Ok(())
    }
}

impl From<&str> for Privileges {
    fn from(value: &str) -> Self {
        let mut values = value.splitn(2, ':');

        let user = values.next().unwrap();
        let group = values.next();

        Self {
            user: user.into(),
            group: group.map(|group| group.into()),
        }
    }
}

impl From<User> for Privileges {
    fn from(user: User) -> Self {
        Self { user, group: None }
    }
}

impl From<Uid> for Privileges {
    fn from(value: Uid) -> Self {
        Self {
            user: value.into(),
            group: None,
        }
    }
}

impl From<uid_t> for Privileges {
    fn from(value: uid_t) -> Self {
        Self {
            user: value.into(),
            group: None,
        }
    }
}

impl<U, G> From<(U, G)> for Privileges
where
    U: Into<User>,
    G: Into<Group>,
{
    fn from((user, group): (U, G)) -> Self {
        Self {
            user: user.into(),
            group: Some(group.into()),
        }
    }
}

impl<U, G> From<(U, Option<G>)> for Privileges
where
    U: Into<User>,
    G: Into<Group>,
{
    fn from((user, group): (U, Option<G>)) -> Self {
        Self {
            user: user.into(),
            group: group.map(|value| value.into()),
        }
    }
}

fn set_env(user: &unistd::User) {
    unsafe { env::set_var("USER", &user.name) };
    unsafe { env::set_var("HOME", &user.dir) };
    unsafe { env::set_var("SHELL", &user.shell) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_uid() {
        let user: User = "0".into();

        assert!(user.uid().is_root());
    }

    #[test]
    fn user_name() {
        let user: User = "root".into();

        assert_eq!("root", user.name());
    }

    #[test]
    fn user_get() {
        let user: User = 0.into();
        let user = user.get().unwrap();

        assert!(user.uid.is_root());
        assert_eq!("root", user.name);
    }

    #[test]
    fn group_gid() {
        let group: Group = "0".into();

        assert_eq!(0, group.gid().as_raw());
    }

    #[test]
    fn group_name() {
        let group: Group = "root".into();

        assert_eq!("root", group.name());
    }

    #[test]
    fn group_get() {
        let group: Group = 0.into();
        let group = group.get().unwrap();

        assert_eq!(0, group.gid.as_raw());
        assert_eq!("root", group.name);
    }

    #[test]
    fn privileges_numeric() {
        let Privileges { user, group } = "0:0".into();

        assert!(user.uid().is_root());
        assert_eq!(0, group.unwrap().gid().as_raw());
    }

    #[test]
    fn privileges_names() {
        let Privileges { user, group } = "root:root".into();

        assert_eq!("root", user.name());
        assert_eq!("root", group.unwrap().name());
    }

    #[test]
    fn privileges_mixed() {
        let Privileges { user, group } = "root:0".into();

        assert_eq!("root", user.name());
        assert_eq!(0, group.unwrap().gid().as_raw());
    }

    #[test]
    fn privileges_user_only() {
        let Privileges { user, group } = "root".into();

        assert_eq!("root", user.name());
        assert!(group.is_none());
    }

    #[test]
    fn privileges_get() {
        let privileges: Privileges = 0.into();
        let (user, group) = privileges.get().unwrap();

        assert_eq!("root", user.name);
        assert_eq!("root", group.name);
    }
}
