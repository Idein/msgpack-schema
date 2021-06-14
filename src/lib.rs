//! _msgpack-schema_ is a schema language for describing data formats encoded in MessagePack.
//! It provides two derive macros `Serialize` and `Deserialize` that allow you to transcode MessagePack binary data to/from Rust data structures in a type-directed way.
//!
//! ```rust,ignore
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
//! - `Option<T>` is roughly equal to declaring `T | null` in TypeScript. Deserializer interprets `null` as `None` whatever `T` is. So `Option<Option<T>>` is the same as `Option<T>` (unless used together with `#[optional]`.)
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
use std::io;
use thiserror::Error;
use value::{Bin, Int, Str};

pub trait Serializer {
    type Error: std::error::Error;
    fn serialize_nil(&mut self) -> Result<(), Self::Error>;
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error>;
    fn serialize_int(&mut self, v: Int) -> Result<(), Self::Error>;
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error>;
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error>;
    fn serialize_str(&mut self, v: &[u8]) -> Result<(), Self::Error>;
    fn serialize_bin(&mut self, v: &[u8]) -> Result<(), Self::Error>;
    fn serialize_array(&mut self, len: u32) -> Result<(), Self::Error>;
    fn serialize_map(&mut self, len: u32) -> Result<(), Self::Error>;
    fn serialize_ext(&mut self, tag: i8, v: &[u8]) -> Result<(), Self::Error>;
}

pub trait Serialize {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer;
}

impl<T: Serialize> Serialize for &T {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        T::serialize(*self, serializer)
    }
}

impl Serialize for bool {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(*self)
    }
}

impl Serialize for Int {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(*self)
    }
}

impl Serialize for u8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for u64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i8 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i16 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for i64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(Int::from(*self))
    }
}

impl Serialize for f32 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(*self)
    }
}

impl Serialize for f64 {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(*self)
    }
}

impl Serialize for Str {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl Serialize for str {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_bytes())
    }
}

impl Serialize for String {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_bytes())
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Some(v) => v.serialize(serializer),
            None => serializer.serialize_nil(),
        }
    }
}

impl<T: Serialize> Serialize for [T] {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_array(self.len() as u32)?;
        for x in self {
            x.serialize(serializer)?;
        }
        Ok(())
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_array(self.len() as u32)?;
        for x in self {
            x.serialize(serializer)?;
        }
        Ok(())
    }
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
    Ext(i8, Vec<u8>),
}

impl Token {
    pub fn to_bool(self) -> Option<bool> {
        match self {
            Token::Bool(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_int(self) -> Option<Int> {
        match self {
            Token::Int(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_f32(self) -> Option<f32> {
        match self {
            Token::F32(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_f64(self) -> Option<f64> {
        match self {
            Token::F64(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_str(self) -> Option<Str> {
        match self {
            Token::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_bin(self) -> Option<Bin> {
        match self {
            Token::Bin(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_array(self) -> Option<u32> {
        match self {
            Token::Array(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_map(self) -> Option<u32> {
        match self {
            Token::Map(v) => Some(v),
            _ => None,
        }
    }
    pub fn to_ext(self) -> Option<(i8, Vec<u8>)> {
        match self {
            Token::Ext(tag, data) => Some((tag, data)),
            _ => None,
        }
    }
}

pub trait Deserializer {
    type Error: std::error::Error;
    fn deserialize(&mut self) -> Result<Token, Self::Error>;
}

#[derive(Debug, Error)]
pub enum DeserializeError<E: std::error::Error> {
    #[error(transparent)]
    Deserializer(#[from] E),
    #[error("invalid type")]
    InvalidType,
    #[error("integer value out of range")]
    IntegerOutOfRange,
    #[error("invalid utf8")]
    InvalidUtf8,
    #[error("invalid length")]
    InvalidLength,
    #[error("duplicated field")]
    DuplicatedField,
    #[error("missing field")]
    MissingField,
    #[error("unknown variant")]
    UnknownVariant,
    #[error("invalid value")]
    InvalidValue,
}

pub trait Deserialize: Sized {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer;
}

impl Deserialize for bool {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let v = deserializer
            .deserialize()?
            .to_bool()
            .ok_or(DeserializeError::InvalidType)?;
        Ok(v)
    }
}

impl Deserialize for Int {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let v = deserializer
            .deserialize()?
            .to_int()
            .ok_or(DeserializeError::InvalidType)?;
        Ok(v)
    }
}

impl Deserialize for u8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for u16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for u32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for u64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for i8 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for i16 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for i32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for i64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        Int::deserialize(deserializer)?
            .try_into()
            .map_err(|_| DeserializeError::IntegerOutOfRange)
    }
}

impl Deserialize for f32 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        deserializer
            .deserialize()?
            .to_f32()
            .ok_or(DeserializeError::InvalidType)
    }
}

impl Deserialize for f64 {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        deserializer
            .deserialize()?
            .to_f64()
            .ok_or(DeserializeError::InvalidType)
    }
}

impl Deserialize for Str {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let buf = deserializer
            .deserialize()?
            .to_str()
            .ok_or(DeserializeError::InvalidType)?;
        Ok(buf)
    }
}

impl Deserialize for String {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let Str(data) = Deserialize::deserialize(deserializer)?;
        let v = String::from_utf8(data).map_err(|_| DeserializeError::InvalidUtf8)?;
        Ok(v)
    }
}

struct Unget<'a, D: Deserializer> {
    token: Option<Token>,
    orig: &'a mut D,
}

impl<'a, D: Deserializer> Deserializer for Unget<'a, D> {
    type Error = D::Error;

    fn deserialize(&mut self) -> Result<Token, Self::Error> {
        if let Some(token) = self.token.take() {
            Ok(token)
        } else {
            self.orig.deserialize()
        }
    }
}

fn unget<'a, D: Deserializer>(token: Token, deserializer: &'a mut D) -> Unget<'a, D> {
    Unget {
        token: Some(token),
        orig: deserializer,
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let token: Token = deserializer.deserialize()?;
        if token == Token::Nil {
            return Ok(None);
        }
        let v = T::deserialize(&mut unget(token, deserializer))?;
        Ok(Some(v))
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
    where
        D: Deserializer,
    {
        let len = deserializer
            .deserialize()?
            .to_array()
            .ok_or(DeserializeError::InvalidType)?;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::deserialize(deserializer)?);
        }
        Ok(vec.into())
    }
}

struct BinarySerializer<W> {
    w: W,
}

impl<W: io::Write> BinarySerializer<W> {
    pub fn new(w: W) -> Self {
        Self { w }
    }
}

impl<W: io::Write> Serializer for BinarySerializer<W> {
    type Error = io::Error;
    fn serialize_nil(&mut self) -> Result<(), Self::Error> {
        rmp::encode::write_nil(&mut self.w)
    }
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        rmp::encode::write_bool(&mut self.w, v)
    }
    fn serialize_int(&mut self, v: Int) -> Result<(), Self::Error> {
        if let Ok(v) = i64::try_from(v) {
            rmp::encode::write_sint(&mut self.w, v)?;
        } else {
            rmp::encode::write_uint(&mut self.w, u64::try_from(v).unwrap())?;
        }
        Ok(())
    }
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        rmp::encode::write_f32(&mut self.w, v)?;
        Ok(())
    }
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        rmp::encode::write_f64(&mut self.w, v)?;
        Ok(())
    }
    fn serialize_str(&mut self, v: &[u8]) -> Result<(), Self::Error> {
        rmp::encode::write_str_len(&mut self.w, v.len() as u32)?;
        self.w.write_all(v)?;
        Ok(())
    }
    fn serialize_bin(&mut self, v: &[u8]) -> Result<(), Self::Error> {
        rmp::encode::write_bin(&mut self.w, v)?;
        Ok(())
    }
    fn serialize_array(&mut self, len: u32) -> Result<(), Self::Error> {
        rmp::encode::write_array_len(&mut self.w, len)?;
        Ok(())
    }
    fn serialize_map(&mut self, len: u32) -> Result<(), Self::Error> {
        rmp::encode::write_map_len(&mut self.w, len)?;
        Ok(())
    }
    fn serialize_ext(&mut self, tag: i8, data: &[u8]) -> Result<(), Self::Error> {
        rmp::encode::write_ext_meta(&mut self.w, data.len() as u32, tag)?;
        self.w.write_all(data)?;
        Ok(())
    }
}

/// Write out a MessagePack object.
pub fn serialize<S: Serialize, W: io::Write>(s: S, w: W) -> io::Result<()> {
    let mut serializer = BinarySerializer::new(w);
    s.serialize(&mut serializer)?;
    Ok(())
}

trait ReadExt: ReadBytesExt {
    fn read_to_vec(&mut self, len: usize) -> Result<Vec<u8>, io::Error> {
        let mut buf = vec![];
        buf.resize(len, 0);
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<R: io::Read> ReadExt for R {}

struct BinaryDeserializer<R> {
    r: R,
}

impl<R: io::Read> BinaryDeserializer<R> {
    pub fn new(r: R) -> Self {
        Self { r }
    }
}

impl<R: io::Read> Deserializer for BinaryDeserializer<R> {
    type Error = io::Error;

    fn deserialize(&mut self) -> Result<Token, Self::Error> {
        let token = match rmp::decode::read_marker(&mut self.r)
            .map_err(|rmp::decode::MarkerReadError(err)| err)?
        {
            rmp::Marker::Null => Token::Nil,
            rmp::Marker::True => Token::Bool(true),
            rmp::Marker::False => Token::Bool(false),
            rmp::Marker::FixPos(v) => Token::Int(Int::from(v)),
            rmp::Marker::FixNeg(v) => Token::Int(Int::from(v)),
            rmp::Marker::U8 => Token::Int(Int::from(self.r.read_u8()?)),
            rmp::Marker::U16 => Token::Int(Int::from(self.r.read_u16::<BigEndian>()?)),
            rmp::Marker::U32 => Token::Int(Int::from(self.r.read_u32::<BigEndian>()?)),
            rmp::Marker::U64 => Token::Int(Int::from(self.r.read_u64::<BigEndian>()?)),
            rmp::Marker::I8 => Token::Int(Int::from(self.r.read_i8()?)),
            rmp::Marker::I16 => Token::Int(Int::from(self.r.read_i16::<BigEndian>()?)),
            rmp::Marker::I32 => Token::Int(Int::from(self.r.read_i32::<BigEndian>()?)),
            rmp::Marker::I64 => Token::Int(Int::from(self.r.read_i64::<BigEndian>()?)),
            rmp::Marker::F32 => Token::F32(self.r.read_f32::<BigEndian>()?),
            rmp::Marker::F64 => Token::F64(self.r.read_f64::<BigEndian>()?),
            rmp::Marker::FixStr(len) => {
                let len = len as usize;
                Token::Str(Str(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Str8 => {
                let len = self.r.read_u8()? as usize;
                Token::Str(Str(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Str16 => {
                let len = self.r.read_u16::<BigEndian>()? as usize;
                Token::Str(Str(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Str32 => {
                let len = self.r.read_u32::<BigEndian>()? as usize;
                Token::Str(Str(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Bin8 => {
                let len = self.r.read_u8()? as usize;
                Token::Bin(Bin(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Bin16 => {
                let len = self.r.read_u16::<BigEndian>()? as usize;
                Token::Bin(Bin(self.r.read_to_vec(len)?))
            }
            rmp::Marker::Bin32 => {
                let len = self.r.read_u32::<BigEndian>()? as usize;
                Token::Bin(Bin(self.r.read_to_vec(len)?))
            }
            rmp::Marker::FixArray(len) => Token::Array(len as u32),
            rmp::Marker::Array16 => Token::Array(self.r.read_u16::<BigEndian>()? as u32),
            rmp::Marker::Array32 => Token::Array(self.r.read_u32::<BigEndian>()? as u32),
            rmp::Marker::FixMap(len) => Token::Map(len as u32),
            rmp::Marker::Map16 => Token::Map(self.r.read_u16::<BigEndian>()? as u32),
            rmp::Marker::Map32 => Token::Map(self.r.read_u32::<BigEndian>()? as u32),
            rmp::Marker::FixExt1 => {
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(1)?)
            }
            rmp::Marker::FixExt2 => {
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(2)?)
            }
            rmp::Marker::FixExt4 => {
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(4)?)
            }
            rmp::Marker::FixExt8 => {
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(8)?)
            }
            rmp::Marker::FixExt16 => {
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(16)?)
            }
            rmp::Marker::Ext8 => {
                let len = self.r.read_u8()? as usize;
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(len)?)
            }
            rmp::Marker::Ext16 => {
                let len = self.r.read_u16::<BigEndian>()? as usize;
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(len)?)
            }
            rmp::Marker::Ext32 => {
                let len = self.r.read_u32::<BigEndian>()? as usize;
                let tag = self.r.read_i8()?;
                Token::Ext(tag, self.r.read_to_vec(len)?)
            }
            rmp::Marker::Reserved => Token::Nil,
        };
        Ok(token)
    }
}

/// Read out a MessagePack object.
pub fn deserialize<D: Deserialize, R: io::Read>(r: R) -> Result<D, DeserializeError<io::Error>> {
    let mut deserializer = BinaryDeserializer::new(r);
    Ok(D::deserialize(&mut deserializer)?)
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
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_map(2)?;
            0u32.serialize(serializer)?;
            self.age.serialize(serializer)?;
            1u32.serialize(serializer)?;
            self.name.serialize(serializer)?;
            Ok(())
        }
    }

    impl Deserialize for Human {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError<D::Error>>
        where
            D: Deserializer,
        {
            let len = deserializer
                .deserialize()?
                .to_map()
                .ok_or(DeserializeError::InvalidType)?;

            let mut age: Option<u32> = None;
            let mut name: Option<String> = None;
            for _ in 0..len {
                let tag = u32::deserialize(deserializer)?;
                match tag {
                    0 => {
                        if age.is_some() {
                            return Err(DeserializeError::DuplicatedField);
                        }
                        age = Some(Deserialize::deserialize(deserializer)?);
                    }
                    1 => {
                        if name.is_some() {
                            return Err(DeserializeError::DuplicatedField);
                        }
                        name = Some(Deserialize::deserialize(deserializer)?);
                    }
                    _ => {
                        let value::Any = Deserialize::deserialize(deserializer)?;
                    }
                }
            }
            Ok(Self {
                age: age.ok_or(DeserializeError::MissingField)?,
                name: name.ok_or(DeserializeError::MissingField)?,
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
        let mut buf1 = vec![];
        serialize(&val, &mut buf1).unwrap();
        let lit = msgpack!({
            0: 42,
            1: "John",
        });
        let mut buf2 = vec![];
        serialize(&lit, &mut buf2).unwrap();
        assert_eq!(buf1, buf2);
    }
}
