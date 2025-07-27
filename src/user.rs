//! Types for working with users and groups.

use nix::{
    libc::{gid_t, uid_t},
    unistd::{self, Gid, Uid},
};
use std::{
    env,
    ffi::CString,
    fmt::{self, Display},
    str::FromStr,
};

/// A value representing a user.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User(pub unistd::User);

impl User {
    /// Get a user by UID.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::User;
    ///
    /// let user = User::from_uid(0.into()).unwrap();
    /// assert_eq!(user.0.name, "root");
    /// ```
    pub fn from_uid(uid: Uid) -> Result<Self, String> {
        let user = unistd::User::from_uid(uid)
            .map_err(|err| format!("user with ID ({uid}): {err}"))?
            .ok_or_else(|| format!("user with ID ({uid}) does not exist"))?;

        Ok(Self(user))
    }

    /// Get a user by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::{nix::unistd::Uid, user::User};
    ///
    /// let user: User = "root".parse().unwrap();
    /// assert_eq!(user.0.uid, Uid::from_raw(0));
    /// ```
    pub fn from_name(name: &str) -> Result<Self, String> {
        let user = unistd::User::from_name(name)
            .map_err(|err| format!("user '{name}': {err}"))?
            .ok_or_else(|| format!("user '{name}' does not exist"))?;

        Ok(Self(user))
    }

    /// Sets certain environment variables to the appropriate values.
    ///
    /// Specifically, this sets the `USER`, `HOME`, and `SHELL` environment
    /// variables to the values found in `/etc/passwd` for the user.
    ///
    /// # Safety
    ///
    /// This method is unsafe to call from a multi-threaded program. See
    /// [`std::env::set_var`] for more information.
    pub unsafe fn set_env(&self) {
        unsafe { env::set_var("USER", &self.0.name) };
        unsafe { env::set_var("HOME", &self.0.dir) };
        unsafe { env::set_var("SHELL", &self.0.shell) };
    }
}

impl Display for User {
    /// Writes the user name to the given formatter.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::User;
    ///
    /// let user = User::from_uid(0.into()).unwrap();
    /// assert_eq!(user.to_string(), "root");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.name)
    }
}

impl FromStr for User {
    type Err = String;

    /// Parses the string into a `User`.
    ///
    /// If the string is a number, it will be treated as the user's UID.
    /// Otherwise, it will be treated as the user's name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::{nix::unistd::Uid, user::User};
    ///
    /// let user: User = "0".parse().unwrap();
    /// assert_eq!(user.0.name, "root");
    ///
    /// let user: User = "root".parse().unwrap();
    /// assert_eq!(user.0.uid, Uid::from_raw(0));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<uid_t>().ok() {
            Some(uid) => Self::from_uid(uid.into()),
            None => Self::from_name(s),
        }
    }
}

/// A value representing a group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group(pub unistd::Group);

impl Group {
    /// Get a group by GID.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Group;
    ///
    /// let group = Group::from_gid(0.into()).unwrap();
    /// assert_eq!(group.0.name, "root");
    /// ```
    pub fn from_gid(gid: Gid) -> Result<Self, String> {
        let group = unistd::Group::from_gid(gid)
            .map_err(|err| format!("group with ID ({gid}): {err}"))?
            .ok_or_else(|| format!("group with ID ({gid}) does not exist"))?;

        Ok(Self(group))
    }

    /// Get a group by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::{nix::unistd::Gid, user::Group};
    ///
    /// let group = Group::from_name("root").unwrap();
    /// assert_eq!(group.0.gid, Gid::from_raw(0));
    /// ```
    pub fn from_name(name: &str) -> Result<Self, String> {
        let group = unistd::Group::from_name(name)
            .map_err(|err| format!("group '{name}': {err}"))?
            .ok_or_else(|| format!("group '{name}' does not exist"))?;

        Ok(Self(group))
    }
}

impl Display for Group {
    /// Writes the group name to the given formatter.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::Group;
    ///
    /// let group = Group::from_gid(0.into()).unwrap();
    /// assert_eq!(group.to_string(), "root");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.name)
    }
}

impl FromStr for Group {
    type Err = String;

    /// Parses the string into a `Group`.
    ///
    /// If the string is a number, it will be treated as the group's GID.
    /// Otherwise, it will be treated as the group's name.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::{nix::unistd::Gid, user::Group};
    ///
    /// let group: Group = "0".parse().unwrap();
    /// assert_eq!(group.0.name, "root");
    ///
    /// let group: Group = "root".parse().unwrap();
    /// assert_eq!(group.0.gid, Gid::from_raw(0));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<gid_t>().ok() {
            Some(gid) => Self::from_gid(gid.into()),
            None => Self::from_name(s),
        }
    }
}

/// A user and group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Privileges {
    pub user: User,
    pub group: Group,
}

impl Privileges {
    /// Sets the current process's user and group.
    ///
    /// This method sets the user ID, group ID, and the supplementary group IDs
    /// using all groups that the user is a member of.
    pub fn drop_privileges(&self) -> Result<(), String> {
        let user = &self.user.0;
        let group = &self.group.0;

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

        Ok(())
    }

    /// Sets certain environment variables to the appropriate values.
    ///
    /// Specifically, this sets the `USER`, `HOME`, and `SHELL` environment
    /// variables to the values found in `/etc/passwd` for the user.
    ///
    /// # Safety
    ///
    /// This method is unsafe to call from a multi-threaded program. See
    /// [`std::env::set_var`] for more information.
    pub unsafe fn set_env(&self) {
        unsafe { self.user.set_env() };
    }
}

impl Display for Privileges {
    /// Writes the user and group names to the given formatter.
    ///
    /// If the user and group names are the same, only the name is written.
    /// Otherwise, both names separated by a colon are written.
    ///
    /// # Examples
    ///
    /// ```
    /// use dmon::user::{Group, Privileges, User};
    ///
    /// let privileges = Privileges {
    ///     user: User::from_name("root").unwrap(),
    ///     group: Group::from_name("root").unwrap(),
    /// };
    ///
    /// assert_eq!(privileges.to_string(), "root");
    /// ```
    ///
    /// ```
    /// use dmon::user::{Group, Privileges, User};
    ///
    /// let privileges = Privileges {
    ///     user: User::from_name("root").unwrap(),
    ///     group: Group::from_name("users").unwrap(),
    /// };
    ///
    /// assert_eq!(privileges.to_string(), "root:users");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let user = self.user.0.name.as_str();
        let group = self.group.0.name.as_str();

        if user == group {
            f.write_str(user)
        } else {
            let s = format!("{user}:{group}");
            f.write_str(&s)
        }
    }
}

impl FromStr for Privileges {
    type Err = String;

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
    /// let privileges: Privileges = "root:root".parse().unwrap();
    /// assert_eq!(privileges.user.0.name, "root");
    /// assert_eq!(privileges.group.0.name, "root");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.splitn(2, ':');

        let user: User = values.next().unwrap().parse()?;

        let group: Group = if let Some(group) = values.next() {
            group.parse()?
        } else {
            Group::from_gid(user.0.gid)?
        };

        Ok(Self { user, group })
    }
}
