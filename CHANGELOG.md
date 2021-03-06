# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Fixed

### Removed

### Changed

---

## [Released]

## 0.4.2 - 2022-01-25

### Changed

- Bump Rust edition to 2021 (no user-facing changes)

## 0.4.1 - 2021-07-29

### Added

- Add `value::Empty`
- Support tuple structs (with more than one field)

## 0.4.0 - 2021-07-19

### Added

- Report errors when there are attributes in an invalid position
- (experimental) Add `schema` attribute
- Support `#[flatten]`

### Removed

- Remove `Serialize` and `Deserialize` impls for `Option<T>`

## 0.3.1 - 2021-06-21

### Fixed

- Fix Cargo dependency bound.

## 0.3.0 - 2021-06-17

### Added

- Add `Ext` type.
- Support untagged structs.

### Removed

- Hide `msgpack` macro.
- All inherent methods of `Token`.

### Changed

- Change the types of `serialize` and `deserialize`.
- `Serialize` never throws an error.
- `DeserializeError` no longer contains detailed information.
- `Deserializer` and `Serializer` are now structs.

## 0.2.1 - 2021-06-15

### Added

- Restore `DeserializeError::InvalidValue` as a unit variant.

## 0.2.0 - 2021-06-15

### Removed

- Remove `DeserializeError::InvalidValue` to allow the `DeserializeError` type to be `Send` and `Sync`.

## 0.1.6 - 2021-06-12

### Added

- Add `msgpack` macro.

### Removed

- Remove `BinarySerializer` and `BinaryDeserializer`. Use `serialize` and `deserialize` instead.

## 0.1.5 - 2021-06-12

### Fixed

- Fix serialization of 0-length map value
- Fix deserialization of FixExt8 and FixExt16

## 0.1.4 - 2021-06-12

### Fixed

- Fix doc test failure.

## 0.1.3 - 2021-06-12

This release mainly includes doc improvements and typo fixes.

### Removed

- `Value`
- `TryFromValueError`
- `value::{Serializer, Deserializer, serialize, deserialize}`

### Changed

- Move `Any` to `value::Any`
- Move `Nil` to `value::Nil`

## 0.1.2 - 2021-06-10

### Added

- Add a blanket implementation `impl<T: Serialize> Serialize for &T`

## 0.1.1 - 2021-06-08

Initial version

---