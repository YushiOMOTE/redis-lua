use std::fs::OpenOptions;
use std::io::prelude::*;
use std::ops::Deref;
use std::{fs, path};

pub struct File {
    _temp: mktemp::Temp,
    file: fs::File,
    path: path::PathBuf,
}

impl File {
    pub fn new() -> File {
        let temp = mktemp::Temp::new_file().unwrap();
        let file = OpenOptions::new().write(true).open(&temp).unwrap();
        let path = temp.to_path_buf();

        Self {
            _temp: temp,
            file,
            path,
        }
    }

    pub fn from_str(s: &str) -> File {
        let mut f = File::new();
        f.write(s);
        f
    }

    pub fn path(&self) -> &path::Path {
        &self.path
    }

    pub fn write(&mut self, s: &str) {
        self.file.write_all(s.as_bytes()).unwrap();
        self.file.flush().unwrap();
    }
}

pub struct Path(File);

impl Path {
    fn new(s: &str) -> Self {
        Self(File::from_str(s))
    }
}

impl Deref for Path {
    type Target = path::Path;

    fn deref(&self) -> &Self::Target {
        self.0.path()
    }
}

pub fn as_path(s: &str) -> Path {
    Path::new(s)
}
