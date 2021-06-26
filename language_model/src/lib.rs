mod iters;

#[derive(Debug)]
pub struct Token<'a> {
    text: &'a str,
    /// Location of the start of this token in the file. Tokens cannot contain
    /// newlines, so to find the end you can add the text length to the column.
    location_in_file: Location,
}

#[derive(Debug)]
pub struct Location {
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
pub enum Command<'a> {
    Define {
        define: Token<'a>,
        // TODO how to note something which is optional in the grammar vs something
        // which is optional because the user hasn't entered it yet (or made a mistake)
        identifier: Option<Token<'a>>,
        body: Vec<Command<'a>>,
        end: Option<Token<'a>>,
        // TODO
        // add ability to track unexpected tokens and add tests for this
    },
    Other {
        command: Token<'a>,
        args: Vec<Token<'a>>,
    },
}

pub fn parse(input: &str) -> Vec<Command> {
    parse_until(&mut iters::lines(input).into_iter(), false).0
}

// TODO clean up this function signature
//
// it is really two functions, the Option<CommandLine> is always None
// if until_end is false
//
// if until_end is true, it is Some assuming the script is well
// formed (not missing an end)
fn parse_until<'a>(
    input: &mut impl Iterator<Item = CommandLine<'a>>,
    until_end: bool,
) -> (Vec<Command<'a>>, Option<CommandLine<'a>>) {
    let mut commands = vec![];
    while let Some(line) = input.next() {
        let mut tokens = iters::tokens(&line);
        match tokens.next() {
            Some(define_token @ Token { text: "define", .. }) => {
                let (body, end_line) = parse_until(input, true);
                commands.push(Command::Define {
                    define: define_token,
                    identifier: tokens.next(),
                    body,
                    // This unwrap is safe because parse_until until_end only returns a
                    // command line if that command line has at least one token and
                    // that token is `end`.
                    //
                    // TODO this should be removed when parse_until is reworked as
                    // described in the todo above.
                    end: end_line.map(|command_line| iters::tokens(&command_line).next().unwrap()),
                });
            }
            Some(Token { text: "end", .. }) => {
                if until_end {
                    return (commands, Some(line));
                }
            }
            Some(command) => {
                commands.push(Command::Other {
                    command,
                    args: tokens.collect(),
                });
            }
            // Ignore empty lines
            None => {}
        }
    }

    (commands, None)
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
                    identifier: Some(
                        Token {
                            text: "say_hi",
                            location_in_file: Location {
                                line: 1,
                                column: 7,
                            },
                        },
                    ),
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
                    end: Some(
                        Token {
                            text: "end",
                            location_in_file: Location {
                                line: 3,
                                column: 0,
                            },
                        },
                    ),
                }
            "#]],
        );
    }
}