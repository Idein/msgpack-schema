# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][https://keepachangelog.com/en/1.0.0/] and this project adheres to [Semantic Versioning][https://semver.org/spec/v2.0.0.html].

## [Unreleased]

### Added

### Fixed

### Removed

### Changed

---

## [Released]

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