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
//! Compared with other schema languages like `rmp-serde`, `msgpack-schema` allows to specify more compact data representation, e.g., fixints as field keys, fixints as variant keys, etc.
//!
//! # Behaviours of serializers and deserializers
//!
//! ## Structs with named fields
//!
//! Structs with named fields are serialized into a `Map` object where keys are fixints specified by `#[tag]` attributes.
//! The current implementation serializes fields in order but one must not rely on this behavior.
//!
//! The deserializer interprets `Map` objects to create such structs.
//! Field order is irrelevant to the result.
//! If `Map` objects contains extra key-value pairs which are not contained in the definition of the struct, the deserializer simply ignores them.
//! If there are two or more values with the same key within a `Map` object, the preceding value is overwritten by the last value.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! struct S {
//!     #[tag = 0]
//!     x: u32,
//!     #[tag = 1]
//!     y: String,
//! }
//!
//! let s = S {
//!   x: 42,
//!   y: "hello".to_owned(),
//! };
//!
//! let b = b"\x82\x00\x2A\x01\xA5\x68\x65\x6c\x6c\x6f"; // 10 bytes; `{ 0: 42, 1: "hello" }`
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//!
//! // ignores irrelevant key-value pairs
//! let b = b"\x83\x00\x2A\x02\xC3\x01\xA5\x68\x65\x6c\x6c\x6f"; // 12 bytes; `{ 0: 42, 2: true, 1: "hello" }`
//! assert_eq!(s, deserialize(b).unwrap());
//!
//! // last value wins
//! let b = b"\x83\x00\xC3\x00\x2A\x01\xA5\x68\x65\x6c\x6c\x6f"; // 12 bytes; `{ 0: true, 0: 42, 1: "hello" }`
//! assert_eq!(s, deserialize(b).unwrap());
//! ```
//!
//! Fields in named structs may be tagged with `#[optional]`.
//!
//! - The tagged field must be of type `Option<T>`.
//! - On serialization, the key-value pair will not be included in the result map object when the field data contains `None`.
//! - On deserialization, the field of the result struct will be filled with `None` when the given MsgPack map object contains no corresponding key-value pair.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! struct S {
//!     #[tag = 0]
//!     x: u32,
//!     #[optional]
//!     #[tag = 1]
//!     y: Option<String>,
//! }
//!
//! let s = S {
//!   x: 42,
//!   y: Some("hello".to_owned()),
//! };
//! let b = b"\x82\x00\x2A\x01\xA5\x68\x65\x6c\x6c\x6f"; // 10 bytes; `{ 0: 42, 1: "hello" }`
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//!
//! let s = S {
//!   x: 42,
//!   y: None,
//! };
//! let b = b"\x81\x00\x2A"; // 3 bytes; `{ 0: 42 }`
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//! ```
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
//! Structs with named fields may be attached `#[untagged]`.
//! Untagged structs are serialized into an array and will not contain tags.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! #[untagged]
//! struct S {
//!     x: u32,
//!     y: String,
//! }
//!
//! let s = S {
//!   x: 42,
//!   y: "hello".to_owned(),
//! };
//! let b = b"\x92\x2A\xA5\x68\x65\x6c\x6c\x6f"; // 8 bytes; `[ 42, "hello" ]`
//!
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//! ```
//!
//! ## Newtype structs
//!
//! Tuple structs with only one element are treated transparently.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! struct S(u32);
//!
//! let s = S(42);
//! let b = b"\x2A"; // 1 byte; `42`
//!
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//! ```
//!
//! ## Unit structs and empty tuple structs
//!
//! Serialization and deserialization of unit structs and empty tuple structs are intentionally unsupported.
//!
//! ```
//! // It is error to derive `Serialize` / `Deserialize` for these types of structs.
//! struct S1;
//! struct S2();
//! ```
//!
//! ## Tuple structs
//!
//! Tuple structs with more than one element are encoded as an array.
//! It is validation error to deserialize an array with unmatched length.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! struct S(u32, bool);
//!
//! let s = S(42, true);
//! let b = b"\x92\x2A\xC3"; // 3 bytes; `[ 42, true ]`
//!
//! assert_eq!(serialize(&s), b);
//! assert_eq!(s, deserialize(b).unwrap());
//! ```
//!
//! ## Unit variants and empty tuple variants
//!
//! Unit variants and empty tuple variants are serialized into a single fixint whose value is determined by the tag.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! enum E {
//!     #[tag = 3]
//!     Foo
//! }
//!
//! let e = E::Foo;
//! let b = b"\x03"; // 1 byte; `3`
//!
//! assert_eq!(serialize(&e), b);
//! assert_eq!(e, deserialize(b).unwrap());
//! ```
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! enum E {
//!     #[tag = 3]
//!     Foo()
//! }
//!
//! let e = E::Foo();
//! let b = b"\x03"; // 1 byte; `3`
//!
//! assert_eq!(serialize(&e), b);
//! assert_eq!(e, deserialize(b).unwrap());
//! ```
//!
//! ## Newtype variants
//!
//! Newtype variants (one-element tuple variants) are serialized into an array of the tag and the inner value.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! enum E {
//!     #[tag = 3]
//!     Foo(u32)
//! }
//!
//! let e = E::Foo(42);
//! let b = b"\x92\x03\x2A"; // 3 bytes; `[ 3, 42 ]`
//!
//! assert_eq!(serialize(&e), b);
//! assert_eq!(e, deserialize(b).unwrap());
//! ```
//!
//! ## Untagged variants
//!
//! Enums may be attached `#[untagged]` when all variants are newtype variants.
//! Serializing untagged variants results in the same data layout as the inner type.
//! The deserializer deserializes into an untagged enum type by trying deserization one by one from the first variant to the last.
//!
//! ```
//! # use msgpack_schema::*;
//! # #[derive(Debug, PartialEq, Eq)]
//! #[derive(Serialize, Deserialize)]
//! #[untagged]
//! enum E {
//!     Foo(String),
//!     Bar(u32),
//! }
//!
//! let e = E::Bar(42);
//! let b = b"\x2A"; // 1 byte; `42`
//!
//! assert_eq!(serialize(&e), b);
//! assert_eq!(e, deserialize(b).unwrap());
//! ```
//!
//! # Write your own implementation of `Serialize` and `Deserialize`
//!
//! You may want to write your own implementation of `Serialize` and `Deserialize` in the following cases:
//!
//! 1. You need `impl` for types that are already defined by someone.
//! 2. You need extreme efficiency.
//! 3. Both.
//!
//! [IpAddr](std::net::IpAddr) is such a type satisfying (3).
//! In the most efficient situation, we want it to be 4 or 16 byte length plus one byte for a tag at any time.
//! This is achieved by giving a hard-written implementation like below.
//!
//! ```
//! # use msgpack_schema::*;
//! # use msgpack_value::*;
//! struct IpAddr(pub std::net::IpAddr);
//!
//! impl Serialize for IpAddr {
//!     fn serialize(&self, serializer: &mut Serializer) {
//!         match self.0 {
//!             std::net::IpAddr::V4(v4) => {
//!                 serializer.serialize_str(&v4.octets()); // 5 bytes
//!             }
//!             std::net::IpAddr::V6(v6) => {
//!                 serializer.serialize_str(&v6.octets()); // 17 bytes
//!             }
//!         }
//!     }
//! }
//!
//! impl Deserialize for IpAddr {
//!     fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
//!         let Str(data) = deserializer.deserialize()?;
//!         let ipaddr = match data.len() {
//!             4 => std::net::IpAddr::V4(std::net::Ipv4Addr::from(
//!                 <[u8; 4]>::try_from(data).unwrap(),
//!             )),
//!             16 => std::net::IpAddr::V6(std::net::Ipv6Addr::from(
//!                 <[u8; 16]>::try_from(data).unwrap(),
//!             )),
//!             _ => return Err(ValidationError.into()),
//!         };
//!         Ok(Self(ipaddr))
//!     }
//! }
//! ```
//!
//! # Appendix: Cheatsheet
//!
//! <table>
//!     <tr>
//!         <th>schema</th>
//!         <th>Rust</th>
//!         <th>MessagePack (human readable)</th>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>struct S {
//!     #[tag = 0]
//!     x: u32,
//!     #[tag = 1]
//!     y: bool,
//! }
//! </code></pre>
//!         </td>
//!         <td>
//!             <code>S { x: 42, y: true }</code>
//!         </td>
//!         <td>
//!             <code>{ 0: 42, 1: true }</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>struct S {
//!     #[optional]
//!     #[tag = 0]
//!     x: Option&lt;u32&gt;,
//! }
//! </code></pre>
//!         </td>
//!         <td>
//!             <code>S { x: Some(42) }</code>
//!         </td>
//!         <td>
//!             <code>{ 0: 42 }</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>struct S {
//!     #[optional]
//!     #[tag = 0]
//!     x: Option&lt;u32&gt;,
//! }
//! </code></pre>
//!         </td>
//!         <td>
//!             <code>S { x: None }</code>
//!         </td>
//!         <td>
//!             <code>{}</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>#[untagged]
//! struct S {
//!     #[tag = 0]
//!     x: u32,
//!     #[tag = 1]
//!     y: bool,
//! }
//! </code></pre>
//!         </td>
//!         <td>
//!             <code>S { x: 42, y: true }</code>
//!         </td>
//!         <td>
//!             <code>[ 42, true ]</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <code>struct S(u32)</code>
//!         </td>
//!         <td>
//!             <code>S(42)</code>
//!         </td>
//!         <td>
//!             <code>42</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <code>struct S</code>
//!         </td>
//!         <td>
//!             <code>S</code>
//!         </td>
//!         <td>UNSUPPORTED</td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <code>struct S()</code>
//!         </td>
//!         <td>
//!             <code>S()</code>
//!         </td>
//!         <td>UNSUPPORTED</td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <code>struct S(u32, bool)</code>
//!         </td>
//!         <td>
//!             <code>S(42, true)</code>
//!         </td>
//!         <td>
//!             <code>[ 42, true ]</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>enum E {
//!     #[tag = 3]
//!     Foo
//! }</code></pre>
//!         </td>
//!         <td>
//!             <code>E::Foo</code>
//!         </td>
//!         <td>
//!             <code>3</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>enum E {
//!     #[tag = 3]
//!     Foo()
//! }</code></pre>
//!         </td>
//!         <td>
//!             <code>E::Foo()</code>
//!         </td>
//!         <td>
//!             <code>3</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>enum E {
//!     #[tag = 3]
//!     Foo(u32)
//! }</code></pre>
//!         </td>
//!         <td>
//!             <code>E::Foo(42)</code>
//!         </td>
//!         <td>
//!             <code>[ 3, 42 ]</code>
//!         </td>
//!     </tr>
//!     <tr>
//!         <td>
//!             <pre><code>#[untagged]
//! enum E {
//!     Foo(u32)
//!     Bar(bool)
//! }</code></pre>
//!         </td>
//!         <td>
//!             <code>E::Bar(true)</code>
//!         </td>
//!         <td>
//!             <code>true</code>
//!         </td>
//!     </tr>
//! </table>
//!

use byteorder::BigEndian;
use byteorder::{self, ReadBytesExt};
pub use msgpack_schema_impl::*;
use msgpack_value::Value;
use msgpack_value::{Bin, Ext, Int, Str};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::Write;
use thiserror::Error;

/// This type holds all intermediate states during serialization.
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

    /// Equivalent to `S::serialize(&s, self)`.
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

impl<T: Serialize> Serialize for Box<T> {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize(&**self);
    }
}

impl<T: Serialize> Serialize for std::rc::Rc<T> {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize(&**self);
    }
}

impl<T: Serialize> Serialize for std::sync::Arc<T> {
    fn serialize(&self, serializer: &mut Serializer) {
        serializer.serialize(&**self);
    }
}

#[doc(hidden)]
pub trait StructSerialize: Serialize {
    fn count_fields(&self) -> u32;
    fn serialize_fields(&self, serializer: &mut Serializer);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Nil,
    Bool(bool),
    Int(Int),
    F32(f32),
    F64(f64),
    Str(&'a [u8]),
    Bin(&'a [u8]),
    Array(u32),
    Map(u32),
    Ext { tag: i8, data: &'a [u8] },
}

/// This error type represents blob-to-MessegePack transcode errors.
///
/// This error type is raised during deserialization either
/// 1. when (first bytes of) given binary data is not a message pack object, or
/// 2. when it unexpectedly reaches the end of input.
#[derive(Debug, Error)]
#[error("invalid input")]
pub struct InvalidInputError;

/// This type holds all intermediate states during deserialization.
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
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Str(ret)
            }
            rmp::Marker::Str8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Str(ret)
            }
            rmp::Marker::Str16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Str(ret)
            }
            rmp::Marker::Str32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Str(ret)
            }
            rmp::Marker::Bin8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Bin(ret)
            }
            rmp::Marker::Bin16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Bin(ret)
            }
            rmp::Marker::Bin32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let ret = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Bin(ret)
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
                    .map_err(|_| InvalidInputError)?,
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
                    .map_err(|_| InvalidInputError)?,
            ),
            rmp::Marker::FixExt1 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..1).ok_or(InvalidInputError)?;
                self.r = self.r.get(1..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::FixExt2 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..2).ok_or(InvalidInputError)?;
                self.r = self.r.get(2..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::FixExt4 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..4).ok_or(InvalidInputError)?;
                self.r = self.r.get(4..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::FixExt8 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..8).ok_or(InvalidInputError)?;
                self.r = self.r.get(8..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::FixExt16 => {
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..16).ok_or(InvalidInputError)?;
                self.r = self.r.get(16..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::Ext8 => {
                let len = self.r.read_u8().map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::Ext16 => {
                let len = self
                    .r
                    .read_u16::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::Ext32 => {
                let len = self
                    .r
                    .read_u32::<BigEndian>()
                    .map_err(|_| InvalidInputError)? as usize;
                let tag = self.r.read_i8().map_err(|_| InvalidInputError)?;
                let data = self.r.get(0..len).ok_or(InvalidInputError)?;
                self.r = self.r.get(len..).unwrap();
                Token::Ext { tag, data }
            }
            rmp::Marker::Reserved => return Err(InvalidInputError),
        };
        Ok(token)
    }

    /// Equivalent to `D::deserialize(self)`.
    pub fn deserialize<D: Deserialize>(&mut self) -> Result<D, DeserializeError> {
        D::deserialize(self)
    }

    /// Tries to deserialize an object of `D`.
    /// If it succeeds it returns `Ok(Some(_))` and the internal state of `self` is changed.
    /// If it fails with `ValidationError` it returns `Ok(None)` and the internal state of `self` is left unchanged.
    /// If it fails with `InvalidInputError` it passes on the error.
    pub fn try_deserialize<D: Deserialize>(&mut self) -> Result<Option<D>, InvalidInputError> {
        let mut branch = *self;
        match branch.deserialize() {
            Ok(v) => {
                *self = branch;
                Ok(Some(v))
            }
            Err(DeserializeError::Validation(_)) => Ok(None),
            Err(DeserializeError::InvalidInput(err)) => Err(err),
        }
    }

    /// Read any single message pack object and discard it.
    pub fn deserialize_any(&mut self) -> Result<(), DeserializeError> {
        let mut count = 1;
        while count > 0 {
            count -= 1;
            match self.deserialize_token()? {
                Token::Nil
                | Token::Bool(_)
                | Token::Int(_)
                | Token::F32(_)
                | Token::F64(_)
                | Token::Str(_)
                | Token::Bin(_)
                | Token::Ext { .. } => {}
                Token::Array(len) => {
                    count += len;
                }
                Token::Map(len) => {
                    count += len * 2;
                }
            }
        }
        Ok(())
    }
}

/// This error type represents type mismatch errors during deserialization.
#[derive(Debug, Error)]
#[error("validation failed")]
pub struct ValidationError;

/// This error type represents all possible errors during deserialization.
#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error(transparent)]
    InvalidInput(#[from] InvalidInputError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

pub trait Deserialize: Sized {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError>;
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
            return Ok(Str(v.to_vec()));
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

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        Ok(Box::new(deserializer.deserialize()?))
    }
}

impl<T: Deserialize> Deserialize for std::rc::Rc<T> {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        Ok(Self::new(deserializer.deserialize()?))
    }
}

impl<T: Deserialize> Deserialize for std::sync::Arc<T> {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        Ok(Self::new(deserializer.deserialize()?))
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

impl Serialize for Value {
    fn serialize(&self, serializer: &mut Serializer) {
        match self {
            Value::Nil => serializer.serialize_nil(),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Int(v) => serializer.serialize_int(*v),
            Value::F32(v) => serializer.serialize_f32(*v),
            Value::F64(v) => serializer.serialize_f64(*v),
            Value::Str(v) => serializer.serialize_str(&v.0),
            Value::Bin(v) => serializer.serialize_bin(&v.0),
            Value::Array(v) => {
                serializer.serialize_array(v.len() as u32);
                for x in v {
                    serializer.serialize(x);
                }
            }
            Value::Map(v) => {
                serializer.serialize_map(v.len() as u32);
                for (k, v) in v {
                    serializer.serialize(k);
                    serializer.serialize(v);
                }
            }
            Value::Ext(v) => serializer.serialize_ext(v.r#type, &v.data),
        }
    }
}

impl Deserialize for Value {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, DeserializeError> {
        let x = match deserializer.deserialize_token()? {
            Token::Nil => Value::Nil,
            Token::Bool(v) => v.into(),
            Token::Int(v) => v.into(),
            Token::F32(v) => v.into(),
            Token::F64(v) => v.into(),
            Token::Str(v) => Str(v.to_vec()).into(),
            Token::Bin(v) => Bin(v.to_vec()).into(),
            Token::Array(len) => {
                let mut vec: Vec<Value> = vec![];
                for _ in 0..len {
                    vec.push(deserializer.deserialize()?);
                }
                vec.into()
            }
            Token::Map(len) => {
                let mut map: Vec<(Value, Value)> = vec![];
                for _ in 0..len {
                    map.push((deserializer.deserialize()?, deserializer.deserialize()?));
                }
                map.into()
            }
            Token::Ext { tag, data } => Ext {
                r#type: tag,
                data: data.to_vec(),
            }
            .into(),
        };
        Ok(x)
    }
}

// for backward compatibility
#[doc(hidden)]
pub mod value {
    use super::*;

    pub use msgpack_value::{Bin, Ext, Int, Str, Value};

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
            deserializer.deserialize_any()?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use msgpack_value::msgpack;
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    macro_rules! roundtrip {
        ($name:ident, $ty:ty) => {
            #[cfg(test)]
            mod $name {
                use super::*;
                proptest! {
                    #[test]
                    fn test(v: $ty) {
                        assert_eq!(v, deserialize::<$ty>(serialize(&v).as_slice()).unwrap());
                    }
                }
            }
        };
    }

    roundtrip!(roundtrip_bool, bool);
    roundtrip!(roundtrip_i8, i8);
    roundtrip!(roundtrip_i16, i16);
    roundtrip!(roundtrip_i32, i32);
    roundtrip!(roundtrip_i64, i64);
    roundtrip!(roundtrip_u8, u8);
    roundtrip!(roundtrip_u16, u16);
    roundtrip!(roundtrip_u32, u32);
    roundtrip!(roundtrip_u64, u64);
    roundtrip!(roundtrip_f32, f32);
    roundtrip!(roundtrip_f64, f64);
    roundtrip!(roundtrip_str, String);
    roundtrip!(roundtrip_blob, Vec<i32>);
    roundtrip!(roundtrip_box, Box<i32>);
    roundtrip!(roundtrip_rc, std::rc::Rc<i32>);
    roundtrip!(roundtrip_arc, std::sync::Arc<i32>);

    roundtrip!(roundtrip_value, Value);
    roundtrip!(roundtrip_int, Int);

    #[derive(Debug, PartialEq, Eq, Arbitrary)]
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
                        deserializer.deserialize_any()?;
                    }
                }
            }
            Ok(Self {
                age: age.ok_or(ValidationError)?,
                name: name.ok_or(ValidationError)?,
            })
        }
    }

    roundtrip!(roundtrip_human, Human);

    fn check_serialize_result<T: Serialize>(x: T, v: Value) {
        let buf1 = serialize(&x);
        let buf2 = serialize(v);
        assert_eq!(buf1, buf2);
    }

    #[test]
    fn struct_vs_value() {
        check_serialize_result(
            Human {
                age: 42,
                name: "John".into(),
            },
            msgpack!({
                0: 42,
                1: "John",
            }),
        );
    }

    #[test]
    fn box_vs_value() {
        check_serialize_result(Box::new(42i32), msgpack!(42));
    }

    #[test]
    fn rc_vs_value() {
        check_serialize_result(std::rc::Rc::new(42i32), msgpack!(42));
    }

    #[test]
    fn arc_vs_value() {
        check_serialize_result(std::sync::Arc::new(42i32), msgpack!(42));
    }
}
