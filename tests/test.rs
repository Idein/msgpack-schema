use msgpack_schema::{
    value::{Bin, Value},
    *,
};
use proptest::prelude::*;

#[test]
fn failing() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn serialize_struct_tag() {
    #[derive(Serialize)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        name: String,
    }

    let val = Human {
        age: 42,
        name: "John".into(),
    };
    assert_eq!(
        value::serialize(&val),
        Value::Map(vec![
            (Value::Int(0.into()), Value::Int(42.into())),
            (Value::Int(2.into()), Value::Str("John".to_owned().into()))
        ])
    );
}

#[test]
fn deserialize_struct_tag() {
    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        name: String,
    }

    let val = Value::Map(vec![
        (Value::Int(0.into()), Value::Int(42.into())),
        (Value::Int(2.into()), Value::Str("John".to_owned().into())),
    ]);

    assert_eq!(
        Human {
            age: 42,
            name: "John".into(),
        },
        value::deserialize(val).unwrap()
    );
}

#[test]
fn struct_tag_roundtrip() {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        name: String,
    }

    let val = Human {
        age: 42,
        name: "John".into(),
    };

    assert_eq!(val, value::deserialize(value::serialize(&val)).unwrap());

    let val = Value::Map(vec![
        (Value::Int(0.into()), Value::Int(42.into())),
        (Value::Int(2.into()), Value::Str("John".to_owned().into())),
    ]);

    assert_eq!(
        val,
        value::serialize(&value::deserialize::<Human>(val.clone()).unwrap())
    );
}

#[test]
fn error_duplicate_tags() {
    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        name: String,
    }

    let val = Value::Map(vec![
        (Value::Int(0.into()), Value::Int(42.into())),
        (Value::Int(0.into()), Value::Int(43.into())),
        (Value::Int(2.into()), Value::Str("John".to_owned().into())),
    ]);

    assert!(value::deserialize::<Human>(val).is_err());
}

#[test]
fn serialize_struct_optional() {
    #[derive(Serialize)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        #[optional]
        name: Option<String>,
    }

    let val = Human {
        age: 42,
        name: Some("John".into()),
    };
    assert_eq!(
        value::serialize(&val),
        Value::Map(vec![
            (Value::Int(0.into()), Value::Int(42.into())),
            (Value::Int(2.into()), Value::Str("John".to_owned().into()))
        ])
    );

    let val = Human {
        age: 42,
        name: None,
    };
    assert_eq!(
        value::serialize(&val),
        Value::Map(vec![(Value::Int(0.into()), Value::Int(42.into())),])
    );
}

#[test]
fn deserialize_struct_optional() {
    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct Human {
        #[tag = 0]
        age: u32,
        #[tag = 2]
        #[optional]
        name: Option<String>,
    }

    let val = Value::Map(vec![
        (Value::Int(0.into()), Value::Int(42.into())),
        (Value::Int(2.into()), Value::Str("John".to_owned().into())),
    ]);
    assert_eq!(
        Human {
            age: 42,
            name: Some("John".into()),
        },
        value::deserialize(val).unwrap()
    );

    let val = Value::Map(vec![(Value::Int(0.into()), Value::Int(42.into()))]);
    assert_eq!(
        Human {
            age: 42,
            name: None,
        },
        value::deserialize(val).unwrap()
    );
}

#[test]
fn serialize_unit_variants() {
    #[derive(Serialize)]
    enum Animal {
        #[tag = 2]
        Dog,
    }

    let val = Animal::Dog;
    assert_eq!(value::serialize(&val), Value::Int(2.into()));
}

#[test]
fn deserialize_unit_variants() {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Animal {
        #[tag = 2]
        Dog,
    }

    let val = Value::Int(2.into());
    assert_eq!(Animal::Dog, value::deserialize(val).unwrap());
}

#[test]
fn serialize_newtype_struct() {
    #[derive(Serialize)]
    struct S(u32);

    let val = S(42);
    assert_eq!(value::serialize(&val), Value::Int(42.into()));
}

#[test]
fn deserialize_newtype_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct S(u32);

    let val = Value::Int(42.into());
    assert_eq!(S(42), value::deserialize(val).unwrap());
}

#[test]
fn serialize_empty_tuple_variants() {
    #[derive(Serialize)]
    enum Animal {
        #[tag = 2]
        Dog(),
    }

    let val = Animal::Dog();
    assert_eq!(value::serialize(&val), Value::Int(2.into()));
}

#[test]
fn deserialize_empty_tuple_variants() {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Animal {
        #[tag = 2]
        Dog(),
    }

    let val = Value::Int(2.into());
    assert_eq!(Animal::Dog(), value::deserialize(val).unwrap());
}

#[test]
fn serialize_tuple_variants() {
    #[derive(Serialize, Debug, PartialEq, Eq)]
    enum Animal {
        #[tag = 1]
        Cat(String),
        #[tag = 2]
        Dog(u32),
    }

    assert_eq!(
        value::serialize(&Animal::Cat("hello".to_owned())),
        Value::Array(vec![1.into(), "hello".to_owned().into()])
    );

    assert_eq!(
        value::serialize(&Animal::Dog(42u32.into())),
        Value::Array(vec![2.into(), 42u32.into()])
    );
}

#[test]
fn deserialize_tuple_variants() {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Animal {
        #[tag = 1]
        Cat(String),
        #[tag = 2]
        Dog(u32),
        #[tag = 3]
        Bird,
    }

    let val = Value::Int(3.into());
    assert_eq!(Animal::Bird, value::deserialize(val).unwrap());

    let val = Value::Int(1.into());
    assert!(value::deserialize::<Animal>(val).is_err());

    let val = Value::Int(10.into());
    assert!(value::deserialize::<Animal>(val).is_err());

    let val = Value::Array(vec![1.into(), 42u32.to_owned().into()]);
    assert!(value::deserialize::<Animal>(val).is_err());

    let val = Value::Array(vec![1.into(), "hello".to_owned().into()]);
    assert_eq!(
        Animal::Cat("hello".to_owned()),
        value::deserialize(val).unwrap()
    );
}

#[test]
fn serialize_untagged_enum() {
    #[derive(Serialize, Debug, PartialEq, Eq)]
    #[untagged]
    enum Animal {
        Cat(String),
        Dog(u32),
    }

    let val = Value::Int(3.into());
    assert_eq!(value::serialize(&Animal::Dog(3)), val);

    let val = Value::Str("hello".to_owned().into());
    assert_eq!(value::serialize(&Animal::Cat("hello".to_owned())), val);
}

#[test]
fn deserialize_untagged_enum() {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[untagged]
    enum Animal {
        Cat(String),
        Dog(u32),
    }

    let val = Value::Int(3.into());
    assert_eq!(Animal::Dog(3), value::deserialize(val).unwrap());

    let val = Value::Str("hello".to_owned().into());
    assert_eq!(
        Animal::Cat("hello".to_owned()),
        value::deserialize::<Animal>(val).unwrap()
    );

    let val = Value::Int((-10).into());
    assert!(value::deserialize::<Animal>(val).is_err());
}

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Nil),
        any::<bool>().prop_map(|v| v.into()),
        any::<u64>().prop_map(|v| v.into()),
        any::<i64>().prop_map(|v| v.into()),
        any::<f32>().prop_map(|v| v.into()),
        any::<f64>().prop_map(|v| v.into()),
        ".*".prop_map(|v| v.into()),
        ".*".prop_map(|v| Bin(v.into_bytes()).into()),
        any::<i8>().prop_flat_map(|tag| ".*".prop_map(move |v| Value::Ext(tag, v.into_bytes()))),
    ];
    leaf.prop_recursive(
        4,   // 4 levels deep
        128, // Shoot for maximum size of 128 nodes
        5,   // We put up to 5 items per collection
        |inner| {
            prop_oneof![
                // Take the inner strategy and make the two recursive cases.
                prop::collection::vec(inner.clone(), 0..=4).prop_map(|v| v.into()),
                prop::collection::vec((inner.clone(), inner), 0..=4).prop_map(|v| v.into()),
            ]
        },
    )
}

proptest! {
    #[test]
    fn roundtrip_value(v in arb_value()) {
        let x = value::serialize(&v);
        assert_eq!(v, x);
        assert_eq!(v, value::deserialize(x).unwrap());
    }

    #[test]
    fn roundtrip_binary(v in arb_value()) {
        let buf = msgpack_schema::serialize(&v);
        assert_eq!(v, msgpack_schema::deserialize(buf.as_slice()).unwrap());
    }
}
