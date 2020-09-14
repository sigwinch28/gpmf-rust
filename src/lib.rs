use nom::bytes::complete::take;
use nom::IResult;

use std::borrow::Cow;

use encoding_rs::mem::decode_latin1;

pub type Key<'a> = Cow<'a, str>;

fn parse_fourcc<'a>(i: &'a [u8]) -> IResult<&'a [u8], Key<'a>> {
    let (i, bytes) = take(4u8)(i)?;
    Ok((i, decode_latin1(bytes)))
}

pub mod raw {
    use super::{parse_fourcc, Key};

    use nom::bytes::complete::take;
    use nom::character::complete::one_of;
    use nom::combinator::{all_consuming, map};
    use nom::multi::many0;
    use nom::number::complete::{be_u16, be_u8};
    use nom::IResult;

    #[derive(Debug)]
    pub enum Value<'a> {
        Raw(&'a [u8]),
        Nested(Vec<Packet<'a>>),
    }

    #[derive(Debug)]
    pub struct Packet<'a> {
        pub key: Key<'a>,
        pub r#type: char,
        pub size: usize,
        pub repeat: usize,
        pub value: Value<'a>,
    }

    impl<'a> Packet<'a> {
        pub fn parse(i: &'a [u8]) -> IResult<&[u8], Packet<'a>> {
            let (i, key) = parse_fourcc(i)?;
            let (i, r#type) = one_of("bBcdfFgjJlLqQsSU?\0")(i)?;
            let (i, size) = map(be_u8, |size| size as usize)(i)?;
            let (i, repeat) = map(be_u16, |repeat| repeat as usize)(i)?;

            let length = (size as usize) * (repeat as usize);
            let padding = (std::mem::size_of::<u32>() - (length % std::mem::size_of::<u32>()))
                % std::mem::size_of::<u32>();

            let (i, raw_value) = take(length)(i)?;
            let (i, _padding) = take(padding)(i)?;

            let value = match r#type {
                '\0' => {
                    let (_, packets) = parse_stream(raw_value)?;
                    Value::Nested(packets)
                }
                _ => Value::Raw(raw_value),
            };

            let packet = Packet {
                key,
                r#type,
                size,
                repeat,
                value,
            };

            Ok((i, packet))
        }
    }

    pub fn parse_stream<'a>(i: &'a [u8]) -> IResult<&[u8], Vec<Packet<'a>>> {
        all_consuming(many0(Packet::parse))(i)
    }
}
// pub mod parser;
