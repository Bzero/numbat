use crate::span::{SourceCodePositition, Span};

use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenizerErrorKind {
    #[error("Unexpected character: '{character}'")]
    UnexpectedCharacter { character: char },

    #[error("Unexpected character in negative exponent")]
    UnexpectedCharacterInNegativeExponent { character: Option<char> },

    #[error("Unexpected character in number literal: '{0}'")]
    UnexpectedCharacterInNumberLiteral(char),

    #[error("Unexpected character in identifier: '{0}'")]
    UnexpectedCharacterInIdentifier(char),

    #[error("Expected digit")]
    ExpectedDigit { character: Option<char> },

    #[error("Expected base-{base} digit")]
    ExpectedDigitInBase {
        base: usize,
        character: Option<char>,
    },

    #[error("Unterminated string")]
    UnterminatedString,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{kind}")]
pub struct TokenizerError {
    pub kind: TokenizerErrorKind,
    pub span: Span,
}

type Result<T> = std::result::Result<T, TokenizerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Brackets
    LeftParen,
    RightParen,
    LeftAngleBracket,
    RightAngleBracket,

    // Operators and special signs
    Plus,
    Minus,
    Multiply,
    Power,
    Divide,
    Per,
    Modulo,
    Comma,
    Arrow,
    Equal,
    Colon,
    ColonColon,
    PostfixApply,
    UnicodeExponent,
    At,
    Ellipsis,

    // Keywords
    Let,
    Fn,
    Dimension,
    Unit,
    Use,

    Long,
    Short,
    Both,
    None,

    // Procedure calls
    ProcedurePrint,
    ProcedureAssertEq,

    // Variable-length tokens
    Number,
    IntegerWithBase(usize),
    Identifier,
    String,

    // Other
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String, // TODO(minor): could be a &'str view into the input
    pub span: Span,
}

struct Tokenizer {
    input: Vec<char>,
    current: SourceCodePositition,
    token_start: SourceCodePositition,
}

fn is_exponent_char(c: char) -> bool {
    matches!(c, '¹' | '²' | '³' | '⁴' | '⁵')
}

fn is_currency_char(c: char) -> bool {
    let c_u32 = c as u32;

    // See https://en.wikipedia.org/wiki/Currency_Symbols_(Unicode_block)
    (0x20A0..=0x20CF).contains(&c_u32) || c == '£' || c == '¥' || c == '$' || c == '฿'
}

fn is_identifier_char(c: char) -> bool {
    (c.is_alphanumeric() || c == '_' || is_currency_char(c)) && !is_exponent_char(c)
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            current: SourceCodePositition::start(),
            token_start: SourceCodePositition::start(),
        }
    }

    fn scan(&mut self) -> Result<Vec<Token>> {
        let mut tokens = vec![];
        while !self.at_end() {
            self.token_start = self.current.clone();
            if let Some(token) = self.scan_single_token()? {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".into(),
            span: self.current.to_single_character_span(),
        });

        Ok(tokens)
    }

    fn consume_stream_of_digits(
        &mut self,
        at_least_one_digit: bool,
        disallow_dot_and_digits_after_stream: bool,
    ) -> Result<()> {
        if at_least_one_digit && !self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Err(TokenizerError {
                kind: TokenizerErrorKind::ExpectedDigit {
                    character: self.peek(),
                },
                span: self.current.to_single_character_span(),
            });
        }

        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }

        if disallow_dot_and_digits_after_stream
            && self
                .peek()
                .map(|c| c.is_ascii_digit() || c == '.')
                .unwrap_or(false)
        {
            return Err(TokenizerError {
                kind: TokenizerErrorKind::UnexpectedCharacterInNumberLiteral(self.peek().unwrap()),
                span: self.current.to_single_character_span(),
            });
        }

        Ok(())
    }

    fn scientific_notation(&mut self) -> Result<()> {
        if self
            .peek2()
            .map(|c| c.is_ascii_digit() || c == '+' || c == '-')
            .unwrap_or(false)
            && (self.match_char('e') || self.match_char('E'))
        {
            let _ = self.match_char('+') || self.match_char('-');

            self.consume_stream_of_digits(true, true)?;
        }

        Ok(())
    }

    fn scan_single_token(&mut self) -> Result<Option<Token>> {
        static KEYWORDS: OnceLock<HashMap<&'static str, TokenKind>> = OnceLock::new();
        let keywords = KEYWORDS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("per", TokenKind::Per);
            m.insert("to", TokenKind::Arrow);
            m.insert("let", TokenKind::Let);
            m.insert("fn", TokenKind::Fn);
            m.insert("dimension", TokenKind::Dimension);
            m.insert("unit", TokenKind::Unit);
            m.insert("use", TokenKind::Use);
            m.insert("long", TokenKind::Long);
            m.insert("short", TokenKind::Short);
            m.insert("both", TokenKind::Both);
            m.insert("none", TokenKind::None);
            m.insert("print", TokenKind::ProcedurePrint);
            m.insert("assert_eq", TokenKind::ProcedureAssertEq);
            m
        });

        if self.peek() == Some('#') {
            // skip over comment until newline
            loop {
                match self.peek() {
                    None => return Ok(None),
                    Some('\n') => break,
                    _ => {
                        self.advance();
                    }
                }
            }
        }

        let current_char = self.advance();

        let tokenizer_error = |position: &SourceCodePositition, kind| -> Result<Option<Token>> {
            Err(TokenizerError {
                kind,
                span: position.to_single_character_span(),
            })
        };

        let kind = match current_char {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '<' => TokenKind::LeftAngleBracket,
            '>' => TokenKind::RightAngleBracket,
            '0' if self
                .peek()
                .map(|c| c == 'x' || c == 'o' || c == 'b')
                .unwrap_or(false) =>
            {
                let (base, is_digit_in_base): (_, Box<dyn Fn(char) -> bool>) =
                    match self.peek().unwrap() {
                        'x' => (16, Box::new(|c| c.is_ascii_hexdigit())),
                        'o' => (8, Box::new(|c| ('0'..='7').contains(&c))),
                        'b' => (2, Box::new(|c| c == '0' || c == '1')),
                        _ => unreachable!(),
                    };

                self.advance(); // skip over the x/o/b

                let mut has_advanced = false;
                while self.peek().map(&is_digit_in_base).unwrap_or(false) {
                    self.advance();
                    has_advanced = true;
                }

                if !has_advanced || self.peek().map(is_identifier_char).unwrap_or(false) {
                    return tokenizer_error(
                        &self.current,
                        TokenizerErrorKind::ExpectedDigitInBase {
                            base,
                            character: self.peek(),
                        },
                    );
                }

                TokenKind::IntegerWithBase(base)
            }
            c if c.is_ascii_digit() => {
                self.consume_stream_of_digits(false, false)?;

                // decimal part
                if self.match_char('.') {
                    self.consume_stream_of_digits(false, true)?;
                }

                self.scientific_notation()?;

                TokenKind::Number
            }
            '.' => {
                self.consume_stream_of_digits(true, true)?;
                self.scientific_notation()?;

                TokenKind::Number
            }
            ' ' | '\t' | '\r' => {
                return Ok(None);
            }
            '\n' => TokenKind::Newline,
            '*' if self.match_char('*') => TokenKind::Power,
            '+' => TokenKind::Plus,
            '*' | '·' | '×' => TokenKind::Multiply,
            '/' if self.match_char('/') => TokenKind::PostfixApply,
            '/' => TokenKind::Divide,
            '÷' => TokenKind::Divide,
            '%' => TokenKind::Modulo,
            '^' => TokenKind::Power,
            ',' => TokenKind::Comma,
            '=' => TokenKind::Equal,
            ':' if self.match_char(':') => TokenKind::ColonColon,
            ':' => TokenKind::Colon,
            '@' => TokenKind::At,
            '→' | '➞' => TokenKind::Arrow,
            '-' if self.match_char('>') => TokenKind::Arrow,
            '-' => TokenKind::Minus,
            '⁻' => {
                let c = self.peek();
                if c.map(is_exponent_char).unwrap_or(false) {
                    self.advance();
                    TokenKind::UnicodeExponent
                } else {
                    return tokenizer_error(
                        &self.current,
                        TokenizerErrorKind::UnexpectedCharacterInNegativeExponent { character: c },
                    );
                }
            }
            '¹' | '²' | '³' | '⁴' | '⁵' => TokenKind::UnicodeExponent,
            '°' => TokenKind::Identifier, // '°' is not an alphanumeric character, so we treat it as a special case here
            '"' => {
                while self.peek().map(|c| c != '"').unwrap_or(false) {
                    self.advance();
                }

                if self.match_char('"') {
                    TokenKind::String
                } else {
                    return tokenizer_error(
                        &self.token_start,
                        TokenizerErrorKind::UnterminatedString,
                    );
                }
            }
            '…' => TokenKind::Ellipsis,
            c if is_identifier_char(c) => {
                while self.peek().map(is_identifier_char).unwrap_or(false) {
                    self.advance();
                }

                if self.peek().map(|c| c == '.').unwrap_or(false) {
                    return tokenizer_error(
                        &self.current,
                        TokenizerErrorKind::UnexpectedCharacterInIdentifier(self.peek().unwrap()),
                    );
                }

                if let Some(kind) = keywords.get(self.lexeme().as_str()) {
                    *kind
                } else {
                    TokenKind::Identifier
                }
            }
            c => {
                return tokenizer_error(
                    &self.token_start,
                    TokenizerErrorKind::UnexpectedCharacter { character: c },
                );
            }
        };

        let token = Some(Token {
            kind,
            lexeme: self.lexeme(),
            span: self.token_start.to_single_character_span(),
        });

        if kind == TokenKind::Newline {
            self.current.line += 1;
            self.current.position = 1;
        }

        Ok(token)
    }

    fn lexeme(&self) -> String {
        self.input[self.token_start.index..self.current.index]
            .iter()
            .collect()
    }

    fn advance(&mut self) -> char {
        let c = self.input[self.current.index];
        self.current.index += 1;
        self.current.byte += c.len_utf8();
        self.current.position += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.current.index).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.input.get(self.current.index + 1).copied()
    }

    fn match_char(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at_end(&self) -> bool {
        self.current.index >= self.input.len()
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokenizer = Tokenizer::new(input);
    tokenizer.scan()
}

#[cfg(test)]
fn tokenize_reduced(input: &str) -> Vec<(String, TokenKind, (usize, usize))> {
    tokenize(input)
        .unwrap()
        .iter()
        .map(|token| {
            (
                token.lexeme.to_string(),
                token.kind,
                (token.span.position.line, token.span.position.position),
            )
        })
        .collect()
}

#[test]
fn test_tokenize_basic() {
    use TokenKind::*;

    assert_eq!(
        tokenize_reduced("  12 + 34  "),
        [
            ("12".to_string(), Number, (1, 3)),
            ("+".to_string(), Plus, (1, 6)),
            ("34".to_string(), Number, (1, 8)),
            ("".to_string(), Eof, (1, 12))
        ]
    );

    assert_eq!(
        tokenize_reduced("1 2"),
        [
            ("1".to_string(), Number, (1, 1)),
            ("2".to_string(), Number, (1, 3)),
            ("".to_string(), Eof, (1, 4))
        ]
    );

    assert_eq!(
        tokenize_reduced("12 × (3 - 4)"),
        [
            ("12".to_string(), Number, (1, 1)),
            ("×".to_string(), Multiply, (1, 4)),
            ("(".to_string(), LeftParen, (1, 6)),
            ("3".to_string(), Number, (1, 7)),
            ("-".to_string(), Minus, (1, 9)),
            ("4".to_string(), Number, (1, 11)),
            (")".to_string(), RightParen, (1, 12)),
            ("".to_string(), Eof, (1, 13))
        ]
    );

    assert_eq!(
        tokenize_reduced("foo to bar"),
        [
            ("foo".to_string(), Identifier, (1, 1)),
            ("to".to_string(), Arrow, (1, 5)),
            ("bar".to_string(), Identifier, (1, 8)),
            ("".to_string(), Eof, (1, 11))
        ]
    );

    assert_eq!(
        tokenize_reduced("1 -> 2"),
        [
            ("1".to_string(), Number, (1, 1)),
            ("->".to_string(), Arrow, (1, 3)),
            ("2".to_string(), Number, (1, 6)),
            ("".to_string(), Eof, (1, 7))
        ]
    );

    assert_eq!(
        tokenize_reduced("45°"),
        [
            ("45".to_string(), Number, (1, 1)),
            ("°".to_string(), Identifier, (1, 3)),
            ("".to_string(), Eof, (1, 4))
        ]
    );

    assert_eq!(
        tokenize_reduced("1+2\n42"),
        [
            ("1".to_string(), Number, (1, 1)),
            ("+".to_string(), Plus, (1, 2)),
            ("2".to_string(), Number, (1, 3)),
            ("\n".to_string(), Newline, (1, 4)),
            ("42".to_string(), Number, (2, 1)),
            ("".to_string(), Eof, (2, 3))
        ]
    );

    assert_eq!(
        tokenize_reduced("\"foo\""),
        [
            ("\"foo\"".to_string(), String, (1, 1)),
            ("".to_string(), Eof, (1, 6))
        ]
    );

    assert!(tokenize("~").is_err());
}

#[test]
fn test_is_currency_char() {
    assert!(is_currency_char('€'));
    assert!(is_currency_char('$'));
    assert!(is_currency_char('¥'));
    assert!(is_currency_char('£'));
    assert!(is_currency_char('฿'));
    assert!(is_currency_char('₿'));

    assert!(!is_currency_char('E'));
}
