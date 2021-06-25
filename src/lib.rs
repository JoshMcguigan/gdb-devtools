#[derive(Debug)]
enum Command {
    // TODO these need to include offsets into file
    Define { body: Vec<Command> },
    Other { command: String, args: Vec<String> },
}

fn parse(input: &str) -> Vec<Command> {
    parse_until(&mut input.lines(), false)
}

fn parse_until<'a>(input: &mut impl Iterator<Item = &'a str>, until_end: bool) -> Vec<Command> {
    let mut commands = vec![];
    while let Some(line) = input.next() {
        match line.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["define", ..] => {
                let body = parse_until(input, true);
                commands.push(Command::Define { body });
            }
            ["end", ..] => {
                if until_end {
                    return commands;
                }
            }
            // Ignore empty lines
            [] => {}
            [command, args @ ..] => {
                commands.push(Command::Other {
                    command: command.to_string(),
                    args: args.into_iter().map(|s| s.to_string()).collect(),
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
                    command: "command_with_no_args",
                    args: [],
                }
                Other {
                    command: "command_with_one_arg",
                    args: [
                        "foo",
                    ],
                }
                Other {
                    command: "command_with_two_args",
                    args: [
                        "foo",
                        "bar",
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
                    body: [
                        Other {
                            command: "echo",
                            args: [
                                "hi",
                            ],
                        },
                    ],
                }
            "#]],
        );
    }
}
