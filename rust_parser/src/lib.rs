#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use rust_lexer::{Identifier, Lexer, Symbol, Keyword, Peekable, LexerError, Token};

mod visibility;

pub enum ParserError<'a> {
    LexerError(LexerError<'a>),
    UnexpectedToken(Token<'a>),
}

impl<'a> From<LexerError<'a>> for ParserError<'a> {
    fn from(value: LexerError<'a>) -> Self {
        ParserError::LexerError(value)
    }
}

pub trait FromTokenStream: Sized {
    fn from_tokens_stream<'a, I, const BUFFER_SIZE: usize>(
        token_stream: &mut Peekable<Token<'a>, I, BUFFER_SIZE>
    ) 
    -> Option<Self>
    where
        I: Iterator<Item=Token<'a>>;
}

#[derive(Debug, Default)]
pub struct Module<'a> {
    pub file_path: String,
    pub module_doc: String,
    pub name: String,
    pub uses: Vec<Use<'a>>,
    //pub enums: Vec<Enum>,
    //pub structs: Vec<Struct>,
    //pub types: Vec<TypeDefinition>,
    //pub traits: Vec<TraitDefinition>,
    //pub consts: Vec<Const>,
    //pub statics: Vec<Static>,
    //pub impls: Vec<Impl>,
    //pub macros: Vec<Macro>,
    //pub macro_calls: Vec<MacroCall>,
    //pub functions: Vec<Function>,
    //pub externs: Vec<Extern>,
    pub mods: BTreeMap<String, Module<'a>>,
}

impl<'b> Module<'b> {
    pub fn from_tokens_stream<'a, I, const BUFFER_SIZE: usize>(
        token_stream: &mut Peekable<Token<'a>, I, BUFFER_SIZE>
    ) -> Result<Self, ParserError<'a>>
    where
        I: Iterator<Item=Token<'a>>,
    {
        let mut module = Module::default();

        while let Some(token) = token_stream.get(0) {
            if let Some(uze) = <Use<'a>>::from_token_stream(&mut token_stream) {
                module.uses.push(uze);
            }
            
            return Err(ParserError::UnexpectedToken(token));
        }

        Ok(module)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute<'a>(Vec<Token<'a>>);

impl<'b> FromTokenStream for Attribute<'b> {
    fn from_tokens_stream<'a, I, const BUFFER_SIZE: usize>(
        token_stream: &mut Peekable<Token<'a>, I, BUFFER_SIZE>
    ) -> Option<Self>
    where
        I: Iterator<Item=Token<'a>>,
    {
        if token_stream.get(0) != Some(Token::Symbol(Symbol::Pound)) 
            || token_stream.get(1) != Some(Token::Symbol(Symbol::OpenBraket)) 
            {
            return None;
        } 

        let mut tokens = Vec::new();
        while let Some(token) = token_stream.next() {
            if token == Token::Symbol(Symbol::CloseBraket) {
                return Some(Attribute(tokens));
            }
            tokens.push(token);
        }

        // mising closing braket
        None
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Use<'a> {
    pub visibility: Visibility,
    pub attributes: Vec<Attribute<'a>>,
    pub content: Vec<Identifier<'a>>,
}

impl<'b> FromTokenStream for Use<'b> {
    fn from_tokens_stream<'a, I, const BUFFER_SIZE: usize>(
        token_stream: &mut Peekable<Token<'a>, I, BUFFER_SIZE>
    ) -> Option<Self>
    where
        I: Iterator<Item=Token<'a>>,
    {
        let mut uze = Use::default();

        uze.visibility = Visibility::from_tokens_stream(token_stream)?;

        let next_token = token_stream.get(0)?;
        if next_token != Token::Keyword(Keyword::Use) {
            return None;
        }

        todo!();

        Some(uze)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility{
    Private,
    Public,
    PublicCrate, 
    PublicSuper,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private
    }
}

impl FromTokenStream for Visibility {
    fn from_tokens_stream<'a, I, const BUFFER_SIZE: usize>(
        token_stream: &mut Peekable<Token<'a>, I, BUFFER_SIZE>
    ) -> Option<Self>
    where
        I: Iterator<Item=Token<'a>>,
    {
        Some(match (token_stream.get(0)?, token_stream.get(1)?, token_stream.get(2)?, token_stream.get(3)?) {
            (Token::Keyword(Keyword::Pub), Token::Symbol(Symbol::OpenBraces), Token::Keyword(Keyword::Crate), Token::Symbol(Symbol::CloseBraces)) => {
                token_stream.consume(4);
                Visibility::PublicCrate
            },
            (Token::Keyword(Keyword::Pub), Token::Symbol(Symbol::OpenBraces), Token::Keyword(Keyword::Super), Token::Symbol(Symbol::CloseBraces)) => {
                token_stream.consume(4);
                Visibility::PublicSuper
            },
            (Token::Keyword(Keyword::Pub), _, _, _) => {
                token_stream.consume(1);
                Visibility::Public
            },    
            _ => Visibility::Private,
        })
    }
}
