use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod parse;
use parse::{parse, Command};

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

    pub fn find_definition<'a>(&self, item_position: FilePosition<'a>) -> Option<FilePosition<'a>> {
        let script = self.files.get(item_position.file)?;

        // Find the token at the requested position.
        let line = parse::iters::lines(script)
            .find(|line| line.start_line_in_file == item_position.line)?;
        let token =
            parse::iters::tokens(&line).find(|token| token.is_at_location(item_position))?;
        let identifier = token.text;

        // Find most recent definition of that token before the requested position.
        parse(script).into_iter().rev().find_map(|command| {
            if let Command::Define {
                define: define_command,
                identifier: Some(defined_identifier),
                ..
            } = command
            {
                if defined_identifier.text == identifier
                    && define_command.location_in_file.line < item_position.line
                {
                    Some(FilePosition {
                        file: item_position.file,
                        line: defined_identifier.location_in_file.line,
                        column: defined_identifier.location_in_file.column,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

type UnresolvedPaths = Vec<PathBuf>;

#[derive(Copy, Clone)]
pub struct FilePosition<'a> {
    pub file: &'a Path,
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{FilePosition, Semantics};

    #[test]
    fn find_definition_simple() {
        let script = r#"
define say_hi
    echo hi
end

say_hi
        "#;
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script.to_owned());

            semantics
        };

        let item_position = FilePosition {
            file: &script_path,
            line: 5,
            column: 0,
        };

        let definition = semantics
            .find_definition(item_position)
            .expect("should find definition");

        assert_eq!(script_path, definition.file);
        assert_eq!(1, definition.line);
        assert_eq!(7, definition.column);
    }

    #[test]
    fn find_definition_returns_none_if_def_is_after_identifier() {
        let script = r#"
say_hi

define say_hi
    echo hi
end
        "#;
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script.to_owned());

            semantics
        };

        let item_position = FilePosition {
            file: &script_path,
            line: 1,
            column: 0,
        };

        let definition = semantics.find_definition(item_position);

        assert!(definition.is_none());
    }

    #[test]
    fn find_definition_returns_most_recent_definition() {
        let script = r#"
define say_hi
    echo hi
end

define say_hi
    echo hi!!!
end

say_hi
        "#;
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script.to_owned());

            semantics
        };

        let item_position = FilePosition {
            file: &script_path,
            line: 9,
            column: 0,
        };

        let definition = semantics
            .find_definition(item_position)
            .expect("should find definition");

        assert_eq!(script_path, definition.file);
        assert_eq!(5, definition.line);
        assert_eq!(7, definition.column);
    }
}
