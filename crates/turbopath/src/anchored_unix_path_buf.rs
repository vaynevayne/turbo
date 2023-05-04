use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{AnchoredSystemPathBuf, IntoUnix, PathError, PathValidationError, RelativeUnixPathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize)]
pub struct AnchoredUnixPathBuf(PathBuf);

impl TryFrom<&Path> for AnchoredUnixPathBuf {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.is_absolute() {
            return Err(PathError::PathValidationError(
                PathValidationError::NotRelative(path.to_path_buf()),
            ));
        }

        Ok(AnchoredUnixPathBuf(path.into_unix()?))
    }
}

impl AnchoredUnixPathBuf {
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    pub fn to_str(&self) -> Result<&str, PathValidationError> {
        self.0
            .to_str()
            .ok_or_else(|| PathValidationError::InvalidUnicode(self.0.clone()))
    }
}

impl From<AnchoredUnixPathBuf> for PathBuf {
    fn from(path: AnchoredUnixPathBuf) -> PathBuf {
        path.0
    }
}

impl Into<RelativeUnixPathBuf> for AnchoredUnixPathBuf {
    fn into(self) -> RelativeUnixPathBuf {
        unsafe { RelativeUnixPathBuf::unchecked_new(self.0) }
    }
}

impl AsRef<Path> for AnchoredUnixPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl From<RelativeUnixPathBuf> for AnchoredUnixPathBuf {
    fn from(path: RelativeUnixPathBuf) -> Self {
        AnchoredUnixPathBuf(path.into())
    }
}

impl TryFrom<&AnchoredSystemPathBuf> for AnchoredUnixPathBuf {
    type Error = PathError;
    fn try_from(path: &AnchoredSystemPathBuf) -> Result<Self, Self::Error> {
        Ok(AnchoredUnixPathBuf(path.as_path().into_unix()?))
    }
}
