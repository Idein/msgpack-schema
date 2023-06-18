//! The MessagePack Data Model
//!
//! See also the [specification](https://github.com/msgpack/msgpack/blob/master/spec.md).
use proptest::prelude::*;
use proptest_derive::Arbitrary;
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

impl Arbitrary for Int {
    type Parameters = ();

    type Strategy = BoxedStrategy<Int>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (any::<i64>(), any::<bool>())
            .prop_map(|(high, low)| {
                if high < 0 {
                    Int::from(high)
                } else {
                    Int::from((high as u64) << 1 | (low as u64))
                }
            })
            .boxed()
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
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary)]
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

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Byte array type.
///
/// As noted in the comment in [Str], using this type in this crate is almost nonsense, unless your data schema is shared by some external data providers.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary)]
pub struct Bin(pub Vec<u8>);

impl Bin {
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// User-extended type.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary)]
pub struct Ext {
    pub r#type: i8,
    pub data: Vec<u8>,
}

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

pub trait Index {
    fn index<'a>(&self, v: &'a Value) -> &'a Value;
}

impl<T: Index + ?Sized> Index for &T {
    fn index<'a>(&self, v: &'a Value) -> &'a Value {
        (*self).index(v)
    }
}

impl Index for str {
    fn index<'a>(&self, v: &'a Value) -> &'a Value {
        let map = v
            .as_map()
            .expect("this type of object is not indexable by str");
        for (key, value) in map.iter().rev() {
            if let Some(Str(key)) = key.as_str() {
                if key == self.as_bytes() {
                    return value;
                }
            }
        }
        panic!("key not found in object");
    }
}

impl Index for String {
    fn index<'a>(&self, v: &'a Value) -> &'a Value {
        self.as_str().index(v)
    }
}

impl Index for usize {
    fn index<'a>(&self, v: &'a Value) -> &'a Value {
        let array = v
            .as_array()
            .expect("this type of object is not indexable by usize");
        &array[*self]
    }
}

impl<T> core::ops::Index<T> for Value
where
    T: Index,
{
    type Output = Value;
    /// Accessing inner values of `value` using indexing `value[0]` or `value["foo"]`.
    ///
    /// # Panics
    ///
    /// This function panics when `self` does not contain given key.
    ///
    /// # Duplicate keys
    ///
    /// If `self` is a map object and contains two or more keys matching against the given index,
    /// indexing works as if the preceding keys do not exist in the object.
    /// This is the same behaviour as [what EMCA-262 specifies](https://stackoverflow.com/a/23195243).
    ///
    /// ```
    /// # use msgpack_value::{msgpack, Int};
    /// let v = msgpack!({ "a" : 0, "a" : 1 });
    /// assert_eq!(v["a"], Int::from(1u32).into());
    /// ```
    fn index(&self, index: T) -> &Self::Output {
        index.index(self)
    }
}

#[test]
fn test_index() {
    let v = msgpack!({ 0: 1, "foo" : "bar", "foo" : "baz" });
    let k = &v["foo"];
    // last value wins
    assert_eq!(k.as_str().unwrap().as_bytes(), "baz".as_bytes());

    let v = msgpack!(["foo", "bar", "baz"]);
    let k = &v[1];
    assert_eq!(k.as_str().unwrap().as_bytes(), "bar".as_bytes());
}

#[test]
#[should_panic]
fn test_index_panic_array_index_by_str() {
    let _ = msgpack!([])["foo"];
}

#[test]
#[should_panic]
fn test_index_panic_array_out_of_range() {
    let _ = msgpack!([])[0];
}

#[test]
#[should_panic]
fn test_index_panic_map_key_not_found() {
    let _ = msgpack!({"foo":"bar"})["baz"];
}

/// Error type returned by `TryFrom<Value>` implementations.
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

impl Arbitrary for Value {
    type Parameters = ();
    type Strategy = BoxedStrategy<Value>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        fn arb_value() -> BoxedStrategy<Value> {
            let leaf = prop_oneof![
                Just(Value::Nil),
                any::<bool>().prop_map(Value::Bool),
                any::<Int>().prop_map(Value::Int),
                any::<f32>().prop_map(Value::F32),
                any::<f64>().prop_map(Value::F64),
                any::<Str>().prop_map(Value::Str),
                any::<Bin>().prop_map(Value::Bin),
                any::<Ext>().prop_map(Value::Ext),
            ];
            leaf.prop_recursive(8, 256, 10, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..10).prop_map(Value::Array),
                    prop::collection::vec((inner.clone(), inner), 0..10).prop_map(Value::Map),
                ]
            })
            .boxed()
        }
        arb_value()
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
        $crate::Value::Nil
    };
    ([ $( $tt: tt )* ]) => {
        {
            #[allow(unused_mut)]
            let mut array;
            #[allow(clippy::vec_init_then_push)]
            {
                array = vec![];
                $crate::msgpack_array!(array $( $tt )*);
            }
            $crate::Value::Array(array)
        }
    };
    ({ $( $tt: tt )* }) => {
        {
            #[allow(unused_mut)]
            let mut map;
            #[allow(clippy::vec_init_then_push)]
            {
                map = vec![];
                $crate::msgpack_map!(@key map [] $( $tt )*);
            }
            $crate::Value::Map(map)
        }
    };
    ($other: expr) => {
        $crate::Value::from($other)
    };
}

/// Constructs a [Value] from literal.
///
/// # Example
///
/// ```
/// # use msgpack_value::{msgpack, Bin, Int, Str, Value};
/// let obj = msgpack!(
///     // array literal
///     [
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
///     // string literal to make a string object
///     "hello",
///     // Use an expression of [Bin] type to create a binary object
///     Bin(vec![0xDE, 0xAD, 0xBE, 0xEF]),
///     // map object
///     { "any value in key": nil },
///     { 0: 1, "trailing comma is ok": nil, }
///     ]
/// );
///
/// assert_eq!(
///     obj,
///     Value::Array(vec![
///         Value::Int(Int::from(0)),
///         Value::Int(Int::from(-42)),
///         Value::F64(3.14),
///         Value::F32(2.71),
///         Value::Int(Int::from(6)),
///         Value::Bool(true),
///         Value::Bool(false),
///         Value::Nil,
///         Value::Str(Str("hello".to_owned().into_bytes())),
///         Value::Bin(Bin(vec![0xDE, 0xAD, 0xBE, 0xEF])),
///         Value::Map(vec![(
///             Value::Str(Str("any value in key".to_owned().into_bytes())),
///             Value::Nil
///         ),]),
///         Value::Map(vec![
///             (Value::Int(Int::from(0)), Value::Int(Int::from(1))),
///             (
///                 Value::Str(Str("trailing comma is ok".to_owned().into_bytes())),
///                 Value::Nil
///             ),
///         ])
///     ])
/// )
/// # ;
/// ```
#[macro_export]
macro_rules! msgpack {
    ($( $tt: tt )*) => {
        $crate::msgpack_value!($( $tt )*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_int() {
        assert_eq!(i64::MAX, Int::from(i64::MAX).try_into().unwrap());
        assert_eq!(i64::MIN, Int::from(i64::MIN).try_into().unwrap());
        assert_eq!(u64::MAX, Int::from(u64::MAX).try_into().unwrap());
        assert_eq!(u64::MIN, Int::from(u64::MIN).try_into().unwrap());
    }

    #[test]
    fn msgpack_macro() {
        assert_eq!(Value::Int(Int::from(42)), msgpack!(42));
        assert_eq!(Value::Int(Int::from(-42)), msgpack!(-42));
        assert_eq!(Value::F64(1.23), msgpack!(1.23));
        assert_eq!(Value::F32(1.23), msgpack!(1.23f32));
        assert_eq!(
            Value::Str(Str("hello world".to_owned().into_bytes())),
            msgpack!("hello world")
        );
        assert_eq!(Value::Bool(true), msgpack!(true));
        assert_eq!(Value::Bool(false), msgpack!(false));
        assert_eq!(Value::Int(Int::from(7)), msgpack!(3 + 4));

        assert_eq!(Value::Nil, msgpack!(nil));

        assert_eq!(
            Value::Array(vec![
                msgpack!(42),
                msgpack!(true),
                msgpack!(nil),
                msgpack!("hello"),
            ]),
            msgpack!([42, true, nil, "hello"])
        );

        assert_eq!(
            Value::Array(vec![
                msgpack!(42),
                msgpack!(true),
                msgpack!(nil),
                msgpack!("hello"),
            ]),
            msgpack!([42, true, nil, "hello",])
        );

        assert_eq!(
            Value::Array(vec![
                msgpack!(42),
                Value::Array(vec![Value::Array(vec![msgpack!(true)]), msgpack!(nil),]),
                msgpack!("hello"),
            ]),
            msgpack!([42, [[true], nil], "hello",])
        );

        assert_eq!(Value::Array(vec![]), msgpack!([]));

        assert_eq!(
            Value::Map(vec![
                (msgpack!(42), msgpack!(true)),
                (msgpack!(nil), msgpack!("hello")),
            ]),
            msgpack!({ 42: true, nil: "hello", })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack!(0), msgpack!(nil)),
                (msgpack!(1), msgpack!(nil)),
            ]),
            msgpack!({ 0: nil, 1: nil })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack!(0), msgpack!({})),
                (msgpack!(1), msgpack!({})),
            ]),
            msgpack!({ 0: {}, 1: {} })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack!(0), msgpack!([])),
                (msgpack!(1), msgpack!([])),
            ]),
            msgpack!({ 0: [], 1: [] })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack!(0), msgpack!(-1)),
                (msgpack!(-1), msgpack!(0)),
            ]),
            msgpack!({ 0: -1, -1: 0 })
        );

        assert_eq!(
            Value::Map(vec![
                (msgpack!(42), msgpack!({ true: false })),
                (msgpack!({ nil: 1.23 }), msgpack!("hello")),
            ]),
            msgpack!({ 42: { true: false }, { nil: 1.23 }: "hello", })
        );
        assert_eq!(Value::Map(vec![]), msgpack!({}));

        assert_eq!(
            Value::Bin(Bin(vec![0xDEu8, 0xAD, 0xBE, 0xEF])),
            msgpack!(Bin(vec![0xDE, 0xAD, 0xBE, 0xEF]))
        );

        assert_eq!(Value::Array(vec![msgpack!(-42)]), msgpack!([-42]));
    }

    proptest! {
        #[test]
        fn no_panic_arb_int(_ in any::<Int>()) {
            // pass
        }

        #[test]
        fn no_panic_arb_value(_ in any::<Value>()) {
            // pass
        }
    }
}
