use core::convert::TryFrom;

macro_rules! impl_symbol {
    ($($variant:ident => $value:literal,)*) => {
        
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Symbol{
    $(
        $variant, // $value
    )*
}

impl Symbol {
    pub const fn len(&self) -> usize {
        use Symbol::*;
        match self {
            $($variant => $value.len(),)*
        }
    }
}

impl From<Symbol> for &'static str {
    fn from(value: Symbol) -> &'static str {
        use Symbol::*;
        match value {
            $($variant => $value,)*
        }
    }
}

impl<'a> TryFrom<&'a str> for Symbol {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        use Symbol::*;
        Ok(match value {
            $(x if x.starts_with($value) => $variant,)*
            _ => return Err(()),
        })
    }
}


#[cfg(test)]
mod test_symbols {
    use super::*;
    #[test]
    fn try_to_parse_symbols() {
        $(
            assert_eq!(
                Symbol::$variant,
                Symbol::try_from(concat!($value, "123")).unwrap(),
            );
        )*
    }
}
    };
}

impl_symbol!(
    DotDotDot => "...",
    DotDotEq => "..=",
    ShlEq => "<<=",
    ShrEq => ">>=",

    AndAnd => "&&",
    OrOr => "||",
    Shl => "<<",
    Shr => ">>",
    PlusEq => "+=",
    MinusEq => "-=",
    StarEq => "*=",
    SlashEq => "/=",
    PercentEq => "%=",
    CaretEq => "^=",
    AndEq => "&=",
    OrEq => "|=",
    EqEq => "==",
    Ne => "!=",
    Ge => ">=",
    Le => "<=",
    DotDot => "..",
    RArrow => "->",
    FatArrow => "=>",
    Shebang => "#!",
    Utf8Bom => "\u{FEFF}",
    EmptyTuple => "()",
    PathSep => "::",

    OpenBraces => "{",
    CloseBraces => "}",
    OpenBraket => "[",
    CloseBraket => "]",
    OpenParenthesis => "(",
    CloseParenthesis => ")",
    Plus => "+",
    Minus => "-",
    Star => "*",
    Slash => "/",
    Percent => "%",
    Caret => "^",
    Not => "!",
    And => "&",
    Or => "|",
    Eq => "=",
    GtOrOpenAngular => ">",
    LtOrClosedAngular => "<",
    At => "@",
    Underscore => "_",
    Dot => ".",
    Comma => ",",
    Semi => ";",
    Colon => ":",
    Pound => "#",
    Dollar => "$",
    Question => "?",
);