use alloc::string::{String, ToString};
use crate::chars_constants::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Identifier<'a>(pub(crate) &'a str);

impl<'a> AsRef<str> for Identifier<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> Identifier<'a> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<Identifier<'a>> for String {
    fn from(value: Identifier<'a>) -> Self {
        value.0.to_string()
    }
}

impl<'a> TryFrom<&'a str> for Identifier<'a> {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut chars = value.chars();
        let first_char = chars.next().ok_or(())?;
        // check if the first char is valid
        if !is_xid_start(first_char) && first_char != '_' {
            return Err(());
        }

        let mut index_length = first_char.len_utf8();

        // check if it's a raw identifier
        if let Some(second_char) = chars.next() {
            if !is_whitespace(second_char) && (is_xid_continue(second_char) || second_char == '#') {
                index_length += second_char.len_utf8();
            } else {
                return Ok(Identifier(&value[..index_length]));   
            }
        } else {
            return Ok(Identifier(&value[..index_length]));   
        }

        // check if it's a raw string
        if let Some(third_char) = chars.next() {
            // It's a raw string
            if third_char == '"' {
                return Err(());   
            }
            if is_whitespace(third_char) || !is_xid_continue(third_char) {
                return Ok(Identifier(&value[..index_length])); 
            } else {
                index_length += third_char.len_utf8();
            }
        } else {
            return Ok(Identifier(&value[..index_length]));   
        }


        // find the first non-identifier char so we know where to stop
        while let Some(current_char) = chars.next() {
            if is_whitespace(current_char) || !is_xid_continue(current_char) {
                break
            }

            index_length += current_char.len_utf8();
        }

        let (identifier, _) = value.split_at(index_length);

        Ok(Identifier(identifier))
    }
}

#[cfg(test)]
mod test_identifiers {
    use super::*;
    #[test]
    fn try_to_parse_identifiers() {
        assert_eq!(Err(()), Identifier::try_from("r#\"aaa\""));
        assert_eq!(Identifier("a"), Identifier::try_from("a (").unwrap());
        assert_eq!(Identifier("ab"), Identifier::try_from("ab (").unwrap());
        assert_eq!(Identifier("ra"), Identifier::try_from("ra(").unwrap());
        assert_eq!(Identifier("foo"), Identifier::try_from("foo (").unwrap());
        assert_eq!(Identifier("_identifier"), Identifier::try_from("_identifier (").unwrap());
        assert_eq!(Identifier("r#true"), Identifier::try_from("r#true (").unwrap());
        assert_eq!(Identifier("Москва"), Identifier::try_from("Москва(").unwrap());
        assert_eq!(Identifier("𫞎𫞔"), Identifier::try_from("𫞎𫞔 (").unwrap());
        //assert_eq!(Identifier("東京"), Identifier::try_from("東京(").unwrap());
    }
}