use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use regex::{Regex, RegexBuilder};
use std::any::Any;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
pub struct Parser {
    exact_re: Regex,
    search_re: Regex,
    field_map: HashMap<String, usize>,
    field_types: HashMap<String, String>,
    type_converters: HashMap<String, Box<dyn TypeConverter>>,
}

#[derive(Debug)]
pub struct ParseResult {
    converted: Vec<Box<dyn Any>>,
    field_map: HashMap<String, usize>,
}

impl ParseResult {
    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        self.converted
            .get(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn named<T: 'static>(&self, name: &str) -> Option<&T> {
        if let Some(&index) = self.field_map.get(name) {
            self.converted
                .get(index)
                .and_then(|value| value.downcast_ref::<T>())
        } else {
            None
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid format string")]
    InvalidFormat,
    #[error("no match found")]
    NoMatch,
    #[error("type conversion failed")]
    TypeConversionFailed,
}

// Type conversion traits
pub trait TypeConverter: Send + Sync + std::fmt::Debug {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError>;
    fn get_pattern(&self) -> Option<&str> {
        None
    }
}

// Built-in type converters
#[derive(Debug, Clone)]
pub struct IntConverter;
impl TypeConverter for IntConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        s.parse::<i64>()
            .map(|n| Box::new(n) as Box<dyn std::any::Any>)
            .map_err(|_| ParseError::TypeConversionFailed)
    }

    fn get_pattern(&self) -> Option<&str> {
        Some(r"-?\d+")
    }
}

#[derive(Debug, Clone)]
pub struct FloatConverter;
impl TypeConverter for FloatConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        s.parse::<f64>()
            .map(|n| Box::new(n) as Box<dyn std::any::Any>)
            .map_err(|_| ParseError::TypeConversionFailed)
    }

    fn get_pattern(&self) -> Option<&str> {
        Some(r"-?\d*\.?\d+")
    }
}

#[derive(Debug, Clone)]
pub struct WordConverter;
impl TypeConverter for WordConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        Ok(Box::new(s.to_string()))
    }

    fn get_pattern(&self) -> Option<&str> {
        Some(r"\w+")
    }
}

#[derive(Debug, Clone)]
pub struct DateTimeConverter {
    format_type: String,
}
impl TypeConverter for DateTimeConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various datetime formats
        let formats = match self.format_type.as_str() {
            // Generic date/time format (tg)
            "tg" => vec![
                // Date and time formats
                "%d/%m/%Y %H:%M:%S",    // 27/12/2024 19:57:55
                "%d/%m/%Y %H:%M",       // 27/12/2024 19:57
                "%d/%m/%Y %I:%M:%S %p", // 27/12/2024 07:57:55 PM
                "%d/%m/%Y %I:%M %p",    // 27/12/2024 07:57 PM
                "%Y/%m/%d %H:%M:%S",    // 2024/12/27 19:57:55
                "%Y/%m/%d %H:%M",       // 2024/12/27 19:57
                "%Y/%m/%d %I:%M:%S %p", // 2024/12/27 07:57:55 PM
                "%Y/%m/%d %I:%M %p",    // 2024/12/27 07:57 PM
                // Date only formats
                "%d/%m/%Y", // 27/12/2024
                "%Y/%m/%d", // 2024/12/27
                // Time only formats
                "%H:%M:%S",    // 19:57:55
                "%H:%M",       // 19:57
                "%I:%M:%S %p", // 07:57:55 PM
                "%I:%M %p",    // 07:57 PM
            ],

            // American date/time format (ta)
            "ta" => vec![
                "%m/%d/%Y %I:%M:%S %p", // 12/27/2024 07:57:55 PM
                "%m/%d/%Y %I:%M %p",    // 12/27/2024 07:57 PM
                "%m/%d/%Y %H:%M:%S",    // 12/27/2024 19:57:55
                "%m/%d/%Y %H:%M",       // 12/27/2024 19:57
                "%m/%d/%Y",             // 12/27/2024
            ],

            // Email date/time format (te)
            "te" => vec![
                "%a, %d %b %Y %H:%M:%S %z", // Fri, 27 Dec 2024 19:57:55 +0000
                "%d %b %Y %H:%M:%S %z",     // 27 Dec 2024 19:57:55 +0000
                "%d %b %Y",                 // 27 Dec 2024
            ],

            // HTTP log format (th)
            "th" => vec![
                "%d/%b/%Y:%H:%M:%S %z", // 27/Dec/2024:19:57:55 +0000
            ],

            // System log format (ts)
            "ts" => vec![
                "%b %d %Y %H:%M:%S", // Dec 27 2024 19:57:55
            ],

            // ISO format (ti)
            "ti" => vec![
                "%Y-%m-%dT%H:%M:%S%.3f%:z", // 2024-12-27T19:57:55.000+00:00
                "%Y-%m-%dT%H:%M:%S%:z",     // 2024-12-27T19:57:55+00:00
                "%Y-%m-%dT%H:%M:%S%.3f",    // 2024-12-27T19:57:55.000
                "%Y-%m-%dT%H:%M:%S",        // 2024-12-27T19:57:55
                "%Y-%m-%d",                 // 2024-12-27
            ],

            _ => return Err(ParseError::TypeConversionFailed),
        };

        // Try to parse using any of the supported formats
        for format in &formats {
            match format {
                f if f.contains("%z") || f.contains("%:z") => {
                    if let Ok(dt) = DateTime::parse_from_str(s, format) {
                        return Ok(Box::new(dt.naive_utc()));
                    }
                }
                _ => {
                    if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
                        return Ok(Box::new(dt));
                    }
                }
            }
        }

        // Try parsing as NaiveDate for date-only formats
        for format in &formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, format) {
                return Ok(Box::new(d));
            }
        }

        // Try parsing as NaiveTime for time-only formats
        for format in &formats {
            if let Ok(t) = NaiveTime::parse_from_str(s, format) {
                return Ok(Box::new(t));
            }
        }

        Err(ParseError::TypeConversionFailed)
    }

    fn get_pattern(&self) -> Option<&str> {
        match self.format_type.as_str() {
            "tg" => Some(
                r"\d{1,2}/\d{1,2}/\d{4}(?:\s+\d{1,2}:\d{2}(?::\d{2})?(?:\s*(?:AM|PM))?)?|\d{4}/\d{1,2}/\d{1,2}(?:\s+\d{1,2}:\d{2}(?::\d{2})?(?:\s*(?:AM|PM))?)?|\d{1,2}:\d{2}(?::\d{2})?(?:\s*(?:AM|PM))?",
            ),
            "ta" => Some(r"\d{1,2}/\d{1,2}/\d{4}(?:\s+\d{1,2}:\d{2}(?::\d{2})?(?:\s*(?:AM|PM))?)?"),
            "te" => Some(
                r"(?:[A-Za-z]{3},\s+)?\d{1,2}\s+[A-Za-z]{3}\s+\d{4}(?:\s+\d{2}:\d{2}:\d{2}\s+[-+]\d{4})?",
            ),
            "th" => Some(r"\d{2}/[A-Za-z]{3}/\d{4}:\d{2}:\d{2}:\d{2}\s+[-+]\d{4}"),
            "ts" => Some(r"[A-Za-z]{3}\s+\d{1,2}\s+\d{4}\s+\d{2}:\d{2}:\d{2}"),
            "ti" => Some(
                r"\d{4}-\d{1,2}-\d{1,2}(?:T\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)?",
            ),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateConverter;
impl TypeConverter for DateConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various date formats
        let formats = [
            // Standard date formats
            "%Y-%m-%d", // 2024-12-27
            "%Y/%m/%d", // 2024/12/27
            // Generic formats
            "%d/%m/%Y", // 27/12/2024
            "%d-%m-%Y", // 27-12-2024
            // US formats
            "%m/%d/%Y", // 12/27/2024
            "%m-%d-%Y", // 12-27-2024
            // Text month formats
            "%d %b %Y",  // 27 Dec 2024
            "%d %B %Y",  // 27 December 2024
            "%b %d, %Y", // Dec 27, 2024
            "%B %d, %Y", // December 27, 2024
            "%d-%b-%Y",  // 27-Dec-2024
            // Compact format
            "%Y%m%d", // 20241227
        ];

        for format in formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, format) {
                return Ok(Box::new(d));
            }
        }

        Err(ParseError::TypeConversionFailed)
    }

    fn get_pattern(&self) -> Option<&str> {
        Some(
            r"(?:(?:19|20)\d\d[-/](?:0[1-9]|1[0-2])[-/](?:0[1-9]|[12]\d|3[01])|(?:0[1-9]|[12]\d|3[01])[-/](?:0[1-9]|1[0-2])[-/](?:19|20)\d\d|(?:0[1-9]|[12]\d|3[01])(?:\s+|-)?(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)(?:\s*,\s*|\s+|-)?(?:19|20)\d\d|(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01]))",
        )
    }
}

#[derive(Debug, Clone)]
pub struct TimeConverter;
impl TypeConverter for TimeConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various time formats
        let formats = [
            // Standard time formats
            "%H:%M:%S",       // 19:57:55
            "%H:%M",          // 19:57
            "%I:%M:%S %p",    // 07:57:55 PM
            "%I:%M %p",       // 07:57 PM
            "%H:%M:%S %z",    // 19:57:55 +0000
            "%I:%M:%S %p %z", // 07:57:55 PM +0000
        ];

        for format in formats {
            if let Ok(t) = NaiveTime::parse_from_str(s, format) {
                return Ok(Box::new(t));
            }
        }

        Err(ParseError::TypeConversionFailed)
    }

    fn get_pattern(&self) -> Option<&str> {
        Some(r"(?:[01]\d|2[0-3]):[0-5]\d(?::[0-5]\d)?(?:\s*[AaPp][Mm])?(?:\s*[-+]\d{2}:?\d{2})?")
    }
}

lazy_static::lazy_static! {
    static ref DEFAULT_TYPES: HashMap<String, Box<dyn TypeConverter>> = {
        let mut m = HashMap::new();
        m.insert("d".to_string(), Box::new(IntConverter) as Box<dyn TypeConverter>);
        m.insert("f".to_string(), Box::new(FloatConverter) as Box<dyn TypeConverter>);
        m.insert("w".to_string(), Box::new(WordConverter) as Box<dyn TypeConverter>);
        m.insert("tg".to_string(), Box::new(DateTimeConverter { format_type: "tg".to_string() }) as Box<dyn TypeConverter>);
        m.insert("ta".to_string(), Box::new(DateTimeConverter { format_type: "ta".to_string() }) as Box<dyn TypeConverter>);
        m.insert("te".to_string(), Box::new(DateTimeConverter { format_type: "te".to_string() }) as Box<dyn TypeConverter>);
        m.insert("th".to_string(), Box::new(DateTimeConverter { format_type: "th".to_string() }) as Box<dyn TypeConverter>);
        m.insert("ts".to_string(), Box::new(DateTimeConverter { format_type: "ts".to_string() }) as Box<dyn TypeConverter>);
        m.insert("ti".to_string(), Box::new(DateTimeConverter { format_type: "ti".to_string() }) as Box<dyn TypeConverter>);
        m
    };
}

impl Parser {
    pub fn new_with_types(
        format: &str,
        case_sensitive: bool,
        extra_types: HashMap<String, Box<dyn TypeConverter>>,
    ) -> Result<Self, ParseError> {
        let (exact_pattern, search_pattern, field_map, field_types) =
            Self::parse_format(format, &extra_types)?;
        let flags = if case_sensitive {
            RegexBuilder::new(&format!("^{}$", exact_pattern))
        } else {
            let mut builder = RegexBuilder::new(&format!("^{}$", exact_pattern));
            builder.case_insensitive(true);
            builder
        };
        let exact_re = flags.build().map_err(|_| ParseError::InvalidFormat)?;

        let flags = if case_sensitive {
            RegexBuilder::new(&search_pattern)
        } else {
            let mut builder = RegexBuilder::new(&search_pattern);
            builder.case_insensitive(true);
            builder
        };
        let search_re = flags.build().map_err(|_| ParseError::InvalidFormat)?;

        Ok(Parser {
            exact_re,
            search_re,
            field_map,
            field_types,
            type_converters: extra_types,
        })
    }

    pub fn new(format: &str, case_sensitive: bool) -> Result<Self, ParseError> {
        let type_converters = Self::get_default_type_converters();
        Self::new_with_types(format, case_sensitive, type_converters)
    }

    fn get_default_type_converters() -> HashMap<String, Box<dyn TypeConverter>> {
        let mut default_types = HashMap::new();
        for k in DEFAULT_TYPES.keys() {
            if let Some(converter) = match k.as_str() {
                "d" => Some(Box::new(IntConverter) as Box<dyn TypeConverter>),
                "f" => Some(Box::new(FloatConverter) as Box<dyn TypeConverter>),
                "w" => Some(Box::new(WordConverter) as Box<dyn TypeConverter>),
                "tg" => Some(Box::new(DateTimeConverter {
                    format_type: "tg".to_string(),
                }) as Box<dyn TypeConverter>),
                "ta" => Some(Box::new(DateTimeConverter {
                    format_type: "ta".to_string(),
                }) as Box<dyn TypeConverter>),
                "te" => Some(Box::new(DateTimeConverter {
                    format_type: "te".to_string(),
                }) as Box<dyn TypeConverter>),
                "th" => Some(Box::new(DateTimeConverter {
                    format_type: "th".to_string(),
                }) as Box<dyn TypeConverter>),
                "ts" => Some(Box::new(DateTimeConverter {
                    format_type: "ts".to_string(),
                }) as Box<dyn TypeConverter>),
                "ti" => Some(Box::new(DateTimeConverter {
                    format_type: "ti".to_string(),
                }) as Box<dyn TypeConverter>),
                _ => None,
            } {
                default_types.insert(k.clone(), converter);
            }
        }
        default_types
    }

    fn process_captures(&self, caps: &regex::Captures) -> Result<ParseResult, ParseError> {
        let mut converted = Vec::with_capacity(self.field_map.len());
        let mut field_map = HashMap::new();

        for i in 0..caps.len() - 1 {
            // Skip group 0 (whole match)
            if let Some(m) = caps.get(i + 1) {
                let value = m.as_str();

                // Find the field name for this group index
                let field_name = self
                    .field_map
                    .iter()
                    .find(|(_, &idx)| idx == i + 1)
                    .map(|(name, _)| name.clone())
                    .unwrap();

                // Convert value if type is specified
                if let Some(type_name) = self.field_types.get(&field_name) {
                    if let Some(converter) = self.type_converters.get(type_name) {
                        match converter.convert(value) {
                            Ok(converted_value) => {
                                field_map.insert(field_name.clone(), converted.len());
                                converted.push(converted_value);
                            }
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    // No type specified, store as string
                    field_map.insert(field_name.clone(), converted.len());
                    converted.push(Box::new(value.to_string()));
                }
            }
        }

        Ok(ParseResult {
            converted,
            field_map,
        })
    }

    fn parse_format(
        format: &str,
        type_converters: &HashMap<String, Box<dyn TypeConverter>>,
    ) -> Result<
        (
            String,
            String,
            HashMap<String, usize>,
            HashMap<String, String>,
        ),
        ParseError,
    > {
        let mut field_map = HashMap::new();
        let mut field_types = HashMap::new();
        let mut group_count = 0;

        let mut in_field = false;
        let mut in_type = false;
        let mut current_field = String::new();
        let mut current_type = String::new();
        let mut chars = format.chars().peekable();
        let mut pattern = String::new();
        let mut brace_count = 0;

        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        pattern.push_str("\\{");
                    } else {
                        if in_field {
                            return Err(ParseError::InvalidFormat);
                        }
                        in_field = true;
                        in_type = false;
                        current_field.clear();
                        current_type.clear();
                        brace_count += 1;
                    }
                }
                '}' => {
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        pattern.push_str("\\}");
                    } else if in_field {
                        in_field = false;
                        in_type = false;
                        group_count += 1;
                        brace_count -= 1;

                        // Get the pattern for the current type
                        let type_pattern = if !current_type.is_empty() {
                            if let Some(converter) = type_converters.get(&current_type) {
                                if let Some(type_pattern) = converter.get_pattern() {
                                    type_pattern
                                } else {
                                    r".*?"
                                }
                            } else {
                                return Err(ParseError::InvalidFormat);
                            }
                        } else {
                            r".*?"
                        };

                        // Add to field map before adding pattern
                        let field_name = if current_field.is_empty() {
                            (group_count - 1).to_string()
                        } else {
                            current_field.clone()
                        };

                        field_map.insert(field_name.clone(), group_count);
                        if !current_type.is_empty() {
                            field_types.insert(field_name, current_type.clone());
                        }

                        pattern.push_str(&format!("({})", type_pattern));
                    } else {
                        return Err(ParseError::InvalidFormat);
                    }
                }
                ':' if in_field => {
                    in_type = true;
                }
                _ => {
                    if in_field {
                        if in_type {
                            current_type.push(c);
                        } else {
                            current_field.push(c);
                        }
                    } else {
                        // Add optional whitespace around punctuation first
                        if c == ',' || c == '=' || c == '+' || c == '-' {
                            pattern.push_str(r"\s*");
                            // Then escape the character if needed
                            if "[](){}.*+?^$\\|".contains(c) {
                                pattern.push('\\');
                            }
                            pattern.push(c);
                            pattern.push_str(r"\s*");
                        } else {
                            // Escape special regex characters
                            if "[](){}.*+?^$\\|".contains(c) {
                                pattern.push('\\');
                            }
                            pattern.push(c);
                        }
                    }
                }
            }
        }

        if brace_count != 0 || in_field {
            return Err(ParseError::InvalidFormat);
        }

        Ok((pattern.clone(), pattern, field_map, field_types))
    }

    pub fn parse(&self, text: &str) -> Option<ParseResult> {
        self.exact_re
            .captures(text)
            .map(|captures| self.process_captures(&captures).ok())
            .flatten()
    }

    pub fn search(&self, text: &str) -> Option<ParseResult> {
        self.search_re
            .captures(text)
            .map(|captures| self.process_captures(&captures).ok())
            .flatten()
    }

    pub fn findall(&self, text: &str) -> Vec<ParseResult> {
        self.search_re
            .captures_iter(text)
            .filter_map(|captures| self.process_captures(&captures).ok())
            .collect()
    }
}

pub fn parse_with_types(
    format: &str,
    text: &str,
    extra_types: HashMap<String, Box<dyn TypeConverter>>,
) -> Option<ParseResult> {
    Parser::new_with_types(format, false, extra_types)
        .ok()?
        .parse(text)
}

pub fn findall_with_types(
    format: &str,
    text: &str,
    extra_types: HashMap<String, Box<dyn TypeConverter>>,
) -> Vec<ParseResult> {
    Parser::new_with_types(format, false, extra_types)
        .map(|p| p.findall(text))
        .unwrap_or_default()
}

pub fn search_with_types(
    format: &str,
    text: &str,
    extra_types: HashMap<String, Box<dyn TypeConverter>>,
) -> Option<ParseResult> {
    Parser::new_with_types(format, false, extra_types)
        .ok()?
        .search(text)
}

pub fn parse(format: &str, text: &str) -> Option<ParseResult> {
    Parser::new(format, false).ok()?.parse(text)
}

pub fn search(format: &str, text: &str) -> Option<ParseResult> {
    Parser::new(format, false).ok()?.search(text)
}

pub fn findall(format: &str, text: &str) -> Vec<ParseResult> {
    Parser::new(format, false)
        .map(|p| p.findall(text))
        .unwrap_or_default()
}
