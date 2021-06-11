use crate::Token;
use std::convert::{Infallible, TryFrom, TryInto};
use std::iter;
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
    Ext(i8, Vec<u8>),
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

impl From<Bin> for Value {
    fn from(v: Bin) -> Self {
        Self::Bin(v)
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
        matches!(self, Self::Ext(_, _))
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
    pub fn as_ext(&self) -> Option<(i8, &[u8])> {
        match self {
            Self::Ext(tag, v) => Some((*tag, v)),
            _ => None,
        }
    }
    pub fn as_ext_mut(&mut self) -> Option<(i8, &mut Vec<u8>)> {
        match self {
            Self::Ext(tag, v) => Some((*tag, v)),
            _ => None,
        }
    }
}

struct Serializer {
    stack: Vec<(usize, Value)>,
}

#[derive(Error, Debug, Clone)]
#[error("any of (hand-written) implementations of Serialize should be incorrect")]
struct SerializerError;

impl Serializer {
    pub fn new() -> Self {
        Self { stack: vec![] }
    }

    pub fn finish(mut self) -> Value {
        assert_eq!(self.stack.len(), 1, "{}", SerializerError);
        let (count, value) = self.stack.pop().unwrap();
        assert_eq!(count, 0, "{}", SerializerError);
        value
    }

    fn push_atom(&mut self, mut v: Value) {
        loop {
            if let Some((count, container)) = self.stack.last_mut() {
                if let Some(array) = container.as_array_mut() {
                    assert!(*count < array.len(), "{}", SerializerError);
                    array[*count] = v;
                    *count += 1;
                    if *count == array.len() {
                        v = self.stack.pop().unwrap().1;
                        continue;
                    }
                } else if let Some(map) = container.as_map_mut() {
                    assert!(*count < map.len() * 2, "{}", SerializerError);
                    if *count % 2 == 0 {
                        map[*count / 2].0 = v;
                    } else {
                        map[*count / 2].1 = v;
                    }
                    *count += 1;
                    if *count == map.len() * 2 {
                        v = self.stack.pop().unwrap().1;
                        continue;
                    }
                } else {
                    panic!("{}", SerializerError);
                }
            } else {
                self.stack.push((0, v));
            }
            break;
        }
    }
}

impl crate::Serializer for Serializer {
    type Error = std::convert::Infallible;
    fn serialize_nil(&mut self) -> Result<(), Self::Error> {
        self.push_atom(Value::Nil);
        Ok(())
    }
    fn serialize_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        self.push_atom(Value::from(v));
        Ok(())
    }
    fn serialize_int(&mut self, v: Int) -> Result<(), Self::Error> {
        self.push_atom(Value::Int(v));
        Ok(())
    }
    fn serialize_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        self.push_atom(Value::from(v));
        Ok(())
    }
    fn serialize_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        self.push_atom(Value::from(v));
        Ok(())
    }
    fn serialize_str(&mut self, v: &[u8]) -> Result<(), Self::Error> {
        self.push_atom(Value::Str(Str(v.to_owned())));
        Ok(())
    }
    fn serialize_bin(&mut self, v: &[u8]) -> Result<(), Self::Error> {
        self.push_atom(Value::Bin(Bin(v.to_owned())));
        Ok(())
    }
    fn serialize_array(&mut self, len: u32) -> Result<(), Self::Error> {
        if len == 0 {
            self.push_atom(Value::Array(vec![]));
        } else {
            let vec = iter::repeat(Value::Nil).take(len as usize).collect();
            self.stack.push((0, Value::Array(vec)));
        }
        Ok(())
    }
    fn serialize_map(&mut self, len: u32) -> Result<(), Self::Error> {
        let vec = iter::repeat((Value::Nil, Value::Nil))
            .take(len as usize)
            .collect();
        self.stack.push((0, Value::Map(vec)));
        Ok(())
    }
    fn serialize_ext(&mut self, tag: i8, v: &[u8]) -> Result<(), Self::Error> {
        self.push_atom(Value::Ext(tag, v.to_owned()));
        Ok(())
    }
}

#[doc(hidden)]
pub fn serialize<S: crate::Serialize>(x: &S) -> Value {
    let mut serializer = Serializer::new();
    x.serialize(&mut serializer)
        .unwrap_or_else(|infallible| match infallible {});
    serializer.finish()
}

impl crate::Serialize for Value {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: crate::Serializer,
    {
        match self {
            Value::Nil => serializer.serialize_nil(),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Int(v) => serializer.serialize_int(*v),
            Value::F32(v) => serializer.serialize_f32(*v),
            Value::F64(v) => serializer.serialize_f64(*v),
            Value::Str(v) => serializer.serialize_str(&v.0),
            Value::Bin(v) => serializer.serialize_bin(&v.0),
            Value::Array(v) => {
                serializer.serialize_array(v.len() as u32)?;
                for x in v {
                    x.serialize(serializer)?;
                }
                Ok(())
            }
            Value::Map(v) => {
                serializer.serialize_map(v.len() as u32)?;
                for (k, v) in v {
                    k.serialize(serializer)?;
                    v.serialize(serializer)?;
                }
                Ok(())
            }
            Value::Ext(tag, data) => serializer.serialize_ext(*tag, data),
        }
    }
}

impl Value {
    fn into_iter(self) -> Box<dyn Iterator<Item = Token>> {
        match self {
            Value::Nil => Box::new(iter::once(Token::Nil)),
            Value::Bool(v) => Box::new(iter::once(Token::Bool(v))),
            Value::Int(v) => Box::new(iter::once(Token::Int(v))),
            Value::F32(v) => Box::new(iter::once(Token::F32(v))),
            Value::F64(v) => Box::new(iter::once(Token::F64(v))),
            Value::Str(v) => Box::new(iter::once(Token::Str(v))),
            Value::Bin(v) => Box::new(iter::once(Token::Bin(v))),
            Value::Array(v) => Box::new(
                iter::once(Token::Array(v.len() as u32))
                    .chain(v.into_iter().flat_map(Value::into_iter)),
            ),
            Value::Map(v) => Box::new(
                iter::once(Token::Map(v.len() as u32)).chain(
                    v.into_iter()
                        .flat_map(|(k, v)| k.into_iter().chain(v.into_iter())),
                ),
            ),
            Value::Ext(tag, data) => Box::new(iter::once(Token::Ext(tag, data))),
        }
    }
}

struct Deserializer {
    iter: Box<dyn Iterator<Item = Token>>,
}

impl Deserializer {
    pub fn new(value: Value) -> Self {
        Self {
            iter: value.into_iter(),
        }
    }
}

impl crate::Deserializer for Deserializer {
    type Error = std::convert::Infallible;

    fn deserialize(&mut self) -> Result<Token, Self::Error> {
        let token = self
            .iter
            .next()
            .expect("any of (hand-written) implementations of Deserialize should be incorrect");
        Ok(token)
    }
}

#[doc(hidden)]
pub fn deserialize<D: crate::Deserialize>(
    value: Value,
) -> Result<D, crate::DeserializeError<Infallible>> {
    let mut deserializer = Deserializer::new(value);
    D::deserialize(&mut deserializer)
}

impl crate::Deserialize for Value {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, crate::DeserializeError<D::Error>>
    where
        D: crate::Deserializer,
    {
        let x = match deserializer.deserialize()? {
            Token::Nil => Value::Nil,
            Token::Bool(v) => v.into(),
            Token::Int(v) => v.into(),
            Token::F32(v) => v.into(),
            Token::F64(v) => v.into(),
            Token::Str(v) => v.into(),
            Token::Bin(v) => v.into(),
            Token::Array(len) => {
                let mut vec = vec![];
                for _ in 0..len {
                    vec.push(Value::deserialize(deserializer)?);
                }
                vec.into()
            }
            Token::Map(len) => {
                let mut map = vec![];
                for _ in 0..len {
                    map.push((
                        Value::deserialize(deserializer)?,
                        Value::deserialize(deserializer)?,
                    ));
                }
                map.into()
            }
            Token::Ext(tag, data) => Value::Ext(tag, data),
        };
        Ok(x)
    }
}

/// A special type for serializing and deserializing the `nil` object.
///
/// In our data model `()` does not represent the `nil` object because `()` should be zero-byte but `nil` has a size.
/// When you want to serialize or deserialize `nil` use this type instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nil;

impl crate::Serialize for Nil {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
    where
        S: crate::Serializer,
    {
        serializer.serialize_nil()
    }
}

impl crate::Deserialize for Nil {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, crate::DeserializeError<D::Error>>
    where
        D: crate::Deserializer,
    {
        let token = deserializer.deserialize()?;
        if token != Token::Nil {
            return Err(crate::DeserializeError::InvalidType);
        }
        Ok(Self)
    }
}

/// A special type used to deserialize any object and discard it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Any;

impl crate::Deserialize for Any {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, crate::DeserializeError<D::Error>>
    where
        D: crate::Deserializer,
    {
        let mut count = 1;
        while count > 0 {
            count -= 1;
            match deserializer.deserialize()? {
                Token::Nil
                | Token::Bool(_)
                | Token::Int(_)
                | Token::F32(_)
                | Token::F64(_)
                | Token::Str(_)
                | Token::Bin(_)
                | Token::Ext(_, _) => {}
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
