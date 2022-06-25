#![no_std]

use core::convert::TryFrom;

extern crate alloc;

mod chars_constants;

mod keyword;
pub use keyword::*;
mod symbol;
pub use symbol::*;
mod literal;
pub use literal::*;
mod identifier;
pub use identifier::*;
mod comment;
pub use comment::*;

#[derive(Debug)]
pub enum LexerError<'a> {
    UnexpectedEndOfFile(Span),
    CannotTokenize{
        hint: &'a str,
        span:Span
    },
}

type Result<'a, T> = core::result::Result<T, LexerError<'a>>;

#[derive(Debug, Clone, Default)]
pub struct Span {
    line: usize,
    byte_offset: usize,
}

pub struct Lexer<'a> {
    original_data: &'a str,
    remaining_text: &'a str,
    span: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            original_data: data,
            remaining_text: data,
            span: Span::default(),
        }
    }

    #[inline]
    pub fn skip_white_space(&mut self) {
        while let Some(current_char) = self.remaining_text.chars().next() {
            if !chars_constants::is_whitespace(current_char) {
                break
            }
            if current_char == '\n' {
                self.span.line += 1;
            }
            self.remaining_text = &self.remaining_text[1..];
            self.span.byte_offset += current_char.len_utf8();
            
        }
    }

    pub fn get_next_token(&mut self) -> Result<Token<'a>> {
        self.skip_white_space();

        macro_rules! try_parse {
            ($type:ident) => {
                if let Ok(value) = $type::try_from(self.remaining_text) {
                    self.span.byte_offset += value.len();
                    let (_extra, rem) = self.remaining_text.split_at(value.len());
                    self.remaining_text = rem;
                    return Ok(Token::$type(value)) 
                }
            };
        }

        try_parse!(Comment);
        try_parse!(Literal);
        try_parse!(Symbol);

        if let Ok(identifier) = Identifier::try_from(self.remaining_text) {
            self.span.byte_offset += identifier.len();
            let (_extra, rem) = self.remaining_text.split_at(identifier.len());
            self.remaining_text = rem;

            return Ok(if let Ok(keyword) = Keyword::try_from(identifier) {
                Token::Keyword(keyword)
            } else {
                Token::Identifier(identifier)
            });
        }

        Err(LexerError::CannotTokenize{
            hint: &self.original_data[
                self.span.byte_offset.saturating_sub(10)
                ..
                self.original_data.len().min(self.span.byte_offset + 10)
            ],
            span:self.span.clone(),
        })
    }
}

#[derive(Debug)]
pub enum Token<'a> {
    Comment(Comment<'a>),
    Literal(Literal<'a>),
    Symbol(Symbol),
    Keyword(Keyword),
    Identifier(Identifier<'a>),
}