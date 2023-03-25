use std::{ffi::OsString, path::PathBuf};

use super::*;

#[derive(Clone)]
pub struct ImportStatement {
    pub path: ImportPath,
    pub source: Option<Span>,
}

impl ImportStatement {
    pub fn get_imported_file(&self, current_path: &Path, root_path: &Path) -> PathBuf {
        fn push_files(path: &mut PathBuf, files: &Vec<Name>) {
            for file in files {
                path.push(PathBuf::from(OsString::from(file)));
            }
        }

        match &self.path {
            ImportPath::Absolute(files) => {
                let mut path = root_path.to_path_buf();
                push_files(&mut path, files);
                path
            }
            ImportPath::Relative(count, files) => {
                let mut path = current_path.to_path_buf();
                for _ in 0..*count {
                    path.pop();
                }
                push_files(&mut path, files);
                path
            }
        }
    }
}

#[derive(Clone)]
pub enum ImportPath {
    Absolute(Vec<Name>),
    Relative(u32 /* number of "../" */, Vec<Name>),
}
