use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, Serializer, Token, ValidationError,
};
use msgpack_value::Value;
pub use msgpack_value::{Bin, Ext, Int, Str};

#[doc(hidden)]
pub fn serialize<S: Serialize>(x: &S) -> Value {
    let buf = crate::serialize(x);
    crate::deserialize(&buf).unwrap()
}

#[doc(hidden)]
pub fn deserialize<D: Deserialize>(value: Value) -> Result<D, DeserializeError> {
    let buf = crate::serialize(value);
    crate::deserialize::<D>(&buf)
}

/// A special type for serializing and deserializing the `nil` object.
///
/// In our data model `()` does not represent the `nil` object because `()` should be zero-byte but `nil` has a size.
/// When you want to serialize or deserialize `nil` use this type instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nil;

impl Serialize for Nil {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_nil()
    }
}

impl Deserialize for Nil {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        let token = deserializer.deserialize_token()?;
        if token != Token::Nil {
            return Err(ValidationError.into());
        }
        Ok(Self)
    }
}

/// A special type used to deserialize any object and discard it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Any;

impl Deserialize for Any {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        let mut count = 1;
        while count > 0 {
            count -= 1;
            match deserializer.deserialize_token()? {
                Token::Nil
                | Token::Bool(_)
                | Token::Int(_)
                | Token::F32(_)
                | Token::F64(_)
                | Token::Str(_)
                | Token::Bin(_)
                | Token::Ext(_) => {}
                Token::Array(len) => {
                    count += len;
                }
                Token::Map(len) => {
                    count += len * 2;
                }
            }
        }
        Ok(Any)
    }
}

/// A special type used to serialize and deserialize the empty map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Empty {}

impl Serialize for Empty {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_map(0)
    }
}

impl Deserialize for Empty {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        let token = deserializer.deserialize_token()?;
        if token != Token::Map(0) {
            return Err(ValidationError.into());
        }
        Ok(Self {})
    }
}
