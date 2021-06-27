use std::{collections::HashMap, path::PathBuf};

mod parse;

pub struct Semantics {
    /// All relative imports are assumed to be relative to the project root.
    project_root: PathBuf,
    /// All known files in the project. This struct does no direct file IO, so
    /// the only known files are ones which have been explicitly added.
    files: HashMap<PathBuf, String>,
}

impl Semantics {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            files: HashMap::new(),
        }
    }

    /// Sets the text content for a given file path. If the file `source`s any
    /// external files which are not already loaded, those paths are returned
    /// as UnresolvedPaths.
    pub fn set_file_text(&mut self, path: PathBuf, text: String) -> UnresolvedPaths {
        self.files.insert(path, text);

        // TODO
        // check for unresolved paths
        vec![]
    }
}

type UnresolvedPaths = Vec<PathBuf>;
