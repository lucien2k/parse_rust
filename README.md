# parse_rust

A Rust implementation of Python's `parse` library, providing a flexible way to parse strings using simple, human-readable format strings.

## Features

- Parse strings using format strings similar to Python's `str.format()`
- Extract typed values from strings
- Support for various data types:
  - Integers
  - Floats
  - Words (strings)
  - Custom types
  - Date and Time formats

## Date and Time Format Specifiers

The library supports various datetime format specifiers:

- `:tg` - Generic date/time format
  ```
  27/12/2024 19:57:55
  27/12/2024 07:57:55 PM
  2024/12/27 19:57:55
  27/12/2024
  19:57:55
  ```

- `:ta` - American date/time format
  ```
  12/27/2024 07:57:55 PM
  12/27/2024 19:57:55
  12/27/2024
  ```

- `:te` - Email date/time format (RFC 2822)
  ```
  Fri, 27 Dec 2024 19:57:55 +0000
  27 Dec 2024 19:57:55 +0000
  27 Dec 2024
  ```

- `:th` - HTTP log date/time format
  ```
  27/Dec/2024:19:57:55 +0000
  ```

- `:ts` - System log date/time format
  ```
  Dec 27 2024 19:57:55
  ```

- `:ti` - ISO 8601 date/time format
  ```
  2024-12-27T19:57:55.000+00:00
  2024-12-27T19:57:55+00:00
  2024-12-27T19:57:55.000
  2024-12-27T19:57:55
  2024-12-27
  ```

## Usage

```rust
use parse_rust::Parser;

// Parse a simple string with an integer
let p = Parser::new("Value is {:d}", true).unwrap();
let result = p.parse("Value is 42").unwrap();
let value: &i64 = result.get(0).unwrap();
assert_eq!(*value, 42);

// Parse a datetime string
let p = Parser::new("Time: {:tg}", true).unwrap();
let result = p.parse("Time: 27/12/2024 19:57:55").unwrap();
let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
assert_eq!(dt.format("%Y-%m-%d %H:%M:%S").to_string(), "2024-12-27 19:57:55");

// Parse with named fields
let p = Parser::new("Name: {name:w}, Age: {age:d}", true).unwrap();
let result = p.parse("Name: John, Age: 30").unwrap();
let name: &str = result.named("name").unwrap();
let age: &i64 = result.named("age").unwrap();
assert_eq!(name, "John");
assert_eq!(*age, 30);
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
parse_rust = "0.1.0"
```

## Dependencies

- `regex` - For pattern matching
- `chrono` - For date and time parsing
- `thiserror` - For error handling
- `lazy_static` - For static initialization

## License

This project is licensed under the MIT License - see the LICENSE file for details.
