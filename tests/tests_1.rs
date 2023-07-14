use msgpack_schema::*;
use msgpack_value::{msgpack, Int, Value};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[test]
fn deserialize_ignores_extra_bytes() {
    let input: Vec<u8> = vec![0x01, 0xc1];
    let v: Int = deserialize(&input).unwrap();
    assert_eq!(v, Int::from(1u32));
}

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
