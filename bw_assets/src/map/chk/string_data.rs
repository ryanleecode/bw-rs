use nom::{
    self,
    bytes::complete::{take, take_until},
    multi::count,
    number::complete::le_u16,
    sequence::preceded,
};

use std::cmp::max;

#[derive(Debug, Eq, PartialEq)]
pub struct StringData(Vec<Vec<u8>>);

impl StringData {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], StringData> {
        let (remaining, str_count) = le_u16(b)?;
        let (_, str_offsets) = count(le_u16, str_count as usize)(remaining)?;

        // number of bytes of this chunk.
        let mut size = 0;

        let mut str_data = vec![];
        for offset in str_offsets {
            let (_, s) = preceded(take(offset), take_until("\0"))(b)?;
            size = max(size, offset as u32 + s.len() as u32);
            str_data.push(s.to_vec());
        }

        // jump over the last null terminator if there are strings
        if str_count > 0 {
            size += 1
        }

        let (remaining, _) = take(size)(b)?;

        Ok((remaining, StringData(str_data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::mem::size_of;

    use byteorder::{LittleEndian, WriteBytesExt};

    #[test]
    fn it_parses_string_data() {
        let s1 = b"starcraft\0";
        let s2 = b"broodwar\0";

        let mut b: Vec<u8> = vec![];

        // string count
        b.write_u16::<LittleEndian>(2).unwrap();
        // s1 offset
        b.write_u16::<LittleEndian>((size_of::<u16>() * 3) as u16)
            .unwrap();
        // s2 offset
        b.write_u16::<LittleEndian>(((size_of::<u16>() * 3) + s1.len()) as u16)
            .unwrap();
        // write s1
        b.extend(s1);
        // write s2
        b.extend(s2);

        let expected_remaining_bytes: &[u8] = &[];
        let expected = (
            expected_remaining_bytes,
            StringData(
                // null terminator is removed
                vec![b"starcraft".to_vec(), b"broodwar".to_vec()],
            ),
        );

        assert_that(&StringData::parse(&b))
            .is_ok()
            .is_equal_to(expected);
    }
}
