#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use rust_lexer::Identifier;

mod visibility;

pub struct Module {
    pub file_path: String,
    pub module_doc: String,
    pub name: String,
    //pub uses: Vec<Use>,
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
    pub mods: BTreeMap<String, Module>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub visibility: Visibility,
    pub attributes: Vec<Attribute>,
    pub content: Vec<Identifier<'a>>,
}