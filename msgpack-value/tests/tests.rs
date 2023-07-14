use msgpack_value::*;
use proptest::prelude::*;

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
