use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

mod completions;
use completions::CompletionPosition;

mod parse;
use parse::{parse, Command};

#[cfg(test)]
mod test_support;

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
    ///
    /// The path must be an absolute path.
    pub fn set_file_text(&mut self, path: PathBuf, text: String) -> UnresolvedPaths {
        let unresolved_paths = parse(&text)
            .into_iter()
            .filter_map(|command| {
                if let Command::Source {
                    file_path: Some(file_path),
                    ..
                } = command
                {
                    let path = PathBuf::from(file_path.text);

                    if self.files.contains_key(&path) {
                        None
                    } else {
                        let path = self.canonicalize_path(PathBuf::from(file_path.text));

                        Some(path)
                    }
                } else {
                    None
                }
            })
            .collect();

        self.files.insert(path, text);

        unresolved_paths
    }

    // TODO
    // this should return full CommandDefine struct, so we could impl
    // hover using it
    pub fn find_definition(&self, cursor_position: CursorPosition) -> Option<CursorPosition> {
        let script = self.files.get(cursor_position.file)?;

        // Find the token at the requested position.
        let line = parse::iters::lines(script)
            .find(|line| line.start_line_in_file == cursor_position.line)?;
        let token =
            parse::iters::tokens(&line).find(|token| token.is_at_location(cursor_position))?;
        let identifier = token.text;

        // Find most recent definition of that token before the requested position.
        self.find_definition_in(cursor_position.file, identifier, Some(cursor_position.line))
    }

    pub fn find_completions(&self, cursor_position: CursorPosition) -> Completions {
        let script = match self.files.get(cursor_position.file) {
            Some(script) => script,
            None => return Completions::default(),
        };
        let completion_position = match CompletionPosition::new(script, cursor_position.into()) {
            Some(completion_position) => completion_position,
            None => return Completions::default(),
        };

        match completion_position {
            CompletionPosition::Command => {
                let built_in = ["define", "if", "else", "end"]
                    .iter()
                    .map(|&command| Completion {
                        text: command.to_owned(),
                    })
                    .collect();

                Completions {
                    built_in,
                    user_provided: vec![],
                }
            }
            // TODO handle completions in arg position, including user defined variables
            CompletionPosition::Arg(_) => Completions::default(),
        }
    }

    /// Find the definition of the given identifier in the given script, including
    /// traversing `source` imports.
    ///
    /// If a line limit is given, the definition must happen above the given line. This
    /// is useful to ensure the definition isn't below the usage.
    fn find_definition_in(
        &self,
        script_path: &Path,
        identifier: &str,
        line_limit: Option<usize>,
    ) -> Option<CursorPosition> {
        let (file_path, script) = self.files.get_key_value(script_path)?;
        parse(script)
            .into_iter()
            .rev()
            .find_map(|command| match command {
                Command::Define {
                    define: define_command,
                    identifier: Some(defined_identifier),
                    ..
                } => {
                    if defined_identifier.text == identifier {
                        if let Some(line_limit) = line_limit {
                            if define_command.location_in_file.line >= line_limit {
                                return None;
                            }
                        }
                        Some(CursorPosition {
                            file: file_path,
                            line: defined_identifier.location_in_file.line,
                            column: defined_identifier.location_in_file.column,
                        })
                    } else {
                        None
                    }
                }
                Command::Source {
                    file_path: Some(file_path),
                    ..
                } => {
                    let path = self.canonicalize_path(PathBuf::from(file_path.text));
                    self.find_definition_in(&path, identifier, None)
                }
                _ => None,
            })
    }

    fn canonicalize_path(&self, path: PathBuf) -> PathBuf {
        if path.is_relative() {
            self.project_root.join(path)
        } else {
            path
        }
    }
}

type UnresolvedPaths = Vec<PathBuf>;

#[derive(Copy, Clone)]
pub struct CursorPosition<'a> {
    pub file: &'a Path,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct Completion {
    pub text: String,
}

#[derive(Default)]
pub struct Completions {
    pub built_in: Vec<Completion>,
    pub user_provided: Vec<Completion>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expect_test::{expect, Expect};

    use crate::test_support::parse_cursor_position;

    use super::{CursorPosition, Semantics};

    #[test]
    fn find_definition_simple() {
        let script = r#"
define say_hi
    echo hi
end

<|>say_hi
        "#;
        let (script, location) = parse_cursor_position(script);
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script);

            semantics
        };

        let item_position = CursorPosition {
            file: &script_path,
            line: location.line,
            column: location.column,
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
<|>say_hi

define say_hi
    echo hi
end
        "#;
        let (script, location) = parse_cursor_position(script);
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script);

            semantics
        };

        let item_position = CursorPosition {
            file: &script_path,
            line: location.line,
            column: location.column,
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

<|>say_hi
        "#;
        let (script, location) = parse_cursor_position(script);
        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script);

            semantics
        };

        let item_position = CursorPosition {
            file: &script_path,
            line: location.line,
            column: location.column,
        };

        let definition = semantics
            .find_definition(item_position)
            .expect("should find definition");

        assert_eq!(script_path, definition.file);
        assert_eq!(5, definition.line);
        assert_eq!(7, definition.column);
    }

    #[test]
    fn find_definition_from_other_file() {
        let script_1 = r#"
source hello.gdb

<|>say_hi
        "#;
        let (script_1, location) = parse_cursor_position(script_1);
        let script_1_path = PathBuf::from("/home/user/foo.gdb");
        let script_2 = r#"
define say_hi
    echo hi
end
        "#;
        let script_2_path = PathBuf::from("/home/user/hello.gdb");

        let semantics = {
            // We use a non-empty CWD here to show that path canonicalization
            // works.
            let fake_cwd: PathBuf = PathBuf::from("/home/user");
            let mut semantics = Semantics::new(fake_cwd);
            let unresolved_imports = semantics.set_file_text(script_1_path.clone(), script_1);
            assert_eq!(1, unresolved_imports.len());
            assert_eq!(&script_2_path, unresolved_imports.get(0).unwrap());

            semantics.set_file_text(script_2_path.clone(), script_2.to_owned());

            semantics
        };

        let item_position = CursorPosition {
            file: &script_1_path,
            line: location.line,
            column: location.column,
        };

        let definition = semantics
            .find_definition(item_position)
            .expect("should find definition");

        assert_eq!(script_2_path, definition.file);
        assert_eq!(1, definition.line);
        assert_eq!(7, definition.column);
    }

    #[test]
    fn set_file_text_requests_unresolved_imports() {
        let script_1 = r#"source bar.gdb"#;
        let script_1_path = PathBuf::from("foo.gdb");

        let script_2 = r#"echo hi from bar"#;
        let script_2_path = PathBuf::from("bar.gdb");

        let script_3 = r#"source bar.gdb"#;
        let script_3_path = PathBuf::from("baz.gdb");

        let mut semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let semantics = Semantics::new(fake_cwd);

            semantics
        };

        let unresolved_imports =
            semantics.set_file_text(script_1_path.clone(), script_1.to_owned());
        assert_eq!(1, unresolved_imports.len());
        assert_eq!(&script_2_path, unresolved_imports.get(0).unwrap());

        let unresolved_imports =
            semantics.set_file_text(script_2_path.clone(), script_2.to_owned());
        assert!(unresolved_imports.is_empty());

        let unresolved_imports =
            semantics.set_file_text(script_3_path.clone(), script_3.to_owned());
        assert!(unresolved_imports.is_empty());
    }

    fn check_completions_user_provided(script: &str, expect_parse: Expect) {
        let (script, location) = parse_cursor_position(script);

        let script_path = PathBuf::from("foo.gdb");

        let semantics = {
            let fake_cwd: PathBuf = PathBuf::new();
            let mut semantics = Semantics::new(fake_cwd);
            semantics.set_file_text(script_path.clone(), script.to_owned());

            semantics
        };

        let cursor_position = CursorPosition {
            file: &script_path,
            line: location.line,
            column: location.column,
        };

        let completions = semantics.find_completions(cursor_position);

        expect_parse.assert_eq(
            &completions
                .user_provided
                .into_iter()
                .map(|completion| completion.text)
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }

    #[test]
    fn completions_user_provided_empty_script() {
        check_completions_user_provided("<|>", expect![[r#""#]]);
    }
}
