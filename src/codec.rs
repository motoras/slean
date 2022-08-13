use std::io::{Read, Write};

use bincode::{deserialize_from, serialize_into};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rmp_serde::encode::write;
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{RemoteError, SleanError, SleanResult};

pub trait Codec<T> {
    fn read(reader: &mut impl Read) -> SleanResult<T>;
    fn write(data: &T, writer: &mut impl Write) -> SleanResult<()>;
}

pub struct TextCodec {}
impl Codec<String> for TextCodec {
    fn read(reader: &mut impl Read) -> SleanResult<String> {
        let mut res = String::new();
        reader.read_to_string(&mut res)?;
        Ok(res)
    }

    fn write(data: &String, writer: &mut impl Write) -> SleanResult<()> {
        writer
            .write_all(&data.as_bytes())
            .map_err(|err| SleanError::UnexpectedIoFailure(err))
    }
}

impl Codec<RemoteError> for TextCodec {
    fn read(reader: &mut impl Read) -> SleanResult<(i32, String)> {
        let code = reader
            .read_i32::<LittleEndian>()
            .map_err(|err| SleanError::UnexpectedIoFailure(err))?;
        let mut msg = String::new();
        reader.read_to_string(&mut msg)?;
        Ok((code, msg))
    }

    fn write((code, msg): &RemoteError, writer: &mut impl Write) -> SleanResult<()> {
        writer
            .write_i32::<LittleEndian>(*code)
            .map_err(|err| SleanError::UnexpectedIoFailure(err))?;
        writer
            .write_all(&msg.as_bytes())
            .map_err(|err| SleanError::UnexpectedIoFailure(err))
    }
}

pub struct MsgPackCodec {}

impl<T: DeserializeOwned + Serialize> Codec<T> for MsgPackCodec {
    fn read(reader: &mut impl Read) -> SleanResult<T> {
        from_read(reader).map_err(|err| SleanError::DecodingFailed(err.to_string()))
    }

    fn write(data: &T, buff: &mut impl Write) -> SleanResult<()> {
        write(buff, &data).map_err(|err| SleanError::EncodingFailed(err.to_string()))
    }
}

pub struct BincodeCodec {}

impl<T: DeserializeOwned + Serialize> Codec<T> for BincodeCodec {
    fn read(reader: &mut impl Read) -> SleanResult<T> {
        deserialize_from(reader).map_err(|err| SleanError::DecodingFailed(err.to_string()))
    }

    fn write(data: &T, buff: &mut impl Write) -> SleanResult<()> {
        serialize_into(buff, &data).map_err(|err| SleanError::EncodingFailed(err.to_string()))
    }
}
