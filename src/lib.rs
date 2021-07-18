//! _msgpack-schema_ is a schema language for describing data formats encoded in MessagePack.
//! It provides two derive macros `Serialize` and `Deserialize` that allow you to transcode MessagePack binary data to/from Rust data structures in a type-directed way.
//!
//! ```rust
//! use msgpack_schema::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct Human {
//!     #[tag = 0]
//!     name: String,
//!     #[tag = 2]
//!     #[optional]
//!     age: Option<u32>,
//! }
//! ```
//!
//! ## Behaviours of serializers and deserializers
//!
//! ### Some general rules
//!
//! - The deserializer ignores irrelevant key-value pairs in MsgPack map objects.
//! - MsgPack map objects must not have duplicate keys.
//! - `Option<T>` is roughly equal to declaring `T | null` in TypeScript. Deserializer interprets `nil` as `None` whatever `T` is. So `Option<Option<T>>` is the same as `Option<T>` (unless used together with `#[optional]`.)
//!
//! ### Structs with named fields
//!
//! Structs with named fields will be serialized into a MsgPack map object whose keys are fixints specified by `#[tag]` attributes.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! struct S {
//!     #[tag = 0]
//!     foo: u32,
//!     #[tag = 1]
//!     bar: String,
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! S { foo: 42, bar: "hello".to_owned() }
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! { 0: 42, 1: "hello" }
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! Fields in named structs may be tagged with `#[optional]`.
//!
//! - The tagged field must be of type `Option<T>`.
//! - On serialization, the key-value pair will not be included in the result map object when the field data contains `None`.
//! - On deserialization, the field of the result struct will be filled with `None` when the given MsgPack map object contains no corresponding key-value pair.
//!
//! The `#[flatten]` attribute is used to factor out a single definition of named struct into multiple ones.
//!
//! ```
//! # use msgpack_schema::*;
//! #[derive(Serialize)]
//! struct S1 {
//!     #[tag = 1]
//!     x: u32,
//! }
//!
//! #[derive(Serialize)]
//! struct S2 {
//!     #[flatten]
//!     s1: S1,
//!     #[tag = 2]
//!     y: u32,
//! }
//!
//! #[derive(Serialize)]
//! struct S3 {
//!     #[tag = 1]
//!     x: u32,
//!     #[tag = 2]
//!     y: u32,
//! }
//!
//! assert_eq!(serialize(S2 { s1: S1 { x: 42 }, y: 43, }), serialize(S3 { x: 42, y: 43 }));
//! ```
//!
//! ### Untagged structs with named fields
//!
//! Structs with named fields may be attached `#[untagged]`.
//! Untagged structs are serialized into an array and will not contain tags.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! #[untagged]
//! struct S {
//!     foo: u32,
//!     bar: String,
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! S { foo: 42, bar: "hello".to_owned() }
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! [ 42, "hello" ]
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Newtype structs
//!
//! Tuple structs with only one element are treated transparently.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! struct S(u32)
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! S(42)
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! 42
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Unit structs and empty tuple structs
//!
//! Serialization and deserialization of unit structs and empty tuple structs are currently unsupported.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! struct S
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! S
//! ```
//!
//! </td>
//! <td>
//!
//! UNSUPPORTED
//!
//! </td>
//! </tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! struct S()
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! S()
//! ```
//!
//! </td>
//! <td>
//!
//! UNSUPPORTED
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Unit variants and empty tuple variants
//!
//! Unit variants and empty tuple variants are serialized into a single fixint whose value is determined by the tag.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! enum E {
//!     #[tag = 3]
//!     Foo
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! E::Foo
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! 3
//! ```
//!
//! </td>
//! </tr>
//!
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! enum E {
//!     #[tag = 3]
//!     Foo()
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! E::Foo()
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! 3
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Newtype variants
//!
//! Newtype variants (one-element tuple variants) are serialized into an array of the tag and the inner value.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! enum E {
//!     #[tag = 3]
//!     Foo(u32)
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! E::Foo(42)
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! [ 3, 42 ]
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!
//! ### Untagged variants
//!
//! Enums may be attached `#[untagged]` when all variants are newtype variants.
//! Serializing untagged variants results in the same data layout as the inner type.
//! The deserializer deserializes into an untagged enum type by trying deserization one by one from the first variant to the last.
//!
//! <table>
//! <tr>
//! <th>
//! schema
//! </th>
//! <th>
//! Rust
//! </th>
//! <th>
//! MessagePack
//! </th>
//! </tr>
//! <tr>
//! <tr>
//! <td>
//!
//! ```rust,ignore
//! #[derive(Serialize, Deserialize)]
//! #[untagged]
//! enum Animal {
//!     Foo(String),
//!     Bar(u32),
//! }
//! ```
//!
//! </td>
//! <td>
//!
//! ```rust,ignore
//! E::Bar(42)
//! ```
//!
//! </td>
//! <td>
//!
//! ```js
//! 42
//! ```
//!
//! </td>
//! </tr>
//! </table>
//!

pub mod value;
use byteorder::BigEndian;
use byteorder::{self, ReadBytesExt};
pub use msgpack_schema_impl::*;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{self, Write};
use thiserror::Error;
use value::{Bin, Ext, Int, Nil, Str};

pub struct Serializer {
    w: Vec<u8>,
}

impl Serializer {
    fn new() -> Self {
        Self { w: vec![] }
    }
    fn into_inner(self) -> Vec<u8> {
        self.w
    }

    pub fn serialize_nil(&mut self) {
        rmp::encode::write_nil(&mut self.w).unwrap()
    }
    pub fn serialize_bool(&mut self, v: bool) {
        rmp::encode::write_bool(&mut self.w, v).unwrap()
    }
    pub fn serialize_int(&mut self, v: Int) {
        if let Ok(v) = i64::try_from(v) {
            rmp::encode::write_sint(&mut self.w, v).unwrap();
        } else {
            rmp::encode::write_uint(&mut self.w, u64::try_from(v).unwrap()).unwrap();
        }
    }
    pub fn serialize_f32(&mut self, v: f32) {
        rmp::encode::write_f32(&mut self.w, v).unwrap();
    }
    pub fn serialize_f64(&mut self, v: f64) {
        rmp::encode::write_f64(&mut self.w, v).unwrap();
    }
    pub fn serialize_str(&mut self, v: &[u8]) {
        rmp::encode::write_str_len(&mut self.w, v.len() as u32).unwrap();
        self.w.write_all(v).unwrap();
    }
    pub fn serialize_bin(&mut self, v: &[u8]) {
        rmp::encode::write_bin(&mut self.w, v).unwrap();
    }
    pub fn serialize_array(&mut self, len: u32) {
        rmp::encode::write_array_len(&mut self.w, len).unwrap();
    }
    pub fn serialize_map(&mut self, len: u32) {
        rmp::encode::write_map_len(&mut self.w, len).unwrap();
    }
    pub fn serialize_ext(&mut self, tag: i8, data: &[u8]) {
        rmp::encode::write_ext_meta(&mut self.w, data.len() as u32, tag).unwrap();
        self.w.write_all(data).unwrap();
    }

    pub fn serialize<S: Serialize>(&mut self, s: S) {
        S::serialize(&s, self)
    }
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut Serializer);
}

impl<T: Serialize> Serialize for &T {
    fn serialize(&self, serializer: &mut Serializer) {
        T::serialize(*self, serializer)
    }
}

impl Serialize for bool {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_bool(*self)
    }
}

impl Serialize for Int {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(*self)
    }
}

impl Serialize for u8 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u16 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u32 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u64 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i8 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i16 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i32 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i64 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for f32 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_f32(*self)
    }
}

impl Serialize for f64 {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_f64(*self)
    }
}

impl Serialize for Str {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_str(&self.0)
    }
}

impl Serialize for str {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_str(self.as_bytes())
    }
}

impl Serialize for String {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_str(self.as_bytes())
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, serializer: &mut Serializer) {
        match self {
            Some(v) => serializer.serialize(v),
            None => serializer.serialize_nil(),
        }
    }
}

impl<T: Serialize> Serialize for [T] {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_array(self.len() as u32);
        for x in self {
            serializer.serialize(x);
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize_array(self.len() as u32);
        for x in self {
            serializer.serialize(x);
        }
    }
}

#[doc(hidden)]
pub trait StructSerialize: Serialize {
    fn count_fields(&self) -> u32;
    fn serialize_fields(&self, serializer: &mut Serializer);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Nil,
    Bool(bool),
    Int(Int),
    F32(f32),
    F64(f64),
    Str(Str),
    Bin(Bin),
    Array(u32),
    Map(u32),
    Ext(Ext),
}

#[derive(Debug, Error)]
#[error("invalid input")]
pub struct InvalidInputError;

trait ReadExt: ReadBytesExt {
    fn read_to_vec(&mut self, len: usize) -> Result<Vec<u8>, io::Error> {
        let mut buf = vec![];
        buf.resize(len, 0);
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<R: io::Read> ReadExt for R {}

#[derive(Clone, Copy)]
pub struct Deserializer<'a> {
    r: &'a [u8],
}

impl<'a> Deserializer<'a> {
    fn new(r: &'a [u8]) -> Self {
        Self { r }
    }

    pub fn deserialize_token(&mut self) -> Result<Token, InvalidInputError> {
        let token = match rmp::decode::read_marker(&mut self.r).map_err(|_| InvalidInputError)? {
            rmp::Marker::Null => Token::Nil,
            rmp::Marker::True => Token::Bool(true),
            rmp::Marker::False => Token::Bool(false),
            rmp::Marker::FixPos(v) => Token::Int(Int::from(v)),
            rmp::Marker::FixNeg(v) => Token::Int(Int::from(v)),
            rmp::Marker::U8 => {
                Token::Int(Int::from(self.r.read_u8().map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::U16 => Token::Int(Int::from(
                self.r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::U32 => Token::Int(Int::from(
                self.r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::U64 => Token::Int(Int::from(
                self.r
                    .read_u64::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::I8 => {
                Token::Int(Int::from(self.r.read_i8().map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::I16 => Token::Int(Int::from(
                self.r
                    .read_i16::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::I32 => Token::Int(Int::from(
                self.r
                    .read_i32::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::I64 => Token::Int(Int::from(
                self.r
                    .read_i64::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            )),
            rmp::Marker::F32 => Token::F32(
                self.r
                    .read_f32::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            ),
            rmp::Marker::F64 => Token::F64(
                self.r
                    .read_f64::<BigEndian>()
                    .map_err(|_| InvalidInputError)?,
            ),
            rmp::Marker::FixStr(len) => {
                let len = len as usize;
                Token::Str(Str(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Str8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                Token::Str(Str(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Str16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                Token::Str(Str(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Str32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                Token::Str(Str(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Bin8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                Token::Bin(Bin(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Bin16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                Token::Bin(Bin(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::Bin32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                Token::Bin(Bin(self
                    .r
                    .read_to_vec(len)
                    .map_err(|_| InvalidInputError)?))
            }
            rmp::Marker::FixArray(len) => Token::Array(len as u32),
            rmp::Marker::Array16 => Token::Array(
                self.r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as u32,
            ),
            rmp::Marker::Array32 => Token::Array(
                self.r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as u32,
            ),
            rmp::Marker::FixMap(len) => Token::Map(len as u32),
            rmp::Marker::Map16 => Token::Map(
                self.r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as u32,
            ),
            rmp::Marker::Map32 => Token::Map(
                self.r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as u32,
            ),
            rmp::Marker::FixExt1 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(1).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::FixExt2 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(2).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::FixExt4 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(4).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::FixExt8 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(8).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::FixExt16 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(16).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::Ext8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(len).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::Ext16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(len).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::Ext32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                Token::Ext(Ext {
                    r#type: tag,
                    data: self.r.read_to_vec(len).map_err(|_| InvalidInputError)?,
                })
            }
            rmp::Marker::Reserved => return Err(InvalidInputError),
        };
        Ok(token)
    }

    pub fn deserialize<D: Deserialize>(&mut self) -> Result<D, DeserializeError> {
        D::deserialize(self)
    }

    #[doc(hidden)]
    pub fn try_deserialize<D: Deserialize>(&mut self) -> Result<Option<D>, InvalidInputError> {
        D::try_deserialize(self)
    }
}

#[derive(Debug, Error)]
#[error("validation failed")]
pub struct ValidationError;

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error(transparent)]
    InvalidInput(#[from] InvalidInputError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

pub trait Deserialize: Sized {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError>;

    #[doc(hidden)]
    fn try_deserialize(deserializer: &mut Deserializer) -> Result<Option<Self>, InvalidInputError> {
        let mut deserializer2 = *deserializer;
        match deserializer2.deserialize() {
            Ok(v) => {
                *deserializer = deserializer2;
                Ok(Some(v))
            }
            Err(DeserializeError::Validation(_)) => Ok(None),
            Err(DeserializeError::InvalidInput(err)) => Err(err),
        }
    }
}

impl Deserialize for bool {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::Bool(v) = deserializer.deserialize_token()? {
            return Ok(v);
        }
        Err(ValidationError.into())
    }
}

impl Deserialize for Int {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::Int(v) = deserializer.deserialize_token()? {
            return Ok(v);
        }
        Err(ValidationError.into())
    }
}

impl Deserialize for u8 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for u16 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for u32 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for u64 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for i8 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for i16 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for i32 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for i64 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        deserializer
            .deserialize::<Int>()?
            .try_into()
            .map_err(|_| ValidationError.into())
    }
}

impl Deserialize for f32 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::F32(v) = deserializer.deserialize_token()? {
            return Ok(v);
        }
        Err(ValidationError.into())
    }
}

impl Deserialize for f64 {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::F64(v) = deserializer.deserialize_token()? {
            return Ok(v);
        }
        Err(ValidationError.into())
    }
}

impl Deserialize for Str {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::Str(v) = deserializer.deserialize_token()? {
            return Ok(v);
        }
        Err(ValidationError.into())
    }
}

impl Deserialize for String {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        let Str(data) = deserializer.deserialize()?;
        let v = String::from_utf8(data).map_err(|_| ValidationError)?;
        Ok(v)
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if deserializer.try_deserialize::<Nil>()?.is_some() {
            return Ok(None);
        }
        let v = deserializer.deserialize()?;
        Ok(Some(v))
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        if let Token::Array(len) = deserializer.deserialize_token()? {
            let mut vec = Vec::with_capacity(len as usize);
            for _ in 0..len {
                vec.push(deserializer.deserialize()?);
            }
            return Ok(vec);
        }
        Err(ValidationError.into())
    }
}

/// Write out a MessagePack object.
pub fn serialize<S: Serialize>(s: S) -> Vec<u8> {
    let mut serializer = Serializer::new();
    serializer.serialize(s);
    serializer.into_inner()
}

/// Read out a MessagePack object.
pub fn deserialize<D: Deserialize>(r: &[u8]) -> Result<D, DeserializeError> {
    let mut deserializer = Deserializer::new(r);
    deserializer.deserialize()
}

/// Constructs a MessagePack object.
///
/// Return type is an opaque type implementing [Serialize].
///
/// # Example
///
/// ```
/// # use msgpack_schema::{msgpack, value::Bin, Serialize, serialize};
/// msgpack!(
///   // array literal
///   [
///     // numeric literals
///     0,
///     -42,
///     3.14,
///     2.71f32,
///     // actually any expression is allowed (as long as it's a supported type)
///     1 + 2 + 3,
///     // boolean
///     true,
///     false,
///     // nil is a keyword denoting the nil object
///     nil,
///     // map object
///     { "any value in key": nil },
///     { 0: 1, "trailing comma is ok": nil, },
///     // string literal to make a string object
///     "hello",
///     // Use an expression of [Bin] type to create a binary object
///     Bin(vec![0xDE, 0xAD, 0xBE, 0xEF])
///   ]
/// )
/// # ;
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! msgpack {
    ( $( $tt: tt )+ ) => {
        $crate::msgpack_value!($( $tt )+)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Human {
        age: u32,
        name: String,
    }

    impl Serialize for Human {
        fn serialize(&self, serializer: &mut Serializer) {
            serializer.serialize_map(2);
            serializer.serialize(0u32);
            serializer.serialize(self.age);
            serializer.serialize(1u32);
            serializer.serialize(&self.name);
        }
    }

    impl Deserialize for Human {
        fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
            let len = match deserializer.deserialize_token()? {
                Token::Map(len) => len,
                _ => return Err(ValidationError.into()),
            };

            let mut age: Option<u32> = None;
            let mut name: Option<String> = None;
            for _ in 0..len {
                let tag: u32 = deserializer.deserialize()?;
                match tag {
                    0 => {
                        if age.is_some() {
                            return Err(InvalidInputError.into());
                        }
                        age = Some(deserializer.deserialize()?);
                    }
                    1 => {
                        if name.is_some() {
                            return Err(InvalidInputError.into());
                        }
                        name = Some(deserializer.deserialize()?);
                    }
                    _ => {
                        let value::Any = deserializer.deserialize()?;
                    }
                }
            }
            Ok(Self {
                age: age.ok_or(ValidationError)?,
                name: name.ok_or(ValidationError)?,
            })
        }
    }

    #[test]
    fn it_works() {
        let val = Human {
            age: 42,
            name: "John".into(),
        };

        assert_eq!(val, value::deserialize(value::serialize(&val)).unwrap())
    }

    #[test]
    fn msgpack_macro() {
        let val = Human {
            age: 42,
            name: "John".into(),
        };
        let buf1 = serialize(&val);
        let lit = msgpack!({
            0: 42,
            1: "John",
        });
        let buf2 = serialize(&lit);
        assert_eq!(buf1, buf2);
    }
}
