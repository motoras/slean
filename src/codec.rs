use std::io::{Read, Write};

use bincode::{deserialize_from, serialize_into};
use rmp_serde::encode::write;
use rmp_serde::from_read;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{SleanError, SleanResult};

pub trait Codec<T> {
    fn read(reader: &mut impl Read) -> SleanResult<T>;
    fn write(data: &T, writer: &mut impl Write) -> SleanResult<()>;
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
