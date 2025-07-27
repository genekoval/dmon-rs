use crate::user::{Group, Privileges, User};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.name)
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for Group {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.name)
    }
}

impl<'de> Deserialize<'de> for Privileges {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl Serialize for Privileges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let user = self.user.0.name.as_str();
        let group = self.group.0.name.as_str();

        if user == group {
            serializer.serialize_str(user)
        } else {
            let string = format!("{user}:{group}");
            serializer.serialize_str(&string)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_tokens};

    #[test]
    fn user() {
        let user = User::from_uid(0.into()).unwrap();

        assert_tokens(&user, &[Token::String("root")]);
    }

    #[test]
    fn group() {
        let group = Group::from_gid(0.into()).unwrap();

        assert_tokens(&group, &[Token::String("root")]);
    }

    #[test]
    fn privileges() {
        let privileges = Privileges {
            user: User::from_uid(0.into()).unwrap(),
            group: Group::from_gid(0.into()).unwrap(),
        };

        assert_tokens(&privileges, &[Token::String("root")]);
    }
}
