use std::{fmt, io};

pub enum Error {
    /// Some external or unknown error ocurred.
    External(String),
    // UnknownSystem,
    Install {
        package: String,
        reason: String,
    },
    /// This error indicates that a package has no known asset
    /// for the given system.
    MissingSystemAsset,
    /// This error means that there's no release for
    /// the given package.
    MissingRelease,
    MissingProg(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Install { package, reason } => {
                write!(f, "error installing {}: {}", package, reason)
            }
            // Error::UnknownSystem => write!(f, "unknown system"),
            Error::MissingProg(prog) => write!(f, "missing program: {}", prog),
            Error::External(reason) => write!(f, "{}", reason),
            Error::MissingSystemAsset => write!(f, "missing release asset for your system"),
            Error::MissingRelease => write!(f, "no release found"),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::External(format!("{}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::External(format!("io: {}", err))
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::External(format!("invalid regex: {}", err))
    }
}
