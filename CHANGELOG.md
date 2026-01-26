# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/inferadb/teapot/compare/v0.1.0...HEAD)

### Added

- `bon` crate dependency (v3.8) for type-safe compile-time builders

### Changed

- `TaskProgressView::builder()` now uses bon-generated builder
  - Replace `.auto_start()` with `.auto_start(true)`
  - Replace `.external_control()` with `.external_control(true)`
- `Style` now uses `#[derive(bon::Builder)]` - use `Style::builder()` instead of `Style::new()`
  - Example: `Style::builder().bold(true).foreground(Color::Red).build()`
- Form field builders now use bon-generated `Field::*()` constructors:
  - `InputField::new("key")` → `Field::input().key("key").build()`
  - `SelectField::new("key")` → `Field::select().key("key").build()`
  - `MultiSelectField::new("key")` → `Field::multi_select().key("key").build()`
  - `ConfirmField::new("key")` → `Field::confirm().key("key").build()`
  - `NoteField::new()` → `Field::note().build()`
  - `FilePickerField::new("key")` → `Field::file_picker().key("key").build()`

### Deprecated

- `InputField` struct - use `Field::input()` builder instead
- `SelectField` struct - use `Field::select()` builder instead
- `MultiSelectField` struct - use `Field::multi_select()` builder instead
- `ConfirmField` struct - use `Field::confirm()` builder instead
- `NoteField` struct - use `Field::note()` builder instead
- `FilePickerField` struct - use `Field::file_picker()` builder instead

## [0.1.0-alpha.1](https://github.com/inferadb/teapot/releases/tag/v0.1.0-alpha.1)

### Added

- Experimental first release of Teapot
