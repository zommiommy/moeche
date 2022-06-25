use crate::Identifier;

macro_rules! impl_keyword {
    ($($variant:ident => $value:literal,)*) => {
        
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Keyword {
    $(
        $variant, // $value
    )*
}

impl Keyword {
    pub const fn len(&self) -> usize {
        use Keyword::*;
        match self {
            $($variant => $value.len(),)*
        }
    }
}

impl From<Keyword> for &'static str {
    fn from(value: Keyword) -> &'static str {
        use Keyword::*;
        match value {
            $($variant => $value,)*
        }
    }
}

impl<'a> TryFrom<Identifier<'a>> for Keyword {
    type Error = ();
    fn try_from(value: Identifier<'a>) -> Result<Self, Self::Error> {
        use Keyword::*;
        Ok(match value.0 {
            $($value => $variant,)*
            _ => return Err(()),
        })
    }
}


#[cfg(test)]
mod test_keyword {
    use super::*;
    #[test]
    fn try_to_parse_keyword() {
        $(
            assert_eq!(
                Keyword::$variant,
                Keyword::try_from(Identifier::try_from($value).unwrap()).unwrap(),
            );
        )*
    }
}
    };
}

// List of keywords, thesere are ordered from longest to shortest (to have 
// proper parsing priority), and then alphabetically.
// Generated with:
// ```python
// print(
//     "\n".join(
//     "{} => \"{}\",".format(*x) 
//     for x in sorted((
//         re.findall(r"\s*(.+),\s*\/\/\s*(.+)", x)[0] 
//         for x in vals.split("\n")
//     ), key=lambda x: (-len(x[1]), x[1])))
// )
// ```
impl_keyword!(
    Abstract => "abstract",
    Continue => "continue",
    Override => "override",
    Unsized => "unsized",
    Virtual => "virtual",
    Become => "become",
    Extern => "extern",
    Return => "return",
    Static => "static",
    Struct => "struct",
    Typeof => "typeof",
    Unsafe => "unsafe",
    Async => "async",
    Await => "await",
    Break => "break",
    Const => "const",
    Crate => "crate",
    False => "false",
    Final => "final",
    Macro => "macro",
    Match => "match",
    Trait => "trait",
    Union => "union",
    Where => "where",
    While => "while",
    Yield => "yield",
    SelfCapitalized => "Self",
    Else => "else",
    Enum => "enum",
    Impl => "impl",
    Loop => "loop",
    Move => "move",
    Priv => "priv",
    SelfLowercase => "self",
    True => "true",
    Type => "type",
    Box => "box",
    Dyn => "dyn",
    For => "for",
    Let => "let",
    Mod => "mod",
    Mut => "mut",
    Pub => "pub",
    Ref => "ref",
    Try => "try",
    Use => "use",
    As => "as",
    Do => "do",
    Fn => "fn",
    If => "if",
    In => "in",
);