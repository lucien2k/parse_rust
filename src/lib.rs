use std::collections::HashMap;
use regex::{Regex, RegexBuilder};
use thiserror::Error;

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
    pub converted: HashMap<String, Box<dyn std::any::Any>>,
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

lazy_static::lazy_static! {
    static ref DEFAULT_TYPES: HashMap<String, Box<dyn TypeConverter>> = {
        let mut m = HashMap::new();
        m.insert("d".to_string(), Box::new(IntConverter) as Box<dyn TypeConverter>);
        m.insert("f".to_string(), Box::new(FloatConverter) as Box<dyn TypeConverter>);
        m.insert("w".to_string(), Box::new(WordConverter) as Box<dyn TypeConverter>);
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
                _ => None,
            } {
                default_types.insert(k.clone(), converter);
            }
        }
        Self::new_with_types(format, case_sensitive, default_types)
    }
    
    pub fn parse(&self, text: &str) -> Option<ParseResult> {
        self.pattern.captures(text).map(|caps| self.process_captures(&caps))
    }
    
    pub fn search(&self, text: &str) -> Option<ParseResult> {
        self.search_pattern.captures(text).map(|caps| self.process_captures(&caps))
    }
    
    fn process_captures(&self, caps: &regex::Captures) -> ParseResult {
        let mut fixed = Vec::new();
        let mut named = HashMap::new();
        let mut spans = Vec::new();
        let mut converted = HashMap::new();
        
        // Initialize fixed with empty strings to preserve order
        fixed.resize(self.field_map.len(), String::new());
        
        // Process captures in order of appearance
        for (field_name, &group_idx) in &self.field_map {
            if let Some(m) = caps.get(group_idx) {
                let value = m.as_str().to_string();
                fixed[group_idx - 1] = value.clone();  // -1 because group 0 is the whole match
                named.insert(field_name.clone(), value.clone());
                spans.push((m.start(), m.end()));
                
                // Handle type conversion
                if let Some(type_name) = self.field_types.get(field_name) {
                    if let Some(converter) = self.type_converters.get(type_name) {
                        if let Ok(converted_value) = converter.convert(&value) {
                            converted.insert(field_name.clone(), converted_value);
                        }
                    }
                }
            }
        }
        
        ParseResult {
            fixed,
            named,
            spans,
            converted,
        }
    }
    
    pub fn findall(&self, text: &str) -> Vec<ParseResult> {
        self.search_pattern
            .captures_iter(text)
            .map(|caps| self.process_captures(&caps))
            .collect()
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
