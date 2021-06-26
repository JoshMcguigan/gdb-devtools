mod iters;

#[derive(Debug)]
struct Token<'a> {
    text: &'a str,
    /// Location of the start of this token in the file. Tokens cannot contain
    /// newlines, so to find the end you can add the text length to the column.
    location_in_file: Location,
}

#[derive(Debug)]
struct Location {
    line: usize,
    column: usize,
}

/// Represents a single GDB command line, which is one or more
/// lines in the script file.
struct CommandLine<'a> {
    text: &'a str,
    /// The line in the file where this command line starts.
    start_line_in_file: usize,
    // TODO maybe add num_lines?
}

#[derive(Debug)]
enum Command<'a> {
    Define {
        define: Token<'a>,
        body: Vec<Command<'a>>,
        // TODO get end token - should be optional in case
        // it hasn't been written yet
        //
        // also add ability to track unexpected tokens and add tests for this
    },
    Other {
        command: Token<'a>,
        args: Vec<Token<'a>>,
    },
}

fn parse(input: &str) -> Vec<Command> {
    parse_until(&mut iters::lines(input).into_iter(), false)
}

fn parse_until<'a>(
    input: &mut impl Iterator<Item = CommandLine<'a>>,
    until_end: bool,
) -> Vec<Command<'a>> {
    let mut commands = vec![];
    while let Some(line) = input.next() {
        let mut tokens = iters::tokens(&line);
        match tokens.first().map(|t| t.text) {
            Some("define") => {
                let body = parse_until(input, true);
                commands.push(Command::Define {
                    define: tokens.remove(0),
                    body,
                });
            }
            Some("end") => {
                if until_end {
                    return commands;
                }
            }
            // Ignore empty lines
            None => {}
            Some(command) => {
                commands.push(Command::Other {
                    command: tokens.remove(0),
                    args: tokens,
                });
            }
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::parse;

    fn check_lex_and_parse(input: &str, expect_parse: Expect) {
        expect_parse.assert_eq(
            &parse(input)
                .into_iter()
                .map(|s| format!("{:#?}\n", s))
                .collect::<Vec<String>>()
                .join(""),
        );
    }

    #[test]
    fn commands() {
        let script = r#"
command_with_no_args
command_with_one_arg foo
command_with_two_args foo bar
        "#;

        check_lex_and_parse(
            script,
            expect![[r#"
                Other {
                    command: Token {
                        text: "command_with_no_args",
                        location_in_file: Location {
                            line: 1,
                            column: 0,
                        },
                    },
                    args: [],
                }
                Other {
                    command: Token {
                        text: "command_with_one_arg",
                        location_in_file: Location {
                            line: 2,
                            column: 0,
                        },
                    },
                    args: [
                        Token {
                            text: "foo",
                            location_in_file: Location {
                                line: 2,
                                column: 21,
                            },
                        },
                    ],
                }
                Other {
                    command: Token {
                        text: "command_with_two_args",
                        location_in_file: Location {
                            line: 3,
                            column: 0,
                        },
                    },
                    args: [
                        Token {
                            text: "foo",
                            location_in_file: Location {
                                line: 3,
                                column: 22,
                            },
                        },
                        Token {
                            text: "bar",
                            location_in_file: Location {
                                line: 3,
                                column: 26,
                            },
                        },
                    ],
                }
            "#]],
        );
    }

    #[test]
    fn function_definition() {
        let script = r#"
define say_hi
    echo hi
end
        "#;

        check_lex_and_parse(
            script,
            expect![[r#"
                Define {
                    define: Token {
                        text: "define",
                        location_in_file: Location {
                            line: 1,
                            column: 0,
                        },
                    },
                    body: [
                        Other {
                            command: Token {
                                text: "echo",
                                location_in_file: Location {
                                    line: 2,
                                    column: 4,
                                },
                            },
                            args: [
                                Token {
                                    text: "hi",
                                    location_in_file: Location {
                                        line: 2,
                                        column: 9,
                                    },
                                },
                            ],
                        },
                    ],
                }
            "#]],
        );
    }
}
