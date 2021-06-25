use logos::Lexer;

#[derive(Debug, Clone, Copy, PartialEq, logos::Logos)]
pub enum TokenType {
    #[regex(r#"('|")([^('|")\\]|\\t|\\u|\\n|\\("|'))*('|")"#)]
    String,
    #[regex(r"[0-9]+")]
    Integer,
    #[regex(r"[0-9]+\.+[0-9]+", priority = 2)]
    Float,

    #[regex(r"[a-zA-Z_]+")]
    Identifier,
    #[regex(r"[$][a-zA-Z_]+")]
    ConvenienceIdentifier,

    #[token("define")]
    Define,

    #[token("echo")]
    Echo,
    #[token("set")]
    Set,

    #[token("if")]
    If,
    #[token("else")]
    Else,

    #[token("while")]
    While,
    #[token("loop_break")]
    LoopBreak,
    #[token("loop_continue")]
    LoopContinue,

    #[token("end")]
    End,

    #[token("=")]
    Assignment,

    #[regex(r"#[^(\r\n|\r|\n)]*")]
    Comment,

    #[regex(r"(\r\n|\r|\n)")]
    Newline,
    #[regex(r"[ \t]+")]
    Whitespace,

    #[error]
    Error,
}

#[derive(Debug)]
enum Statement {
    /// The `echo` command is treated differently from other functions, because
    /// the arguments are not evaluated.
    EchoCall,
    FunctionDefine {
        body: Vec<Statement>,
    },
    FunctionCall,
}

fn parse(mut tokens: impl Iterator<Item = TokenType>) -> Vec<Statement> {
    let mut tokens = tokens.peekable();
    let mut out = vec![];

    while let Some(token) = tokens.next() {
        match token {
            TokenType::Echo => {
                // For now we aren't doing anything with function call args.
                let mut _args = vec![];
                while let Some(token) = tokens.next_if(|&token| token != TokenType::Newline) {
                    _args.push(token);
                }
                out.push(Statement::EchoCall);
            }
            TokenType::Define => {
                // For now we aren't doing anything with function call args.
                let mut _args = vec![];
                while let Some(token) = tokens.next_if(|&token| token != TokenType::Newline) {
                    _args.push(token);
                }

                let mut body = vec![];
                while let Some(token) = tokens.next_if(|&token| token != TokenType::End) {
                    body.push(token);
                }

                out.push(Statement::FunctionDefine {
                    body: parse(body.into_iter()),
                });
            }
            TokenType::Identifier => {
                // For now we aren't doing anything with function call args.
                let mut _args = vec![];
                while let Some(token) = tokens.next_if(|&token| token != TokenType::Newline) {
                    _args.push(token);
                }

                out.push(Statement::FunctionCall);
            }
            TokenType::Whitespace => {
                // For now whitespace is not represented in the syntax tree.
            }
            // For now drop all other tokens, but eventually these should all
            // be handled.
            _ => {}
        };
    }

    out
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use logos::Logos;

    use super::{parse, TokenType};

    fn check_lex_and_parse(input: &str, expect_lex: Expect, expect_parse: Expect) {
        let mut out_lex = String::new();

        let mut lexer = TokenType::lexer(input);
        let lexer_clone = lexer.clone();

        while let Some(token_type) = lexer.next() {
            let slice = match lexer.slice() {
                "\n" => "\\n",
                "\t" => "\\t",
                // Render any slice of all whitespace as a single space
                // character between single quotes.
                slice if slice.trim_start().is_empty() => "' '",
                slice => slice,
            };
            out_lex += &format!("{:?} {:?} {}\n", token_type, lexer.span(), slice);
        }

        expect_lex.assert_eq(&out_lex);

        expect_parse.assert_eq(
            &parse(lexer_clone)
                .into_iter()
                .map(|s| format!("{:?}\n", s))
                .collect::<Vec<String>>()
                .join(""),
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
                Newline 0..1 \n
                Define 1..7 define
                Whitespace 7..8 ' '
                Identifier 8..14 say_hi
                Newline 14..15 \n
                Whitespace 15..19 ' '
                Echo 19..23 echo
                Whitespace 23..24 ' '
                Identifier 24..26 hi
                Newline 26..27 \n
                End 27..30 end
                Newline 30..31 \n
                Whitespace 31..39 ' '
            "#]],
            expect![[r#"
                FunctionDefine { body: [EchoCall] }
            "#]],
        );
    }
}
