# parse_rust

A Rust implementation of Python's [parse](https://github.com/r1chardj0n3s/parse) library. This library provides a simple way to parse strings using format strings, similar to Python's `str.format()` but in reverse.

## Features

- Parse strings using format strings with named and unnamed fields
- Support for type conversion (integers, floats, words)
- Custom type converters
- Case-sensitive and case-insensitive matching
- Complex field names with dot notation and array indexing
- Search and findall functionality

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
parse_rust = "0.1.0"
```

### Basic Parsing

```rust
use parse_rust::parse;

// Simple parsing
let result = parse("{} {}", "hello world").unwrap();
assert_eq!(result.fixed, vec!["hello", "world"]);

// Named fields
let result = parse("{name} {age}", "John 25").unwrap();
assert_eq!(result.named["name"], "John");
assert_eq!(result.named["age"], "25");

// Type conversion
let result = parse("{name} {age:d}", "John 25").unwrap();
assert_eq!(result.named["name"], "John");
let age = result.converted["age"].downcast_ref::<i64>().unwrap();
assert_eq!(*age, 25);
```

### Built-in Type Converters

The library comes with three built-in type converters:

- `:d` - Converts to integers (i64)
- `:f` - Converts to floating point numbers (f64)
- `:w` - Converts to words (String)

```rust
// Integer conversion
let result = parse("{:d}", "123").unwrap();
let value = result.converted["0"].downcast_ref::<i64>().unwrap();
assert_eq!(*value, 123);

// Float conversion
let result = parse("{:f}", "123.45").unwrap();
let value = result.converted["0"].downcast_ref::<f64>().unwrap();
assert_eq!(*value, 123.45);

// Word conversion
let result = parse("{:w}", "hello123").unwrap();
let value = result.converted["0"].downcast_ref::<String>().unwrap();
assert_eq!(value, "hello123");
```

### Custom Type Converters

You can create custom type converters by implementing the `TypeConverter` trait:

```rust
use parse_rust::{TypeConverter, ParseError};
use std::collections::HashMap;

#[derive(Debug)]
struct HexConverter;

impl TypeConverter for HexConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        i64::from_str_radix(s.trim_start_matches("0x"), 16)
            .map(|n| Box::new(n) as Box<dyn std::any::Any>)
            .map_err(|_| ParseError::TypeConversionFailed)
    }
    
    fn get_pattern(&self) -> Option<&str> {
        Some(r"0x[0-9a-fA-F]+")
    }
}

let mut extra_types = HashMap::new();
extra_types.insert("hex".to_string(), Box::new(HexConverter) as Box<dyn TypeConverter>);

let result = parse_with_types("{:hex}", "0x1F", extra_types).unwrap();
let value = result.converted["0"].downcast_ref::<i64>().unwrap();
assert_eq!(*value, 31);
```

### Complex Field Names

The library supports dot notation and array indexing in field names:

```rust
let result = parse("{person.name} {array[0]}", "John 123").unwrap();
assert_eq!(result.named["person__name"], "John");
assert_eq!(result.named["array__0"], "123");
```

### Search and FindAll

Besides exact matching with `parse()`, you can also search for patterns within text:

```rust
use parse_rust::{search, findall};

// Search for first occurrence
let result = search("Age: {:d}", "Name: John, Age: 25, Height: 180").unwrap();
let age = result.converted["0"].downcast_ref::<i64>().unwrap();
assert_eq!(*age, 25);

// Find all occurrences
let results = findall("{:d}", "Numbers: 1, 2, 3, 4, 5");
assert_eq!(results.len(), 5);
```

### Case Sensitivity

By default, parsing is case-insensitive. For case-sensitive parsing, use the Parser directly:

```rust
use parse_rust::Parser;

let parser = Parser::new("Hello {:w}", true).unwrap();  // case_sensitive = true
assert!(parser.parse("hello world").is_none());  // Won't match due to case
assert!(parser.parse("Hello world").is_some());  // Matches
```

## Error Handling

The library uses a custom `ParseError` enum for error handling:

```rust
pub enum ParseError {
    InvalidFormat,      // The format string is invalid
    NoMatch,           // No match found
    TypeConversionFailed, // Type conversion failed
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
