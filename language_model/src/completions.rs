use crate::parse::{self, Location};

#[derive(Debug, PartialEq)]
pub(crate) enum CompletionPosition<'a> {
    Command,
    Arg(CompletionPositionArg<'a>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct CompletionPositionArg<'a> {
    /// The command this arg is being passed to.
    pub command: &'a str,
    /// Any args which come before the arg being completed.
    ///
    /// The arg at the cursor position (if there is one) is not
    /// included here, since that is the token we would be trying
    /// to complete.
    pub leading_args: Vec<&'a str>,
}

impl<'a> CompletionPosition<'a> {
    pub(crate) fn new(script: &'a str, cursor_position: Location) -> Option<Self> {
        let line = parse::iters::lines(script)
            .find(|line| line.start_line_in_file == cursor_position.line)?;
        let mut tokens_before_this = parse::iters::tokens(&line).take_while(|token| {
            token.location_in_file.column + token.text.len() < cursor_position.column
        });

        let res = match tokens_before_this.next() {
            Some(command) => CompletionPosition::Arg(CompletionPositionArg {
                command: command.text,
                leading_args: tokens_before_this.map(|t| t.text).collect(),
            }),
            None => CompletionPosition::Command,
        };

        Some(res)
    }

    #[cfg(test)]
    fn into_arg(self) -> Option<CompletionPositionArg<'a>> {
        if let Self::Arg(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompletionPosition, Location};

    #[test]
    fn empty_script() {
        let script = "";
        let completion_position = CompletionPosition::new(script, Location { line: 0, column: 0 })
            .expect("should resolve completion position");

        assert_eq!(CompletionPosition::Command, completion_position);
    }

    #[test]
    fn if_cursor_on_command_it_is_not_included() {
        let script = "def";
        let completion_position = CompletionPosition::new(script, Location { line: 0, column: 2 })
            .expect("should resolve completion position");

        assert_eq!(CompletionPosition::Command, completion_position);
    }

    #[test]
    fn first_and_only_arg() {
        let script = "define  ";
        let completion_position_arg =
            CompletionPosition::new(script, Location { line: 0, column: 7 })
                .expect("should resolve completion position")
                .into_arg()
                .expect("should resolve as arg");

        assert_eq!("define", completion_position_arg.command);
        assert!(completion_position_arg.leading_args.is_empty());
    }

    #[test]
    fn last_arg() {
        let script = "set max-completions  ";
        let completion_position_arg = CompletionPosition::new(
            script,
            Location {
                line: 0,
                column: 20,
            },
        )
        .expect("should resolve completion position")
        .into_arg()
        .expect("should resolve as arg");

        assert_eq!("set", completion_position_arg.command);
        assert_eq!(
            vec!["max-completions"],
            completion_position_arg.leading_args
        );
    }

    #[test]
    fn middle_arg() {
        let script = "set   max-completions";
        let completion_position_arg =
            CompletionPosition::new(script, Location { line: 0, column: 4 })
                .expect("should resolve completion position")
                .into_arg()
                .expect("should resolve as arg");

        assert_eq!("set", completion_position_arg.command);
        assert!(completion_position_arg.leading_args.is_empty());
    }

    #[test]
    fn if_cursor_on_arg_it_is_not_included() {
        let script = "set max-completions";
        let completion_position_arg =
            CompletionPosition::new(script, Location { line: 0, column: 5 })
                .expect("should resolve completion position")
                .into_arg()
                .expect("should resolve as arg");

        assert_eq!("set", completion_position_arg.command);
        assert!(completion_position_arg.leading_args.is_empty());
    }
}
