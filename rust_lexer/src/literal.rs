#[derive(Debug, PartialEq, Eq)]
pub enum Literal<'a> {
    Integer(&'a str),
    Float(&'a str),
    Char(&'a str),
    Byte(&'a str),
    String(&'a str),
    RawString(&'a str),
    Bytes(&'a str),
    RawBytes(&'a str),
}

impl<'a> Literal<'a> {
    pub fn len(&self) -> usize {
        use Literal::*;
        match self {
            Integer(raw) => raw.len(),
            Float(raw) => raw.len(),
            Char(raw) => raw.len() + 1 + 1,
            String(raw) => raw.len() + 1 + 1,
            RawString(raw) => raw.len() + 3 + 2,
            Bytes(raw) => raw.len() + 2 + 1,
            Byte(raw) => raw.len() + 2 + 1,
            RawBytes(raw) => raw.len() + 4 + 2,
        }
    }
}

fn get_literal_end<'a>(data: &'a str, opening_tag: &str, closing_tag: &str) -> Option<&'a str> {
    if !data.starts_with(opening_tag) {
        return None;
    }
    // skip the
    let (_, ptr) = data.split_at(opening_tag.len());

    let mut index = 0;
    let mut is_escaped = false;
    for char in ptr.chars() {
        if char == '\\' {
            is_escaped ^= true;
        } else {
            is_escaped = false;
        }

        if !is_escaped {
            if ptr[index..].starts_with(closing_tag) {
                return Some(&ptr[..index]);
            }
        }

        index += char.len_utf8();
    }
    // not closing tag
    None
}

impl<'a> TryFrom<&'a str> for Literal<'a> {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Some(literal) = get_literal_end(value, "br#\"", "\"#") {
            return Ok(Literal::RawBytes(literal));
        }
        if let Some(literal) = get_literal_end(value, "r#\"", "\"#") {
            return Ok(Literal::RawString(literal));
        }
        if let Some(literal) = get_literal_end(value, "b\"", "\"") {
            return Ok(Literal::Bytes(literal));
        }
        if let Some(literal) = get_literal_end(value, "b'", "'") {
            return Ok(Literal::Byte(literal));
        }
        if let Some(literal) = get_literal_end(value, "\"", "\"") {
            return Ok(Literal::String(literal));
        }
        if let Some(literal) = get_literal_end(value, "'", "'") {
            return Ok(Literal::Char(literal));
        }   

        // TODO!: integers

        Err(())
    }
}

#[cfg(test)]
mod test_literal {
    use super::*;
    #[test]
    fn try_to_parse_literal() {
        assert_eq!(Literal::try_from("'a' less").unwrap(), Literal::Char("a"));
        assert_eq!(Literal::try_from("b'a' less").unwrap(), Literal::Byte("a"));
        assert_eq!(Literal::try_from("\"aaaa\" less").unwrap(), Literal::String("aaaa"));
        assert_eq!(Literal::try_from("b\"bbbb\" less").unwrap(), Literal::Bytes("bbbb"));
        assert_eq!(Literal::try_from("r#\"abcd\"# less").unwrap(), Literal::RawString("abcd"));
        assert_eq!(Literal::try_from("br#\"ohmy\"# less").unwrap(), Literal::RawBytes("ohmy"));
    }
}