mod chars_constants;
pub use chars_constants::*;

pub trait Parse<T> {
    fn inner_parse(&mut self) -> Option<T>;
}

pub struct Data<'a> {
    data: &'a str,
    line_number: usize,
}

impl<'a> Data<'a> {
    #[inline]
    pub fn parse<T>(&mut self) -> Option<T> 
    where
        Data<'a>: Parse<T> 
    {
        self.skip_white_space();
        Parse::<T>::inner_parse(self)
    }

    #[inline]
    pub fn get_current_line_number(&self) -> usize {
        self.line_number
    }

    #[inline]
    pub fn split_at_white_space(&mut self) -> &str {
        let (skipped, ptr) = self.data.split_once(WHITE_SPACE)
            .unwrap_or(("", self.data));

        self.data = ptr;
        skipped
    }

    #[inline]
    pub fn skip_white_space(&mut self) {
        let skipped = self.split_at_white_space();
        self.line_number += skipped.chars().filter(|x| *x == '\n').count();

    }
}

pub struct Identifier<'a>(&'a str);

impl <'a> Parse<Identifier> for Data<'a> {  
    #[inline]
    fn inner_parse(&mut self) -> Option<Identifier> {
        let mut result = String::new();

        let mut ptr = self.data;
        while let Some(c) = ptr.chars().nth(0) {
            if !IDENTIFIER_ALPHABET.contains(c) {
                break
            }
            result.push(c);
            ptr = &ptr[1..];
        }

        if result.is_empty() {
            None
        } else {
            self.data = ptr;
            Some(Identifier(result))
        }
    }
}

pub enum Literal<'a> {
    Integer(&'a str),
    Float(f64),
    String(String),
    RawString(String),
    Bytes(Vec<u8>),
    RawBytes(Vec<u8>),
}

pub enum Token<'a> {
    // structured tokens
    Identifier(Identifier<'a>),
    Lifetime(&'a str),
    Comment(&'a str),
    Literal(Literal<'a>),

    // Symbols
    OpenBraces,        // {
    CloseBraces,       // }
    OpenBraket,        // [
    CloseBraket,       // ]
    OpenParenthesis,   // (
    CloseParenthesis,  // )
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Caret,      // ^
    Not,        // !
    And,        // &
    Or,         // |
    AndAnd,     // &&
    OrOr,       // ||
    Shl,        // <<
    Shr,        // >>
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    CaretEq,    // ^=
    AndEq,      // &=
    OrEq,       // |=
    ShlEq,      // <<=
    ShrEq,      // >>=
    Eq,         // =
    EqEq,       // ==
    Ne,         // !=
    GtOrOpenAngular,   // >
    LtOrClosedAngular, // <
    Ge,         // >=
    Le,         // <=
    At,         // @
    Underscore, // _
    Dot,        // .
    DotDot,     // ..
    DotDotDot,  // ...
    DotDotEq,   // ..=
    Comma,      // ,
    Semi,       // ;
    Colon,      // :
    PathSep,    // ::
    RArrow,     // ->
    FatArrow,   // =>
    Pound,      // #
    Dollar,     // $
    Question,   // ?
    Shebang,    // #!
    Utf8Bom,    // \uFEFF
    EmptyTuple, // ()

    // strict keywords
    As,         // as
    Break,      // break
    Const,      // const
    Continue,   // continue
    Crate,      // crate
    Else,       // else
    Enum,       // enum
    Extern,     // extern
    False,      // false
    Fn,         // fn
    For,        // for
    If,         // if
    Impl,       // impl
    In,         // in
    Let,        // let
    Loop,       // loop
    Match,      // match
    Mod,        // mod
    Move,       // move
    Mut,        // mut
    Pub,        // pub
    Ref,        // ref
    Return,     // return
    SelfLowercase,   // self
    SelfCapitalized, // Self
    Static,     // static
    Struct,     // struct
    Trait,      // trait
    True,       // true
    Type,       // type
    Unsafe,     // unsafe
    Use,        // use
    Where,      // where
    While,      // while

    // strict keywords (2018 edition)
    Async,      // async
    Await,      // await
    Dyn,        // dyn

    // reserved keywords
    Abstract,   // abstract
    Become,     // become
    Box,        // box
    Do,         // do
    Final,      // final
    Macro,      // macro
    Override,   // override
    Priv,       // priv
    Typeof,     // typeof
    Unsized,    // unsized
    Virtual,    // virtual
    Yield,      // yield

    // reserved keywords (2018 edition)
    Try,        // try

    // weak keywords
    Union,      // union
}