use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, Serializer, Token, ValidationError,
};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

/// Integer ranging from `-(2^63)` to `(2^64)-1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Int {
    sign: bool,
    /// Whenever `sign` is true, `value & (1 << 63)` is nonzero.
    value: u64,
}

impl From<u64> for Int {
    fn from(v: u64) -> Self {
        Self {
            sign: false,
            value: v,
        }
    }
}

impl From<i64> for Int {
    fn from(v: i64) -> Self {
        if v >= 0 {
            (v as u64).into()
        } else {
            Self {
                sign: true,
                value: v as u64,
            }
        }
    }
}

impl From<u8> for Int {
    fn from(v: u8) -> Self {
        (v as u64).into()
    }
}

impl From<u16> for Int {
    fn from(v: u16) -> Self {
        (v as u64).into()
    }
}

impl From<u32> for Int {
    fn from(v: u32) -> Self {
        (v as u64).into()
    }
}

#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl From<usize> for Int {
    fn from(v: usize) -> Self {
        (v as u64).into()
    }
}

impl From<i8> for Int {
    fn from(v: i8) -> Self {
        (v as i64).into()
    }
}

impl From<i16> for Int {
    fn from(v: i16) -> Self {
        (v as i64).into()
    }
}

impl From<i32> for Int {
    fn from(v: i32) -> Self {
        (v as i64).into()
    }
}

#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl From<isize> for Int {
    fn from(v: isize) -> Self {
        (v as i64).into()
    }
}

impl std::fmt::Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sign {
            (self.value as i64).fmt(f)
        } else {
            self.value.fmt(f)
        }
    }
}

/// Error type returned by `TryFrom<Int>` implementations.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
#[error("out of range integral type conversion attempted")]
pub struct TryFromIntError(());

impl TryFrom<Int> for u64 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        if value.sign {
            Err(TryFromIntError(()))
        } else {
            Ok(value.value)
        }
    }
}

impl TryFrom<Int> for i64 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        if value.sign {
            Ok(value.value as i64)
        } else {
            let v = value.value as i64;
            if v >= 0 {
                Ok(v)
            } else {
                Err(TryFromIntError(()))
            }
        }
    }
}

impl TryFrom<Int> for u8 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        u64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for u16 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        u64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for u32 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        u64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for usize {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        u64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for i8 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        i64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for i16 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        i64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

impl TryFrom<Int> for i32 {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        i64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl TryFrom<Int> for isize {
    type Error = TryFromIntError;

    fn try_from(value: Int) -> Result<Self, Self::Error> {
        i64::try_from(value)?
            .try_into()
            .map_err(|_| TryFromIntError(()))
    }
}

/// String objects of MessagePack are essentially byte arrays type that may contain any bytes.
///
/// # String type vs Binary type
///
/// MessagePack has a complicated history about its distinction between string type and binary type.
///
/// While an earlier version of MessagePack had only one type for encoding a binary data, namely the raw type,
/// it was later divided into two distinct types for supporting use cases in dynamically-typed languages. [^1]
/// The string type, one of the newly added types, is essentially what was originally called the raw type.
/// Because of this origin, despite its name, string objects of MessagePack can contain not just valid UTF-8 sequences but also any byte sequences.
/// And encoding non-UTF-8 byte sequences as a string object is a *perfectly valid* and *expected* usage by the spec authors.
///
/// [^1]: [https://github.com/msgpack/msgpack/issues/121](https://github.com/msgpack/msgpack/issues/121)
///
/// # So which to use in encoding my binary data?
///
/// When you decide to implement a custom serializer/deserializer for your own binary type,
/// it is recommended to use _string type_ instead of binary type for its encoding scheme for the following reasons.
///
/// - It just saves some memory. If your byte array is less than 32 byte length, using string type instead of byte array saves one byte per object.
/// - The disiction only matters when _not_ using a data schema. Because this crate offers a statically-typed data schema, and we know how to decode data into a Rust object at compile time,
/// distinction of these types in the input binary data is almost useless,
///
/// Although we strongly recommend you to use string types rather than binary types, this crate does _not_ force you to do so.
/// The functions and trait implementations provided by this crate are all taking a neutral stand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Str(pub Vec<u8>);

impl From<String> for Str {
    fn from(x: String) -> Self {
        Str(x.into_bytes())
    }
}

impl Str {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Byte array type.
///
/// As noted in the comment in [Str], using this type in this crate is almost nonsense, unless your data schema is shared by some external data providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bin(pub Vec<u8>);

impl Bin {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ext {
    pub r#type: i8,
    pub data: Vec<u8>,
}

#[doc(hidden)]
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(Int),
    F32(f32),
    F64(f64),
    Str(Str),
    Bin(Bin),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Ext(Ext),
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<Int> for Value {
    fn from(v: Int) -> Self {
        Self::Int(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Self::Int(v.into())
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Self::Int(v.into())
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::Int(v.into())
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::Int(v.into())
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Self::Int(v.into())
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Self::Int(v.into())
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::Int(v.into())
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Int(v.into())
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Int(v.into())
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Self {
        Self::Int(v.into())
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::F32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::F64(v)
    }
}

impl From<Str> for Value {
    fn from(v: Str) -> Self {
        Self::Str(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Str(Str(v.into_bytes()))
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        v.to_owned().into()
    }
}

impl From<Bin> for Value {
    fn from(v: Bin) -> Self {
        Self::Bin(v)
    }
}

impl From<Ext> for Value {
    fn from(v: Ext) -> Self {
        Self::Ext(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::Array(v)
    }
}

impl From<Vec<(Value, Value)>> for Value {
    fn from(v: Vec<(Value, Value)>) -> Self {
        Self::Map(v)
    }
}

impl From<Nil> for Value {
    fn from(_: Nil) -> Self {
        Self::Nil
    }
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Error)]
#[error("value is not of the expected type")]
pub struct TryFromValueError(());

impl TryFrom<Value> for bool {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl TryFrom<Value> for Int {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Int(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl TryFrom<Value> for f32 {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F32(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::F64(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl TryFrom<Value> for Vec<(Value, Value)> {
    type Error = TryFromValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(v) => Ok(v),
            _ => Err(TryFromValueError(())),
        }
    }
}

impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }
    pub fn is_f32(&self) -> bool {
        matches!(self, Self::F32(_))
    }
    pub fn is_f64(&self) -> bool {
        matches!(self, Self::F64(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    pub fn is_bin(&self) -> bool {
        matches!(self, Self::Bin(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
    pub fn is_ext(&self) -> bool {
        matches!(self, Self::Ext(_))
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<Int> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::F32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&Str> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_str_mut(&mut self) -> Option<&mut Str> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_bin(&self) -> Option<&Bin> {
        match self {
            Self::Bin(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_bin_mut(&mut self) -> Option<&mut Bin> {
        match self {
            Self::Bin(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::Array(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_map(&self) -> Option<&[(Value, Value)]> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_map_mut(&mut self) -> Option<&mut Vec<(Value, Value)>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_ext(&self) -> Option<&Ext> {
        match self {
            Self::Ext(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_ext_mut(&mut self) -> Option<&mut Ext> {
        match self {
            Self::Ext(v) => Some(v),
            _ => None,
        }
    }
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
            Token::Str(v) => v.into(),
            Token::Bin(v) => v.into(),
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
            Token::Ext(v) => v.into(),
        };
        Ok(x)
    }
}

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

#[doc(hidden)]
#[macro_export]
macro_rules! msgpack_array {
    ($acc: ident) => {
    };
    ($acc: ident,) => {
    };
    ($acc: ident nil) => {
        $acc.push($crate::msgpack_value!(nil));
    };
    ($acc: ident nil, $( $rest: tt )*) => {
        $acc.push($crate::msgpack_value!(nil));
        $crate::msgpack_array!($acc $( $rest )*);
    };
    ($acc: ident [ $( $tt: tt )* ]) => {
        $acc.push($crate::msgpack_value!([ $( $tt )* ]));
    };
    ($acc: ident [ $( $tt: tt )* ], $( $rest: tt )*) => {
        $acc.push($crate::msgpack_value!([ $( $tt )* ]));
        $crate::msgpack_array!($acc $( $rest )*);
    };
    ($acc: ident { $( $tt: tt )* }) => {
        $acc.push($crate::msgpack_value!({ $( $tt )* }));
    };
    ($acc: ident { $( $tt: tt )* }, $( $rest: tt )*) => {
        $acc.push($crate::msgpack_value!({ $( $tt )* }));
        $crate::msgpack_array!($acc $( $rest )*);
    };
    ($acc: ident $expr: expr) => {
        $acc.push($crate::msgpack_value!($expr));
    };
    ($acc: ident $expr: expr, $( $rest: tt )*) => {
        $acc.push($crate::msgpack_value!($expr));
        $crate::msgpack_array!($acc $( $rest )*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! msgpack_map {
    (@key $acc: ident []) => {
    };
    (@key $acc: ident [],) => {
    };
    (@key $acc: ident [ $( $key: tt )* ] : $( $rest: tt )*) => {
        let key = $crate::msgpack_value!($( $key )*);
        $crate::msgpack_map!(@val $acc key $( $rest )*);
    };
    (@key $acc: ident [ $( $key: tt )* ] $tt: tt $( $rest: tt )*) => {
        $crate::msgpack_map!(@key $acc [ $( $key )* $tt ] $( $rest )*);
    };
    (@val $acc: ident $key: ident nil) => {
        $acc.push(($key, $crate::msgpack_value!(nil)));
    };
    (@val $acc: ident $key: ident nil, $( $rest: tt )*) => {
        $acc.push(($key, $crate::msgpack_value!(nil)));
        $crate::msgpack_map!(@key $acc [] $( $rest )*);
    };
    (@val $acc: ident $key: ident [ $( $tt: tt )* ]) => {
        $acc.push(($key, $crate::msgpack_value!([ $( $tt )* ])));
    };
    (@val $acc: ident $key: ident [ $( $tt: tt )* ], $( $rest: tt )*) => {
        $acc.push(($key, $crate::msgpack_value!([ $( $tt )* ])));
        $crate::msgpack_map!(@key $acc [] $( $rest )*);
    };
    (@val $acc: ident $key: ident { $( $tt: tt )* }) => {
        $acc.push(($key, $crate::msgpack_value!({ $( $tt )* })));
    };
    (@val $acc: ident $key: ident { $( $tt: tt )* }, $( $rest: tt )*) => {
        $acc.push(($key, $crate::msgpack_value!({ $( $tt )* })));
        $crate::msgpack_map!(@key $acc [] $( $rest )*);
    };
    (@val $acc: ident $key: ident $expr: expr) => {
        $acc.push(($key, $crate::msgpack_value!($expr)));
    };
    (@val $acc: ident $key: ident $expr: expr, $( $rest: tt )*) => {
        $acc.push(($key, $crate::msgpack_value!($expr)));
        $crate::msgpack_map!(@key $acc [] $( $rest )*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! msgpack_value {
    (nil) => {
        $crate::value::Value::Nil
    };
    ([ $( $tt: tt )* ]) => {
        {
            #[allow(unused_mut)]
            let mut array = vec![];
            $crate::msgpack_array!(array $( $tt )*);
            $crate::value::Value::Array(array)
        }
    };
    ({ $( $tt: tt )* }) => {
        {
            #[allow(unused_mut)]
            let mut map = vec![];
            $crate::msgpack_map!(@key map [] $( $tt )*);
            $crate::value::Value::Map(map)
        }
    };
    ($other: expr) => {
        $crate::value::Value::from($other)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msgpack_macro() {
        assert_eq!(Value::Int(Int::from(42)), msgpack_value!(42));
        assert_eq!(Value::Int(Int::from(-42)), msgpack_value!(-42));
        assert_eq!(Value::F64(3.14), msgpack_value!(3.14));
        assert_eq!(Value::F32(3.14), msgpack_value!(3.14f32));
        assert_eq!(
            Value::Str(Str("hello world".to_owned().into_bytes())),
            msgpack_value!("hello world")
        );
        assert_eq!(Value::Bool(true), msgpack_value!(true));
        assert_eq!(Value::Bool(false), msgpack_value!(false));
        assert_eq!(Value::Nil, msgpack_value!(Nil));
        assert_eq!(Value::Int(Int::from(7)), msgpack_value!(3 + 4));

        assert_eq!(Value::Nil, msgpack_value!(nil));

        assert_eq!(
            Value::Array(vec![
                msgpack_value!(42),
                msgpack_value!(true),
                msgpack_value!(nil),
                msgpack_value!("hello"),
            ]),
            msgpack_value!([42, true, nil, "hello"])
        );

        assert_eq!(
            Value::Array(vec![
                msgpack_value!(42),
                msgpack_value!(true),
                msgpack_value!(nil),
                msgpack_value!("hello"),
            ]),
            msgpack_value!([42, true, nil, "hello",])
        );

        assert_eq!(
            Value::Array(vec![
                msgpack_value!(42),
                Value::Array(vec![
                    Value::Array(vec![msgpack_value!(true)]),
                    msgpack_value!(nil),
                ]),
                msgpack_value!("hello"),
            ]),
            msgpack_value!([42, [[true], nil], "hello",])
        );

        assert_eq!(Value::Array(vec![]), msgpack_value!([]));

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(42), msgpack_value!(true)),
                (msgpack_value!(nil), msgpack_value!("hello")),
            ]),
            msgpack_value!({ 42: true, nil: "hello", })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(0), msgpack_value!(nil)),
                (msgpack_value!(1), msgpack_value!(nil)),
            ]),
            msgpack_value!({ 0: nil, 1: nil })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(0), msgpack_value!({})),
                (msgpack_value!(1), msgpack_value!({})),
            ]),
            msgpack_value!({ 0: {}, 1: {} })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(0), msgpack_value!([])),
                (msgpack_value!(1), msgpack_value!([])),
            ]),
            msgpack_value!({ 0: [], 1: [] })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(0), msgpack_value!(-1)),
                (msgpack_value!(-1), msgpack_value!(0)),
            ]),
            msgpack_value!({ 0: -1, -1: 0 })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack_value!(42), msgpack_value!({ true: false })),
                (msgpack_value!({ nil: 3.14 }), msgpack_value!("hello")),
            ]),
            msgpack_value!({ 42: { true: false }, { nil: 3.14 }: "hello", })
        );
        assert_eq!(Value::Map(vec![]), msgpack_value!({}));

        assert_eq!(
            Value::Bin(Bin(vec![0xDEu8, 0xAD, 0xBE, 0xEF])),
            msgpack_value!(Bin(vec![0xDE, 0xAD, 0xBE, 0xEF]))
        );

        assert_eq!(
            Value::Array(vec![msgpack_value!(-42)]),
            msgpack_value!([-42])
        );
    }
}
