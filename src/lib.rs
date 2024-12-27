use std::collections::HashMap;
use regex::{Regex, RegexBuilder};
use thiserror::Error;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};
use std::any::Any;

#[derive(Debug)]
pub struct Parser {
    pattern: Regex,
    search_pattern: Regex,
    field_map: HashMap<String, usize>,
    field_types: HashMap<String, String>,
    type_converters: HashMap<String, Box<dyn TypeConverter>>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub fixed: Vec<String>,
    pub named: HashMap<String, String>,
    pub spans: Vec<(usize, usize)>,
    pub converted: Vec<Box<dyn Any>>,
}

impl ParseResult {
    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        self.converted.get(index).and_then(|value| value.downcast_ref::<T>())
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
    fn get_pattern(&self) -> Option<&str> { None }
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
pub struct DateTimeConverter;
impl TypeConverter for DateTimeConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various datetime formats
        let formats = [
            // Standard formats
            "%Y-%m-%d %H:%M:%S",     // 2024-12-27 19:57:55
            "%Y-%m-%d %H:%M",        // 2024-12-27 19:57
            "%Y-%m-%dT%H:%M:%S",     // 2024-12-27T19:57:55
            "%Y-%m-%dT%H:%M:%SZ",    // 2024-12-27T19:57:55Z
            "%Y-%m-%d %H:%M:%S%.f",  // 2024-12-27 19:57:55.123
            
            // Generic date/time formats (tg)
            "%Y/%m/%d %I:%M:%S %p",  // 2024/12/27 07:57:55 PM
            "%Y/%m/%d %I:%M %p",     // 2024/12/27 07:57 PM
            "%d/%m/%Y %H:%M:%S",     // 27/12/2024 19:57:55
            "%d/%m/%Y %H:%M",        // 27/12/2024 19:57
            
            // US date/time formats (ta)
            "%m/%d/%Y %I:%M:%S %p",  // 12/27/2024 07:57:55 PM
            "%m/%d/%Y %I:%M %p",     // 12/27/2024 07:57 PM
            
            // Email date/time format (te)
            "%a, %d %b %Y %H:%M:%S %z", // Fri, 27 Dec 2024 19:57:55 +0000
            "%d %b %Y %H:%M:%S %z",     // 27 Dec 2024 19:57:55 +0000
            
            // HTTP log format (th)
            "%d/%b/%Y:%H:%M:%S %z",     // 27/Dec/2024:19:57:55 +0000
            
            // Linux system log format (ts)
            "%b %e %H:%M:%S",           // Dec 27 19:57:55
        ];
        
        // Try to parse using any of the supported formats
        for format in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
                return Ok(Box::new(dt));
            }
        }
        
        Err(ParseError::TypeConversionFailed)
    }
    
    fn get_pattern(&self) -> Option<&str> {
        Some(r"(?:19|20)\d\d[-/](?:0[1-9]|1[0-2])[-/](?:0[1-9]|[12]\d|3[01])(?:[T ](?:[01]\d|2[0-3]):[0-5]\d(?::[0-5]\d(?:\.\d+)?)?(?:Z|[-+]\d{2}:?\d{2})?)?")
    }
}

#[derive(Debug, Clone)]
pub struct DateConverter;
impl TypeConverter for DateConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various date formats
        let formats = [
            // Standard date formats
            "%Y-%m-%d",      // 2024-12-27
            "%Y/%m/%d",      // 2024/12/27
            
            // Generic formats
            "%d/%m/%Y",      // 27/12/2024
            "%d-%m-%Y",      // 27-12-2024
            
            // US formats
            "%m/%d/%Y",      // 12/27/2024
            "%m-%d-%Y",      // 12-27-2024
            
            // Text month formats
            "%d %b %Y",      // 27 Dec 2024
            "%d %B %Y",      // 27 December 2024
            "%b %d, %Y",     // Dec 27, 2024
            "%B %d, %Y",     // December 27, 2024
            "%d-%b-%Y",      // 27-Dec-2024
            
            // Compact format
            "%Y%m%d",        // 20241227
        ];
        
        for format in formats {
            if let Ok(d) = NaiveDate::parse_from_str(s, format) {
                return Ok(Box::new(d));
            }
        }
        
        Err(ParseError::TypeConversionFailed)
    }
    
    fn get_pattern(&self) -> Option<&str> {
        Some(r"(?:(?:19|20)\d\d[-/](?:0[1-9]|1[0-2])[-/](?:0[1-9]|[12]\d|3[01])|(?:0[1-9]|[12]\d|3[01])[-/](?:0[1-9]|1[0-2])[-/](?:19|20)\d\d|(?:0[1-9]|1[0-2])[-/](?:0[1-9]|[12]\d|3[01])[-/](?:19|20)\d\d|(?:0[1-9]|[12]\d|3[01])(?:\s+|-)?(?:Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:tember)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)(?:\s*,\s*|\s+|-)?(?:19|20)\d\d|(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01]))")
    }
}

#[derive(Debug, Clone)]
pub struct TimeConverter;
impl TypeConverter for TimeConverter {
    fn convert(&self, s: &str) -> Result<Box<dyn std::any::Any>, ParseError> {
        // Try various time formats
        let formats = [
            // Standard time formats
            "%H:%M:%S",        // 19:57:55
            "%H:%M",           // 19:57
            "%I:%M:%S %p",     // 07:57:55 PM
            "%I:%M %p",        // 07:57 PM
            "%H:%M:%S %z",     // 19:57:55 +0000
            "%I:%M:%S %p %z",  // 07:57:55 PM +0000
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
        m.insert("datetime".to_string(), Box::new(DateTimeConverter) as Box<dyn TypeConverter>);
        m.insert("date".to_string(), Box::new(DateConverter) as Box<dyn TypeConverter>);
        m.insert("time".to_string(), Box::new(TimeConverter) as Box<dyn TypeConverter>);
        m
    };
}

impl Parser {
    fn parse_format(format: &str, type_converters: &HashMap<String, Box<dyn TypeConverter>>) -> Result<(String, String, HashMap<String, usize>, HashMap<String, String>), ParseError> {
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
                                    r"[^\s]+"
                                }
                            } else {
                                return Err(ParseError::InvalidFormat);
                            }
                        } else {
                            r"[^\s]+"
                        };
                        
                        // Add to field map before adding pattern
                        let field_name = if current_field.is_empty() {
                            (group_count - 1).to_string()
                        } else {
                            // Support dot notation and array indexing
                            current_field.replace(".", "__").replace("[", "__").replace("]", "")
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
                        pattern.push(c);
                    }
                }
            }
        }
        
        if brace_count != 0 || in_field {
            return Err(ParseError::InvalidFormat);
        }
        
        let exact_pattern = format!("^{}$", pattern);
        let search_pattern = pattern;
        
        Ok((exact_pattern, search_pattern, field_map, field_types))
    }

    pub fn new_with_types(format: &str, case_sensitive: bool, extra_types: HashMap<String, Box<dyn TypeConverter>>) -> Result<Self, ParseError> {
        // Merge default types with extra types
        let mut all_types = HashMap::new();
        for k in DEFAULT_TYPES.keys() {
            if !extra_types.contains_key(k) {
                if let Some(converter) = match k.as_str() {
                    "d" => Some(Box::new(IntConverter) as Box<dyn TypeConverter>),
                    "f" => Some(Box::new(FloatConverter) as Box<dyn TypeConverter>),
                    "w" => Some(Box::new(WordConverter) as Box<dyn TypeConverter>),
                    "datetime" => Some(Box::new(DateTimeConverter) as Box<dyn TypeConverter>),
                    "date" => Some(Box::new(DateConverter) as Box<dyn TypeConverter>),
                    "time" => Some(Box::new(TimeConverter) as Box<dyn TypeConverter>),
                    _ => None,
                } {
                    all_types.insert(k.clone(), converter);
                }
            }
        }
        all_types.extend(extra_types);
        
        let (pattern, search_pattern, field_map, field_types) = Self::parse_format(format, &all_types)?;
        
        let pattern = RegexBuilder::new(&pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|_| ParseError::InvalidFormat)?;
            
        let search_pattern = RegexBuilder::new(&search_pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|_| ParseError::InvalidFormat)?;
            
        Ok(Parser {
            pattern,
            search_pattern,
            field_map,
            field_types,
            type_converters: all_types,
        })
    }
    
    pub fn new(format: &str, case_sensitive: bool) -> Result<Self, ParseError> {
        let mut default_types = HashMap::new();
        for k in DEFAULT_TYPES.keys() {
            if let Some(converter) = match k.as_str() {
                "d" => Some(Box::new(IntConverter) as Box<dyn TypeConverter>),
                "f" => Some(Box::new(FloatConverter) as Box<dyn TypeConverter>),
                "w" => Some(Box::new(WordConverter) as Box<dyn TypeConverter>),
                "datetime" => Some(Box::new(DateTimeConverter) as Box<dyn TypeConverter>),
                "date" => Some(Box::new(DateConverter) as Box<dyn TypeConverter>),
                "time" => Some(Box::new(TimeConverter) as Box<dyn TypeConverter>),
                _ => None,
            } {
                default_types.insert(k.clone(), converter);
            }
        }
        Self::new_with_types(format, case_sensitive, default_types)
    }
    
    pub fn parse(&self, text: &str) -> Option<ParseResult> {
        self.pattern.captures(text).map(|caps| self.process_captures(&caps)).and_then(|r| r.ok())
    }
    
    pub fn search(&self, text: &str) -> Option<ParseResult> {
        self.search_pattern.captures(text).map(|caps| self.process_captures(&caps)).and_then(|r| r.ok())
    }
    
    pub fn findall(&self, text: &str) -> Vec<ParseResult> {
        self.search_pattern
            .captures_iter(text)
            .map(|caps| self.process_captures(&caps))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_default()
    }
    
    fn process_captures(&self, caps: &regex::Captures) -> Result<ParseResult, ParseError> {
        let mut fixed = Vec::new();
        let mut named = HashMap::new();
        let mut spans = Vec::new();
        let mut converted = Vec::with_capacity(self.field_map.len());
        
        // Initialize fixed with empty strings to preserve order
        fixed.resize(self.field_map.len(), String::new());
        
        // First pass: collect all values
        for (field_name, &group_idx) in &self.field_map {
            if let Some(m) = caps.get(group_idx) {
                let value = m.as_str().to_string();
                fixed[group_idx - 1] = value.clone();  // -1 because group 0 is the whole match
                named.insert(field_name.clone(), value);
                spans.push((m.start(), m.end()));
            }
        }
        
        // Second pass: convert values in order
        for i in 0..fixed.len() {
            for (field_name, &group_idx) in &self.field_map {
                if group_idx - 1 == i {  // -1 because group 0 is the whole match
                    if let Some(type_name) = self.field_types.get(field_name) {
                        if let Some(converter) = self.type_converters.get(type_name) {
                            match converter.convert(&fixed[i]) {
                                Ok(converted_value) => converted.push(converted_value),
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
            }
        }
        
        Ok(ParseResult {
            fixed,
            named,
            spans,
            converted,
        })
    }
}

pub fn parse_with_types(format: &str, text: &str, extra_types: HashMap<String, Box<dyn TypeConverter>>) -> Option<ParseResult> {
    Parser::new_with_types(format, false, extra_types).ok()?.parse(text)
}

pub fn search_with_types(format: &str, text: &str, extra_types: HashMap<String, Box<dyn TypeConverter>>) -> Option<ParseResult> {
    Parser::new_with_types(format, false, extra_types).ok()?.search(text)
}

pub fn findall_with_types(format: &str, text: &str, extra_types: HashMap<String, Box<dyn TypeConverter>>) -> Vec<ParseResult> {
    Parser::new_with_types(format, false, extra_types).ok()
        .map(|p| p.findall(text))
        .unwrap_or_default()
}

pub fn parse(format: &str, text: &str) -> Option<ParseResult> {
    Parser::new(format, false).ok()?.parse(text)
}

pub fn search(format: &str, text: &str) -> Option<ParseResult> {
    Parser::new(format, false).ok()?.search(text)
}

pub fn findall(format: &str, text: &str) -> Vec<ParseResult> {
    Parser::new(format, false).ok()
        .map(|p| p.findall(text))
        .unwrap_or_default()
}
