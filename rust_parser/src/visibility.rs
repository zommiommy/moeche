use rust_lexer::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility{
    Private,
    Public,
    PublicCrate, 
    PublicSuper,
}
