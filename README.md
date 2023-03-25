# msgpack-schema [![Crates.io](https://img.shields.io/crates/v/msgpack-schema)](https://crates.io/crates/msgpack-schema) [![docs.rs](https://img.shields.io/docsrs/msgpack-schema)](https://docs.rs/msgpack-schema/) [![CI](https://github.com/Idein/msgpack-schema/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/Idein/msgpack-schema/actions/workflows/ci.yml)

<!-- cargo-rdme start -->

_msgpack-schema_ is a schema language for describing data formats encoded in MessagePack.
It provides two derive macros `Serialize` and `Deserialize` that allow you to transcode MessagePack binary data to/from Rust data structures in a type-directed way.

```rust
use msgpack_schema::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Human {
    #[tag = 0]
    name: String,
    #[tag = 2]
    #[optional]
    age: Option<u32>,
}
```

Compared with other schema languages like `rmp-serde`, `msgpack-schema` allows to specify more compact data representation, e.g., fixints as field keys, fixints as variant keys, etc.

## Behaviours of serializers and deserializers

### Structs with named fields

Structs with named fields are serialized into a `Map` object where keys are fixints specified by `#[tag]` attributes.
The current implementation serializes fields in order but one must not rely on this behavior.

The deserializer interprets `Map` objects to create such structs.
Field order is irrelevant to the result.
If `Map` objects contains extra key-value pairs which are not contained in the definition of the struct, the deserializer simply ignores them.
It is validation error when `Map` objects have two or more values with the same key.

```rust
#[derive(Serialize, Deserialize)]
struct S {
    #[tag = 0]
    x: u32,
    #[tag = 1]
    y: String,
}

let s = S {
  x: 42,
  y: "hello".to_owned(),
};

let b = b"\x82\x00\x2A\x01\xA5\x68\x65\x6c\x6c\x6f"; // 10 bytes; `{ 0: 42, 1: "hello" }`
assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());

// ignores irrelevant key-value pairs
let b = b"\x83\x00\x2A\x02\xC3\x01\xA5\x68\x65\x6c\x6c\x6f"; // 12 bytes; `{ 0: 42, 2: true, 1: "hello" }`
assert_eq!(s, deserialize(b).unwrap());

// maps with duplicate keys are invalid input
let b = b"\x83\x00\x2A\x00\xC3\x01\xA5\x68\x65\x6c\x6c\x6f"; // 12 bytes; `{ 0: 42, 0: true, 1: "hello" }`
assert!(matches!(deserialize::<S>(b).unwrap_err(), DeserializeError::InvalidInput(_)));
```

Fields in named structs may be tagged with `#[optional]`.

- The tagged field must be of type `Option<T>`.
- On serialization, the key-value pair will not be included in the result map object when the field data contains `None`.
- On deserialization, the field of the result struct will be filled with `None` when the given MsgPack map object contains no corresponding key-value pair.

```rust
#[derive(Serialize, Deserialize)]
struct S {
    #[tag = 0]
    x: u32,
    #[optional]
    #[tag = 1]
    y: Option<String>,
}

let s = S {
  x: 42,
  y: Some("hello".to_owned()),
};
let b = b"\x82\x00\x2A\x01\xA5\x68\x65\x6c\x6c\x6f"; // 10 bytes; `{ 0: 42, 1: "hello" }`
assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());

let s = S {
  x: 42,
  y: None,
};
let b = b"\x81\x00\x2A"; // 3 bytes; `{ 0: 42 }`
assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());
```

The `#[flatten]` attribute is used to factor out a single definition of named struct into multiple ones.

```rust
#[derive(Serialize)]
struct S1 {
    #[tag = 1]
    x: u32,
}

#[derive(Serialize)]
struct S2 {
    #[flatten]
    s1: S1,
    #[tag = 2]
    y: u32,
}

#[derive(Serialize)]
struct S3 {
    #[tag = 1]
    x: u32,
    #[tag = 2]
    y: u32,
}

assert_eq!(serialize(S2 { s1: S1 { x: 42 }, y: 43, }), serialize(S3 { x: 42, y: 43 }));
```

Structs with named fields may be attached `#[untagged]`.
Untagged structs are serialized into an array and will not contain tags.

```rust
#[derive(Serialize, Deserialize)]
#[untagged]
struct S {
    x: u32,
    y: String,
}

let s = S {
  x: 42,
  y: "hello".to_owned(),
};
let b = b"\x92\x2A\xA5\x68\x65\x6c\x6c\x6f"; // 8 bytes; `[ 42, "hello" ]`

assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());
```

### Newtype structs

Tuple structs with only one element are treated transparently.

```rust
#[derive(Serialize, Deserialize)]
struct S(u32);

let s = S(42);
let b = b"\x2A"; // 1 byte; `42`

assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());
```

### Unit structs and empty tuple structs

Serialization and deserialization of unit structs and empty tuple structs are intentionally unsupported.

```rust
// It is error to derive `Serialize` / `Deserialize` for these types of structs.
struct S1;
struct S2();
```

### Tuple structs

Tuple structs with more than one element are encoded as an array.
It is validation error to deserialize an array with unmatched length.

```rust
#[derive(Serialize, Deserialize)]
struct S(u32, bool);

let s = S(42, true);
let b = b"\x92\x2A\xC3"; // 3 bytes; `[ 42, true ]`

assert_eq!(serialize(&s), b);
assert_eq!(s, deserialize(b).unwrap());
```

### Unit variants and empty tuple variants

Unit variants and empty tuple variants are serialized into a single fixint whose value is determined by the tag.

```rust
#[derive(Serialize, Deserialize)]
enum E {
    #[tag = 3]
    Foo
}

let e = E::Foo;
let b = b"\x03"; // 1 byte; `3`

assert_eq!(serialize(&e), b);
assert_eq!(e, deserialize(b).unwrap());
```

```rust
#[derive(Serialize, Deserialize)]
enum E {
    #[tag = 3]
    Foo()
}

let e = E::Foo();
let b = b"\x03"; // 1 byte; `3`

assert_eq!(serialize(&e), b);
assert_eq!(e, deserialize(b).unwrap());
```

### Newtype variants

Newtype variants (one-element tuple variants) are serialized into an array of the tag and the inner value.

```rust
#[derive(Serialize, Deserialize)]
enum E {
    #[tag = 3]
    Foo(u32)
}

let e = E::Foo(42);
let b = b"\x92\x03\x2A"; // 3 bytes; `[ 3, 42 ]`

assert_eq!(serialize(&e), b);
assert_eq!(e, deserialize(b).unwrap());
```

### Untagged variants

Enums may be attached `#[untagged]` when all variants are newtype variants.
Serializing untagged variants results in the same data layout as the inner type.
The deserializer deserializes into an untagged enum type by trying deserization one by one from the first variant to the last.

```rust
#[derive(Serialize, Deserialize)]
#[untagged]
enum E {
    Foo(String),
    Bar(u32),
}

let e = E::Bar(42);
let b = b"\x2A"; // 1 byte; `42`

assert_eq!(serialize(&e), b);
assert_eq!(e, deserialize(b).unwrap());
```

## Appendix: Cheatsheet

<table>
    <tr>
        <th>schema</th>
        <th>Rust</th>
        <th>MessagePack (human readable)</th>
    </tr>
    <tr>
        <td>
            <pre><code>struct S {
    #[tag = 0]
    x: u32,
    #[tag = 1]
    y: bool,
}
</code></pre>
        </td>
        <td>
            <code>S { x: 42, y: true }</code>
        </td>
        <td>
            <code>{ 0: 42, 1: true }</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>struct S {
    #[optional]
    #[tag = 0]
    x: Option&lt;u32&gt;,
}
</code></pre>
        </td>
        <td>
            <code>S { x: Some(42) }</code>
        </td>
        <td>
            <code>{ 0: 42 }</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>struct S {
    #[optional]
    #[tag = 0]
    x: Option&lt;u32&gt;,
}
</code></pre>
        </td>
        <td>
            <code>S { x: None }</code>
        </td>
        <td>
            <code>{}</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>#[untagged]
struct S {
    #[tag = 0]
    x: u32,
    #[tag = 1]
    y: bool,
}
</code></pre>
        </td>
        <td>
            <code>S { x: 42, y: true }</code>
        </td>
        <td>
            <code>[ 42, true ]</code>
        </td>
    </tr>
    <tr>
        <td>
            <code>struct S(u32)</code>
        </td>
        <td>
            <code>S(42)</code>
        </td>
        <td>
            <code>42</code>
        </td>
    </tr>
    <tr>
        <td>
            <code>struct S</code>
        </td>
        <td>
            <code>S</code>
        </td>
        <td>UNSUPPORTED</td>
    </tr>
    <tr>
        <td>
            <code>struct S()</code>
        </td>
        <td>
            <code>S()</code>
        </td>
        <td>UNSUPPORTED</td>
    </tr>
    <tr>
        <td>
            <code>struct S(u32, bool)</code>
        </td>
        <td>
            <code>S(42, true)</code>
        </td>
        <td>
            <code>[ 42, true ]</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>enum E {
    #[tag = 3]
    Foo
}</code></pre>
        </td>
        <td>
            <code>E::Foo</code>
        </td>
        <td>
            <code>3</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>enum E {
    #[tag = 3]
    Foo()
}</code></pre>
        </td>
        <td>
            <code>E::Foo()</code>
        </td>
        <td>
            <code>3</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>enum E {
    #[tag = 3]
    Foo(u32)
}</code></pre>
        </td>
        <td>
            <code>E::Foo(42)</code>
        </td>
        <td>
            <code>[ 3, 42 ]</code>
        </td>
    </tr>
    <tr>
        <td>
            <pre><code>#[untagged]
enum E {
    Foo(u32)
    Bar(bool)
}</code></pre>
        </td>
        <td>
            <code>E::Bar(true)</code>
        </td>
        <td>
            <code>true</code>
        </td>
    </tr>
</table>

<!-- cargo-rdme end -->

#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in msgpack-schema by you shall be licensed as above, without any additional terms or conditions.
</sub>
