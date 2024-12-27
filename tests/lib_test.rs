//! Test cases for the parse_rust library
use chrono::NaiveDateTime;
use parse_rust::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let p = Parser::new("{:w} {:w}", true).unwrap();
        let result = p.parse("hello world").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "hello");
        assert_eq!(*result.get::<String>(1).unwrap(), "world");
    }

    #[test]
    fn test_pattern_with_text() {
        let p = Parser::new("Hello, {:w}!", true).unwrap();
        let result = p.parse("Hello, world!").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "world");
    }

    #[test]
    fn test_multiple_matches() {
        let p = Parser::new("{:w} {:w} {:w}", true).unwrap();
        let result = p.parse("a b c").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "a");
        assert_eq!(*result.get::<String>(1).unwrap(), "b");
        assert_eq!(*result.get::<String>(2).unwrap(), "c");
    }

    #[test]
    fn test_named_fields_basic() {
        let p = Parser::new("{first:w} {second:w}", true).unwrap();
        let result = p.parse("hello world").unwrap();
        assert_eq!(*result.named::<String>("first").unwrap(), "hello");
        assert_eq!(*result.named::<String>("second").unwrap(), "world");
    }

    #[test]
    fn test_mixed_named_unnamed() {
        let p = Parser::new("{:w} {name:w} {:w}", true).unwrap();
        let result = p.parse("a world c").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "a");
        assert_eq!(*result.named::<String>("name").unwrap(), "world");
        assert_eq!(*result.get::<String>(2).unwrap(), "c");
    }

    #[test]
    fn test_integer() {
        let p = Parser::new("{:d}", true).unwrap();
        let result = p.parse("25").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 25);
    }

    #[test]
    fn test_findall() {
        let p = Parser::new("{:w}", true).unwrap();
        let results = p.findall("a b c");
        assert_eq!(results.len(), 3);
        assert_eq!(*results[0].get::<String>(0).unwrap(), "a");
        assert_eq!(*results[1].get::<String>(0).unwrap(), "b");
        assert_eq!(*results[2].get::<String>(0).unwrap(), "c");
    }

    #[test]
    fn test_datetime() {
        use chrono::NaiveDateTime;

        let p = Parser::new("{:tg}", true).unwrap();
        let result = p.parse("27/12/2024 20:45:27").unwrap();
        let dt = result.get::<NaiveDateTime>(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 20:45:27"
        );
    }

    #[test]
    fn test_empty_pattern() {
        // Empty pattern should match empty string
        let p = Parser::new("", true).unwrap();
        assert!(p.parse("").is_some());

        // Pattern with just a literal should match that literal
        let p = Parser::new("hello", true).unwrap();
        assert!(p.parse("hello").is_some());
        assert!(p.parse("world").is_none());
    }

    #[test]
    fn test_named_fields() {
        let p = Parser::new("Name: {name:w}, Age: {age:d}, Score: {score:f}", true).unwrap();
        let result = p.parse("Name: Alice, Age: 25, Score: 95.5").unwrap();

        // Test basic named field access
        assert_eq!(*result.named::<String>("name").unwrap(), "Alice");
        assert_eq!(*result.named::<i64>("age").unwrap(), 25);
        assert_eq!(*result.named::<f64>("score").unwrap(), 95.5);

        // Test non-existent field
        assert!(result.named::<String>("nonexistent").is_none());

        // Test wrong type access
        assert!(result.named::<i64>("name").is_none());
        assert!(result.named::<String>("age").is_none());
        assert!(result.named::<i64>("score").is_none());
    }

    #[test]
    fn test_complex_named_fields() {
        let p = Parser::new(
            "User {user.name:w} ({user.id:d}) - Role: {user.role:w}",
            true,
        )
        .unwrap();
        let result = p.parse("User admin (123) - Role: superuser").unwrap();

        // Test dot notation fields
        assert_eq!(*result.named::<String>("user.name").unwrap(), "admin");
        assert_eq!(*result.named::<i64>("user.id").unwrap(), 123);
        assert_eq!(*result.named::<String>("user.role").unwrap(), "superuser");

        // Test accessing parent field (should be None)
        assert!(result.named::<String>("user").is_none());
    }

    #[test]
    fn test_mixed_named_and_positional() {
        let p = Parser::new("{:d} - {name:w} - {value:f}", true).unwrap();
        let result = p.parse("42 - Alice - 3.14").unwrap();

        // Test positional access
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);
        assert_eq!(*result.get::<f64>(2).unwrap(), 3.14);

        // Test named access
        assert_eq!(*result.named::<String>("name").unwrap(), "Alice");
    }

    #[test]
    fn test_repeated_named_fields() {
        let p = Parser::new("{x:d} + {y:d} = {sum:d}", true).unwrap();
        let result = p.parse("5 + 7 = 12").unwrap();

        assert_eq!(*result.named::<i64>("x").unwrap(), 5);
        assert_eq!(*result.named::<i64>("y").unwrap(), 7);
        assert_eq!(*result.named::<i64>("sum").unwrap(), 12);
    }

    #[test]
    fn test_empty_named_fields() {
        let p = Parser::new("Name: {name:w}, Age: {age:d}", true).unwrap();

        // Test with missing fields
        assert!(p.parse("Name: , Age: 25").is_none());
        assert!(p.parse("Name: Alice, Age: ").is_none());
    }

    #[test]
    fn test_case_sensitivity() {
        // Test case-sensitive parsing
        let p = Parser::new("Hello, {name:w}!", true).unwrap();
        assert!(p.parse("Hello, World!").is_some());
        assert!(p.parse("HELLO, World!").is_none());

        // Test case-insensitive parsing
        let p = Parser::new("Hello, {name:w}!", false).unwrap();
        assert!(p.parse("Hello, World!").is_some());
        assert!(p.parse("HELLO, World!").is_some());
        assert!(p.parse("hello, World!").is_some());
    }

    #[test]
    fn test_special_characters() {
        // Test regex special characters in pattern
        let p = Parser::new("Price: ${price:f}", true).unwrap();
        let result = p.parse("Price: $123.45").unwrap();
        assert_eq!(*result.named::<f64>("price").unwrap(), 123.45);

        // Test with parentheses
        let p = Parser::new("({value:w})", true).unwrap();
        let result = p.parse("(test)").unwrap();
        assert_eq!(*result.named::<String>("value").unwrap(), "test");

        // Test with square brackets
        let p = Parser::new("[{value:w}]", true).unwrap();
        let result = p.parse("[test]").unwrap();
        assert_eq!(*result.named::<String>("value").unwrap(), "test");
    }

    #[test]
    fn test_optional_whitespace() {
        let p = Parser::new("{a:w},{b:w}", true).unwrap();

        // Test with and without spaces
        let result = p.parse("hello,world").unwrap();
        assert_eq!(*result.named::<String>("a").unwrap(), "hello");
        assert_eq!(*result.named::<String>("b").unwrap(), "world");

        let result = p.parse("hello, world").unwrap();
        assert_eq!(*result.named::<String>("a").unwrap(), "hello");
        assert_eq!(*result.named::<String>("b").unwrap(), "world");
    }

    #[test]
    fn test_basic_examples() {
        // Basic string parsing
        let result = parse("It's {}, I love it!", "It's spam, I love it!").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "spam");

        // Search in a larger string
        let result = search("Age: {:d}\n", "Name: Rufus\nAge: 42\nColor: red\n").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);

        // Find all occurrences
        let results = findall(">{}<", "<p>the <b>bold</b> text</p>");
        let texts: Vec<String> = results
            .iter()
            .map(|r| r.get::<String>(0).unwrap().clone())
            .collect();
        assert_eq!(texts.join(""), "the bold text");
    }

    #[test]
    fn test_format_syntax() {
        // Anonymous fields
        let result = parse("Bring me a {}", "Bring me a shrubbery").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "shrubbery");

        // Multiple anonymous fields
        let result = parse("The {} who {} {}", "The knights who say Ni!").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "knights");
        assert_eq!(*result.get::<String>(1).unwrap(), "say");
        assert_eq!(*result.get::<String>(2).unwrap(), "Ni!");

        // Named fields
        let result = parse(
            "Meet {name} at {time:tg}",
            "Meet Alan at 27/12/2024 20:45:27",
        )
        .unwrap();
        assert_eq!(*result.named::<String>("name").unwrap(), "Alan");
        let dt = result.named::<NaiveDateTime>("time").unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 20:45:27"
        );

        // Mixed named and anonymous fields
        let result = parse("The {} is {color}", "The sky is blue").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "sky");
        assert_eq!(*result.named::<String>("color").unwrap(), "blue");
    }

    #[test]
    fn test_type_conversions() {
        // Integer
        let result = parse("{:d}", "42").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);

        // Float
        let result = parse("{:f}", "3.14").unwrap();
        assert_eq!(*result.get::<f64>(0).unwrap(), 3.14);

        // Word
        let result = parse("{:w}", "hello").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "hello");

        // DateTime
        let result = parse("{:tg}", "27/12/2024 20:45:27").unwrap();
        let dt = result.get::<NaiveDateTime>(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 20:45:27"
        );
    }

    #[test]
    fn test_search_examples() {
        // Search at start of string
        let result = search("Age: {:d}", "Age: 42").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);

        // Search in middle of string
        let result = search("age={:d}", "name=John age=42 color=blue").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);

        // Search with named fields
        let result = search("name={:d}", "color=blue name=42 age=42").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);
    }

    #[test]
    fn test_findall_examples() {
        // Find all numbers
        let results = findall("{:d}", "1 2 3 4 5");
        let numbers: Vec<i64> = results.iter().map(|r| *r.get::<i64>(0).unwrap()).collect();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5]);

        // Find all key-value pairs
        let results = findall("{key:w}={value:w}", "name=John age=42 color=blue");
        let pairs: Vec<(String, String)> = results
            .iter()
            .map(|r| {
                (
                    r.named::<String>("key").unwrap().clone(),
                    r.named::<String>("value").unwrap().clone(),
                )
            })
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("name".to_string(), "John".to_string()),
                ("age".to_string(), "42".to_string()),
                ("color".to_string(), "blue".to_string())
            ]
        );
    }

    #[test]
    fn test_examples_basic_parsing() {
        // Basic integer parsing
        let p = Parser::new("Value is {:d}", true).unwrap();
        let result = p.parse("Value is 42").unwrap();
        let value: &i64 = result.get(0).unwrap();
        assert_eq!(*value, 42);

        // Basic word parsing
        let p = Parser::new("Hello, {:w}!", true).unwrap();
        let result = p.parse("Hello, World!").unwrap();
        let word: &String = result.get(0).unwrap();
        assert_eq!(word, "World");

        // Multiple fields
        let p = Parser::new("{:w} is {:d} years old", true).unwrap();
        let result = p.parse("Alice is 25 years old").unwrap();
        let name: &String = result.get(0).unwrap();
        let age: &i64 = result.get(1).unwrap();
        assert_eq!(name, "Alice");
        assert_eq!(*age, 25);
    }

    #[test]
    fn test_examples_datetime_parsing() {
        // Generic datetime format
        let p = Parser::new("Event time: {:tg}", true).unwrap();
        let result = p.parse("Event time: 27/12/2024 19:57:55").unwrap();
        let dt: &NaiveDateTime = result.get(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 19:57:55"
        );

        // American format
        let p = Parser::new("Meeting at {:ta}", true).unwrap();
        let result = p.parse("Meeting at 12/27/2024 07:57:55 PM").unwrap();
        let dt: &NaiveDateTime = result.get(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 19:57:55"
        );

        // Email format
        let p = Parser::new("Sent: {:te}", true).unwrap();
        let result = p.parse("Sent: Fri, 27 Dec 2024 19:57:55 +0000").unwrap();
        let dt: &NaiveDateTime = result.get(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 19:57:55"
        );

        // ISO format
        let p = Parser::new("Timestamp: {:ti}", true).unwrap();
        let result = p.parse("Timestamp: 2024-12-27T19:57:55.000+00:00").unwrap();
        let dt: &NaiveDateTime = result.get(0).unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-12-27 19:57:55"
        );
    }

    #[test]
    fn test_examples_named_fields() {
        // Named fields with different types
        let p = Parser::new("Name: {name:w}, Age: {age:d}, Score: {score:f}", true).unwrap();
        let result = p.parse("Name: Alice, Age: 25, Score: 95.5").unwrap();

        let name: &String = result.named("name").unwrap();
        let age: &i64 = result.named("age").unwrap();
        let score: &f64 = result.named("score").unwrap();

        assert_eq!(name, "Alice");
        assert_eq!(*age, 25);
        assert_eq!(*score, 95.5);

        // Complex field names with dot notation
        let p = Parser::new("User {user.name:w} has role {user.role:w}", true).unwrap();
        let result = p.parse("User admin has role superuser").unwrap();

        let username: &String = result.named("user.name").unwrap();
        let role: &String = result.named("user.role").unwrap();

        assert_eq!(username, "admin");
        assert_eq!(role, "superuser");
    }

    #[test]
    fn test_examples_search_and_findall() {
        // Search for a pattern in text
        let p = Parser::new("age: {:d}", true).unwrap();
        let text = "User profile - name: John, age: 30, city: New York";

        let result = p.search(text).unwrap();
        let age: &i64 = result.get(0).unwrap();
        assert_eq!(*age, 30);

        // Find all numbers in text
        let p = Parser::new("{:d}", true).unwrap();
        let text = "Scores: 85, 92, 78, 95, 88";

        let results = p.findall(text);
        let scores: Vec<i64> = results.iter().map(|r| *r.get::<i64>(0).unwrap()).collect();
        assert_eq!(scores, vec![85, 92, 78, 95, 88]);

        // Find all dates in text
        let p = Parser::new("{:tg}", true).unwrap();
        let text = "Events: 27/12/2024 19:57:55, 28/12/2024 10:30:00, 29/12/2024 15:45:00";

        let results = p.findall(text);
        let dates: Vec<String> = results
            .iter()
            .map(|r| {
                r.get::<NaiveDateTime>(0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .collect();
        assert_eq!(
            dates,
            vec![
                "2024-12-27 19:57:55",
                "2024-12-28 10:30:00",
                "2024-12-29 15:45:00"
            ]
        );
    }

    #[test]
    fn test_multiline_search() {
        let result = search("age: {:d}\n", "name: Rufus\nage: 42\ncolor: red\n").unwrap();
        assert_eq!(*result.get::<i64>(0).unwrap(), 42);
    }

    #[test]
    fn test_dotted_field_names() {
        // Test dotted field names in type conversion
        let result = parse("{a.b:d}", "1").unwrap();
        assert_eq!(*result.named::<i64>("a.b").unwrap(), 1);

        // Test mixed dotted and underscored names
        let result = parse("{a_b:w} {a.b:d}", "1 2").unwrap();
        assert_eq!(*result.named::<String>("a_b").unwrap(), "1");
        assert_eq!(*result.named::<i64>("a.b").unwrap(), 2);
    }

    #[test]
    fn test_pm_handling() {
        // Test PM times around noon
        let result = parse("Meet at {:tg}", "Meet at 1/2/2011 12:15 PM").unwrap();
        let dt = result.get::<NaiveDateTime>(0).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2011-02-01 12:15");

        // Test AM times around midnight
        let result = parse("Meet at {:tg}", "Meet at 1/2/2011 12:15 AM").unwrap();
        let dt = result.get::<NaiveDateTime>(0).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2011-02-01 00:15");
    }

    #[test]
    fn test_case_sensitive_findall() {
        // Test case-insensitive (default)
        let results = findall("x({:w})x", "X(hi)X");
        let words: Vec<String> = results
            .iter()
            .map(|r| r.get::<String>(0).unwrap().clone())
            .collect();
        assert_eq!(words, vec!["hi"]);

        // Test case-sensitive
        let p = Parser::new("x({:w})x", true).unwrap();
        let results = p.findall("X(hi)X");
        assert!(results.is_empty());
    }

    #[test]
    fn test_unmatched_brace() {
        // Test that unmatched braces don't parse
        assert!(Parser::new("a{b", true).is_err());
        assert!(Parser::new("a}b", true).is_err());
        assert!(Parser::new("a{b}}", true).is_err());
        assert!(Parser::new("a{{b}", true).is_err());
    }

    #[test]
    fn test_trailing_newline() {
        // Test that patterns can match strings with trailing newlines
        let result = parse("Hello {:w}!", "Hello World!\n").unwrap();
        assert_eq!(*result.get::<String>(0).unwrap(), "World");
    }
}
