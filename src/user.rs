//! Types for working with users and groups.

use nix::{
    libc::{gid_t, uid_t},
    unistd::{self, Gid, Uid},
};
use std::{
    env,
    ffi::CString,
    fmt::{self, Display, Formatter},
};

/// A value representing a user.
#[derive(Clone, Debug)]
pub enum User {
    Uid(Uid),
    Name(String),
}

impl User {
    /// Gets the user by either the UID or name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::User;
    ///
    /// let user = User::Uid(0.into()).get().unwrap();
    /// assert_eq!(user.name, "root");
    /// ```
    pub fn get(&self) -> Result<unistd::User, String> {
        match self {
            Self::Uid(uid) => unistd::User::from_uid(*uid),
            Self::Name(name) => unistd::User::from_name(name),
        }
        .map_err(|err| format!("{self}: {err}"))?
        .ok_or_else(|| format!("{self} does not exist"))
    }

    /// Returns the UID if present.
    ///
    /// # Panics
    ///
    /// Panics if the self value equals [`Self::Name`].
    pub fn uid(&self) -> Uid {
        match self {
            Self::Uid(uid) => *uid,
            Self::Name(name) => panic!("expected uid; found name '{name}'"),
        }
    }

    /// Returns the user name if present.
    ///
    /// # Panics
    ///
    /// Panics if the self value equals [`Self::Uid`].
    pub fn name(&self) -> &str {
        match self {
            Self::Uid(uid) => panic!("expected name; found uid {uid}"),
            Self::Name(name) => name,
        }
    }
}

impl Display for User {
    /// Formats the value into a human-readable string.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::User;
    ///
    /// let user: User = 0.into();
    /// assert_eq!(user.to_string(), "user with ID (0)");
    ///
    /// let user: User = "root".into();
    /// assert_eq!(user.to_string(), "user 'root'");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uid(uid) => write!(f, "user with ID ({uid})"),
            Self::Name(name) => write!(f, "user '{name}'"),
        }
    }
}

impl From<&str> for User {
    /// Parses the string into a `User`.
    ///
    /// If the string is a number, a value of [`Self::Uid`] is returned.
    /// Otherwise, a value of [`Self::Name`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::User;
    ///
    /// let user = User::from("0");
    /// assert_eq!(user.uid().as_raw(), 0);
    ///
    /// let user = User::from("root");
    /// assert_eq!(user.name(), "root");
    /// ```
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

/// A value representing a group.
#[derive(Clone, Debug)]
pub enum Group {
    Gid(Gid),
    Name(String),
}

impl Group {
    /// Gets the group by either the GID or name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Group;
    ///
    /// let group = Group::Gid(0.into()).get().unwrap();
    /// assert_eq!(group.name, "root");
    /// ```
    pub fn get(&self) -> Result<unistd::Group, String> {
        match self {
            Self::Gid(gid) => unistd::Group::from_gid(*gid),
            Self::Name(name) => unistd::Group::from_name(name),
        }
        .map_err(|err| format!("{self}: {err}"))?
        .ok_or_else(|| format!("{self} does not exist"))
    }

    /// Returns the GID if present.
    ///
    /// # Panics
    ///
    /// Panics if the self value equals [`Self::Name`].
    pub fn gid(&self) -> Gid {
        match self {
            Self::Gid(gid) => *gid,
            Self::Name(name) => panic!("expected gid; found name '{name}'"),
        }
    }

    /// Returns the group name if present.
    ///
    /// # Panics
    ///
    /// Panics if the self value equals [`Self::Gid`].
    pub fn name(&self) -> &str {
        match self {
            Self::Gid(gid) => panic!("expected name; found gid {gid}"),
            Self::Name(name) => name,
        }
    }
}

impl Display for Group {
    /// Formats the value into a human-readable string.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Group;
    ///
    /// let group: Group = 0.into();
    /// assert_eq!(group.to_string(), "group with ID (0)");
    ///
    /// let group: Group = "root".into();
    /// assert_eq!(group.to_string(), "group 'root'");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gid(gid) => write!(f, "group with ID ({gid})"),
            Self::Name(name) => write!(f, "group '{name}'"),
        }
    }
}

impl From<&str> for Group {
    /// Parses the string into a `Group`.
    ///
    /// If the string is a number, a value of [`Self::Gid`] is returned.
    /// Otherwise, a value of [`Self::Name`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Group;
    ///
    /// let group = Group::from("0");
    /// assert_eq!(group.gid().as_raw(), 0);
    ///
    /// let group = Group::from("root");
    /// assert_eq!(group.name(), "root");
    /// ```
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

/// A user and optional group.
#[derive(Clone, Debug)]
pub struct Privileges {
    pub user: User,
    pub group: Option<Group>,
}

impl Privileges {
    /// Gets the user and group.
    ///
    /// If the self value does not contain a group, the user's group ID is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Privileges;
    ///
    /// let (user, group) = Privileges::from("root").get().unwrap();
    /// assert_eq!(user.uid.as_raw(), 0);
    /// assert_eq!(group.gid.as_raw(), 0);
    /// ```
    pub fn get(&self) -> Result<(unistd::User, unistd::Group), String> {
        let user = self.user.get()?;

        let group = match &self.group {
            Some(group) => group.get()?,
            None => Group::Gid(user.gid).get()?,
        };

        Ok((user, group))
    }

    /// Sets the current process's user and group.
    ///
    /// This method sets the user ID, group ID, and the supplementary group IDs
    /// using all groups that the user is a member of. It also sets the `USER`,
    /// `HOME`, and `SHELL` environment variables to the appropriate values.
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
    /// Parses the string into `Privileges`.
    ///
    /// The string may consist of a numeric UID, a user name, or a user and
    /// group separated by a colon.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Privileges;
    ///
    /// let privileges = Privileges::from("1000");
    /// assert_eq!(privileges.user.uid().as_raw(), 1000);
    /// assert!(privileges.group.is_none());
    ///
    /// let privileges = Privileges::from("myuser:mygroup");
    /// assert_eq!(privileges.user.name(), "myuser");
    /// assert_eq!(privileges.group.unwrap().name(), "mygroup");
    /// ```
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
