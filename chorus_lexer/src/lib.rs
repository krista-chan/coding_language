//! A lot of the parser/lexer code is a modified version of the rustc parser

mod cursor;

use colored::*;
use regex::Regex;

use cursor::Cursor;

use self::TokenKind::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Token {
    kind: TokenKind,
    len: usize,
}

impl Token {
    fn new(kind: TokenKind, len: usize) -> Self {
        Self { kind, len }
    }
}

pub const VARIABLE_NAME_VALIDATION: [&str; 8] =
    ["/", "+", "-", "%", "=", "<", ">", "="];

#[derive(Debug, PartialEq, PartialOrd)]
pub enum TokenKind {
    // TODO: implement literals
    // literals
    /// 0-9
    // Int(usize),
    /// FLoating point integer
    // Float(f64),
    /// String literal
    // Str(String),
    /// Boolean literal
    // Bool(bool),

    /// `//`
    Comment,
    /// `/*` `*/`
    BlockComment { terminated: bool },

    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `=`
    Assign,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `==`
    Eq,
    /// `<=`
    Lte,
    /// `>=`
    Gte,

    // Keywords
    /// `let foo<T> = bar;`
    Let { t: Types, name: String, val: String },

    /// `;`
    Semi,

    /// A newline
    Newline,
    /// Any whitespace character
    Whitespace,
    /// Unknown token
    Unknown,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Types {
    String,
    Float,
    Int,
    Bool,
    Dyn(String),
}

pub fn tokenize(mut input: &str) -> impl Iterator<Item = Token> + '_ {
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }
        let token = Cursor::new(input).advance_token();
        input = &input[token.len..];

        Some(token)
    })
}

impl Cursor<'_> {
    fn advance_token(&mut self) -> Token {
        let first_char = self.bump().unwrap();
        let token_kind = match first_char {
            // Comments
            '/' => match self.first() {
                '/' => {
                    self.bump().unwrap();
                    Comment
                }
                '*' => {
                    self.bump().unwrap();
                    BlockComment { terminated: true }
                }
                _ => Slash,
            },
            // Eq
            '=' => match self.first() {
                '=' => {
                    self.bump().unwrap();
                    Eq
                }
                _ => Assign,
            },

            // Keywords
            'l' => match self.first() {
                'e' => {
                    self.bump().unwrap();
                    match self.first() {
                        't' => {
                            self.bump().unwrap();
                            self.bump().unwrap();
                            let mut let_name = String::new();
                            let mut let_type = String::new();
                            let mut let_valu = String::new();
                            let mut a = self.bump().unwrap();
                            while a != '<' {
                                if a.is_whitespace() || a == ';' || a == '=' {
                                    println!("\n{}", self.ln);
                                    throw_error("Syntax Error:".red().bold(), 
                                    "Missing let type notation (use `let foo<T> = ...;`)".to_string(),
                                    self.ln,
                                    self.col
                                );
                                }
                                let_name.push(a);
                                a = self.bump().unwrap();
                            }

                            if VARIABLE_NAME_VALIDATION
                                .contains(&let_name.as_str())
                            {
                                println!("{}", self.ln);
                                throw_error(
                                    "Syntax Error:".red().bold(),
                                    format!("Invalid let name {}", let_name),
                                    self.ln,
                                    self.col,
                                )
                            }

                            while !a.is_whitespace() {
                                let_type.push(a);
                                a = self.bump().unwrap();
                            }

                            if !Regex::new(r"<(String|Bool|Int|Float|Dyn\(\b.*\b\))>")
                                .unwrap()
                                .is_match(let_type.as_str())
                            {
                                throw_error(
                                    "Syntax Error:".red().bold(),
                                    format!(
                                        "Missing or invalid `type` at `let {}`",
                                        let_name,
                                    ),
                                    self.ln,
                                    self.col,
                                )
                            }

                            while a != ';' {
                                if a.is_whitespace() {
                                    a = self.bump().unwrap();
                                    continue;
                                }
                                let_valu.push(a);
                                a = self.bump().unwrap();
                            }

                            if !let_valu.contains("=") {
                                throw_error(
                                    "Syntax Error:".red().bold(),
                                    format!(
                                        "Missing `=` at `let {}{}`",
                                        let_name, let_type
                                    ),
                                    self.ln,
                                    self.col,
                                )
                            } else if let_valu == "=" {
                                throw_error(
                                    "Syntax Error:".red().bold(),
                                    format!(
                                        "Missing value at `let {}{}`",
                                        let_name, let_type
                                    ),
                                    self.ln,
                                    self.col,
                                )
                            }

                            Let {
                                t: resolve_t(let_type, self.ln, self.col),
                                name: let_name,
                                val: let_valu.replace("=", ""),
                            }
                        }
                        _ => Unknown,
                    }
                }
                _ => Unknown,
            },
            // single char tokens
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '%' => Percent,
            ';' => Semi,
            '<' => match self.first() {
                '=' => {
                    self.bump().unwrap();
                    Lte
                }
                _ => Lt,
            },
            '>' => match self.first() {
                '=' => {
                    self.bump().unwrap();
                    Gte
                }
                _ => Gt,
            },

            _ => Unknown,
        };

        Token::new(token_kind, self.len_consumed())
    }
}

fn resolve_t(let_type: String, ln: i32, col: i32) -> Types {
    if let_type
        .replace("<", "")
        .replace(">", "")
        .starts_with("Dyn")
    {
        Types::Dyn(
            let_type
                .replace("Dyn", "")
                .replace("(", "")
                .replace(")", "")
                .replace("<", "")
                .replace(">", ""),
        )
    } else {
        match let_type.replace("<", "").replace(">", "").as_str() {
            "String" => Types::String,
            "Bool" => Types::Bool,
            "Int" => Types::Int,
            "Float" => Types::Float,
            _ => {
                throw_error("Syntax Error:".red().bold(), format!("Invalid let type {}", let_type), ln, col);
                Types::Dyn(String::from(""))
            }
        }
    }
}

fn throw_error(t: ColoredString, message: String, ln: i32, col: i32) {
    println!("{} {} @ line {}, col {}", t, message, ln, col);
    std::process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An easier way to construct a vector of tokens
    /// to be used in unit tests
    macro_rules! construct_test {
        // construct_test![(TokenKind, 1), (TokenKind, 2)]
        ( $(($typ:tt, $len:expr)),* $(,)?) => {
            vec![$(Token { kind: $typ, len: $len },)*]
        };

        // construct_test![TokenKind, TokenKind, TokenKind]
        ($($typ:tt),* $(,)?) => {
            construct_test![$(($typ, 1),)*]
        };
    }

    const SINGLE_CHAR_TEST_STR: &'static str = "+-*/%;<>";
    const DOUBLE_CHAR_TEST_STR: &'static str = "<=>===";

    #[test]
    fn create_token() {
        let new_token = Token::new(TokenKind::Plus, 1);

        assert_eq!(new_token.kind, TokenKind::Plus);
        assert_eq!(new_token.len, 1);
    }

    #[test]
    fn single_char_tokens() {
        let tokens = tokenize(SINGLE_CHAR_TEST_STR).collect::<Vec<Token>>();
        let expected =
            construct_test![Plus, Minus, Star, Slash, Percent, Semi, Lt, Gt];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn double_char_tokens() {
        let tokens = tokenize(DOUBLE_CHAR_TEST_STR).collect::<Vec<Token>>();
        let expected = construct_test![Lte, Gte, Eq];

        assert_eq!(tokens, expected);
    }
}
