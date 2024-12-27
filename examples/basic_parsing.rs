use parse_rust::Parser;

fn main() {
    // Basic integer parsing
    let p = Parser::new("Value is {:d}", true).unwrap();
    let result = p.parse("Value is 42").unwrap();
    let value: &i64 = result.get(0).unwrap();
    println!("Parsed integer: {}", value); // Output: Parsed integer: 42

    // Basic word parsing
    let p = Parser::new("Hello, {:w}!", true).unwrap();
    let result = p.parse("Hello, World!").unwrap();
    let word: &String = result.get(0).unwrap();
    println!("Parsed word: {}", word); // Output: Parsed word: World

    // Multiple fields
    let p = Parser::new("{:w} is {:d} years old", true).unwrap();
    let result = p.parse("Alice is 25 years old").unwrap();
    let name: &String = result.get(0).unwrap();
    let age: &i64 = result.get(1).unwrap();
    println!("Name: {}, Age: {}", name, age); // Output: Name: Alice, Age: 25
}
