# msgpack-schema [![Crates.io](https://img.shields.io/crates/v/msgpack-schema)](https://crates.io/crates/msgpack-schema) [![docs.rs](https://img.shields.io/docsrs/msgpack-schema)](https://docs.rs/msgpack-schema/)

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

## Behaviours of serializers and deserializers

### Some general rules

- The deserializer ignores irrelevant key-value pairs in MsgPack map objects.
- MsgPack map objects must not have duplicate keys.
- `Option<T>` is roughly equal to declaring `T | null` in TypeScript. Deserializer interprets `nil` as `None` whatever `T` is. So `Option<Option<T>>` is the same as `Option<T>` (unless used together with `#[optional]`.)


### Structs with named fields

Structs with named fields will be serialized into a MsgPack map object whose keys are fixints specified by `#[tag]` attributes.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<td>

```rust
struct S {
    #[tag = 0]
    foo: u32,
    #[tag = 1]
    bar: String,
}
```

</td>
<td>

```rust
S { foo: 42, bar: "hello".to_owned() }
```

</td>
<td>

```js
{ 0: 42, 1: "hello" }
```

</td>
</tr>
</table>

Fields in named structs may be tagged with `#[optional]`.

- The tagged field must be of type `Option<T>`.
- On serialization, the key-value pair will not be included in the result map object when the field data contains `None`.
- On deserialization, the field of the result struct will be filled with `None` when the given MsgPack map object contains no corresponding key-value pair.

### Untagged structs with named fields

Structs with named fields may be attached `#[untagged]`.
Untagged structs are serialized into an array and will not contain tags.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<td>

```rust
#[untagged]
struct S {
    foo: u32,
    bar: String,
}
```

</td>
<td>

```rust
S { foo: 42, bar: "hello".to_owned() }
```

</td>
<td>

```js
[ 42, "hello" ]
```

</td>
</tr>
</table>

### Newtype structs

Tuple structs with only one element are treated transparently.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<tr>
<td>

```rust
struct S(u32)
```

</td>
<td>

```rust
S(42)
```

</td>
<td>

```js
42
```

</td>
</tr>
</table>

### Unit structs and empty tuple structs

Serialization and deserialization of unit structs and empty tuple structs are currently unsupported.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<tr>
<td>

```rust
struct S
```

</td>
<td>

```rust
S
```

</td>
<td>

UNSUPPORTED

</td>
</tr>
<tr>
<td>

```rust
struct S()
```

</td>
<td>

```rust
S()
```

</td>
<td>

UNSUPPORTED

</td>
</tr>
</table>

### Unit variants and empty tuple variants

Unit variants and empty tuple variants are serialized into a single fixint whose value is determined by the tag.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<tr>
<td>

```rust
enum E {
    #[tag = 3]
    Foo
}
```

</td>
<td>

```rust
E::Foo
```

</td>
<td>

```js
3
```

</td>
</tr>

<tr>
<td>

```rust
enum E {
    #[tag = 3]
    Foo()
}
```

</td>
<td>

```rust
E::Foo()
```

</td>
<td>

```js
3
```

</td>
</tr>
</table>

### Newtype variants

Newtype variants (one-element tuple variants) are serialized into an array of the tag and the inner value.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<tr>
<td>

```rust
enum E {
    #[tag = 3]
    Foo(u32)
}
```

</td>
<td>

```rust
E::Foo(42)
```

</td>
<td>

```js
[ 3, 42 ]
```

</td>
</tr>
</table>

### Untagged variants

Enums may be attached `#[untagged]` when all variants are newtype variants.
Serializing untagged variants results in the same data layout as the inner type.
The deserializer deserializes into an untagged enum type by trying deserization one by one from the first variant to the last.

<table>
<tr>
<th>
schema
</th>
<th>
Rust
</th>
<th>
MessagePack
</th>
</tr>
<tr>
<tr>
<td>

```rust
#[derive(Serialize, Deserialize)]
#[untagged]
enum Animal {
    Foo(String),
    Bar(u32),
}
```

</td>
<td>

```rust
E::Bar(42)
```

</td>
<td>

```js
42
```

</td>
</tr>
</table>

## TODO

- nonempty tuple structs
- tuple variants
- variants with named fields
- check duplicated tags
- `#[required]`

#### License

<sup>
Licensed under <a href="LICENSE-MIT">MIT license</a>.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in msgpack-schema by you shall be licensed as above, without any additional terms or conditions.
</sub>
