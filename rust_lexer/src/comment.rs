

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comment<'a> {
    LineComment(&'a str),   // //
    BlockComment(&'a str),  // /* */
    InnerLineDoc(&'a str),  // //!
    InnerBlockDoc(&'a str), // /*! */
    OuterLineDoc(&'a str),  // ///
    OuterBlockDoc(&'a str), // /** */
}

impl<'a> Comment<'a> {
    pub fn len(&self) -> usize {
        use Comment::*;
        match self {
            LineComment(x)   => x.len() + 2 + 1,
            BlockComment(x)  => x.len() + 2 + 2,
            InnerLineDoc(x)  => x.len() + 3 + 1,
            InnerBlockDoc(x) => x.len() + 2 + 2,
            OuterLineDoc(x)  => x.len() + 3 + 1,
            OuterBlockDoc(x) => x.len() + 3 + 2,
        }
    }
}

fn find_matching(data: &str) -> Result<usize, ()> {
    let mut index = 0;
    loop {
        if data[index..].starts_with("/**") {
            index += find_matching(&data[index + 3..])? + 3 + 2;
            continue;
        }

        if data[index..].starts_with("/*!") {
            index += find_matching(&data[index + 3..])? + 3 + 2;
            continue;
        }

        if data[index..].starts_with("/*") {
            index += find_matching(&data[index + 2..])? + 2 + 2;
            continue;
        }

        if data[index..].starts_with("*/") {
            return Ok(index);
        }

        let char = data[index..].chars().next().ok_or(())?;
        index += char.len_utf8();
    }
}

impl<'a> TryFrom<&'a str> for Comment<'a> {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.starts_with("/**") {
            let res_index = find_matching(&value[3..])?;
            return Ok(Comment::OuterBlockDoc(&value[3..3 + res_index]));
        }

        if value.starts_with("/*!") {
            let res_index = find_matching(&value[3..])?;
            return Ok(Comment::InnerBlockDoc(&value[3..3 + res_index]));
        }

        if value.starts_with("/*") {
            let res_index = find_matching(&value[2..])?;
            return Ok(Comment::BlockComment(&value[2..2 + res_index]));
        }

        if value.starts_with("///") {
            let (comment, _) = value.split_once("\n").unwrap_or((value, ""));
            return Ok(Comment::OuterLineDoc(&comment[3..]));
        }

        if value.starts_with("//!") {
            let (comment, _) = value.split_once("\n").unwrap_or((value, ""));
            return Ok(Comment::InnerLineDoc(&comment[3..]));
        }

        if value.starts_with("//") {
            let (comment, _) = value.split_once("\n").unwrap_or((value, ""));
            return Ok(Comment::LineComment(&comment[2..]));
        }

        Err(())
    }
}


#[cfg(test)]
mod test_commnet {
    use super::*;
    #[test]
    fn try_to_parse_commnet() {
        assert_eq!(Comment::try_from("// a // less").unwrap(), Comment::LineComment(" a // less"));
        assert_eq!(Comment::try_from("// a // less\naaaaaaaa\n\n").unwrap(), Comment::LineComment(" a // less"));
        assert_eq!(Comment::try_from("//! 'a'/// less").unwrap(), Comment::InnerLineDoc(" 'a'/// less"));
        assert_eq!(Comment::try_from("/// 'a'/// less").unwrap(), Comment::OuterLineDoc(" 'a'/// less"));

        assert_eq!(Comment::try_from("/* \n\n\n*/ a // less").unwrap(), Comment::BlockComment(" \n\n\n"));
        assert_eq!(Comment::try_from("/*! \n\n\n*/ a // less").unwrap(), Comment::InnerBlockDoc(" \n\n\n"));
        assert_eq!(Comment::try_from("/** \n\n\n*/ a // less").unwrap(), Comment::OuterBlockDoc(" \n\n\n"));

        assert_eq!(Comment::try_from("/* /*! /** /* */ */ */ */a // less").unwrap(), Comment::BlockComment(" /*! /** /* */ */ */ "));
        assert_eq!(Comment::try_from("/** /*! /** /* */ */ */ */a // less").unwrap(), Comment::OuterBlockDoc(" /*! /** /* */ */ */ "));
        assert_eq!(Comment::try_from("/*! /*! /** /* */ */ */ */a // less").unwrap(), Comment::InnerBlockDoc(" /*! /** /* */ */ */ "));
    }
}