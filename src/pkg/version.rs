use anyhow::{bail, Error};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt;

/// Major, minor and patch.
#[derive(Clone)]
pub enum Version {
    Sem(u16, u16, u16),
    Date(u16, u16, u16),
    Unknown(String),
}

impl TryFrom<&String> for Version {
    type Error = Error;

    fn try_from(value: &String) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Version {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        lazy_static! {
            static ref SEMVER: Regex = Regex::new(r"^v?\d\.\d{1,2}\.\d{1,2}").unwrap();
            static ref DATE: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap();
        }

        let mut semver = true;
        let m: Vec<&str> = if SEMVER.is_match(value) {
            value.trim_start_matches('v').split('.').collect()
        } else if DATE.is_match(value) {
            semver = false;
            value.split('-').collect()
        } else {
            return Ok(Version::Unknown(value.to_string()));
        };

        if m.len() != 3 {
            bail!("invalid version format");
        }

        let x = m.first().unwrap();
        let y = m.get(1).unwrap();
        let z = m.get(2).unwrap();

        let x: u16 = str::parse(x)?;
        let y: u16 = str::parse(y)?;
        let z: u16 = str::parse(z)?;

        if semver {
            Ok(Version::Sem(x, y, z))
        } else {
            Ok(Version::Date(x, y, z))
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Sem(x, y, z) => write!(f, "v{}.{}.{}", x, y, z),
            Version::Date(yy, mm, dd) => write!(f, "{}-{:02}-{:02}", yy, mm, dd),
            Version::Unknown(tag) => write!(f, "{}", tag),
        }
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a valid arn")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Version::try_from(v) {
            Ok(version) => Ok(version),
            Err(_) => Err(serde::de::Error::custom(format!(
                "invalid version format: {}",
                v
            ))),
        }
    }
}
