// why cant std::path::Path just implement Display i am tired of writing path.display()

use std::fmt::{self, Debug, Display, Formatter};
use std::path::{Path as StdPath, PathBuf};

/// a file system path.
/// thats it.
/// works as you would expect.
/// implements Display.
#[derive(Clone)]
pub struct Path {
    inner: PathBuf,
}

impl Path {
    pub fn new<T: AsRef<StdPath>>(path: T) -> Self {
        Path {
            inner: path.as_ref().to_path_buf(),
        }
    }

    pub fn join<T: AsRef<StdPath>>(&self, path: T) -> Self {
        let mut p = self.clone();
        p.inner.push(path);
        p
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

// now this is the shit
impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner.display(), f)
    }
}

impl AsRef<StdPath> for Path {
    fn as_ref(&self) -> &StdPath {
        &self.inner
    }
}
