use parse_rust::Parser;

fn main() {
    // Search for a pattern in text
    let p = Parser::new("age: {:d}", true).unwrap();
    let text = "User profile - name: John, age: 30, city: New York";

    if let Some(result) = p.search(text) {
        let age: &i64 = result.get(0).unwrap();
        println!("Found age: {}", age);
    }

    // Find all numbers in text
    let p = Parser::new("{:d}", true).unwrap();
    let text = "Scores: 85, 92, 78, 95, 88";

    let results = p.findall(text);
    println!("\nAll scores:");
    for result in results {
        let score: &i64 = result.get(0).unwrap();
        println!("  {}", score);
    }

    // Find all dates in text
    let p = Parser::new("{:tg}", true).unwrap();
    let text = "Events: 27/12/2024 19:57:55, 28/12/2024 10:30:00, 29/12/2024 15:45:00";

    let results = p.findall(text);
    println!("\nAll events:");
    for result in results {
        let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
        println!("  {}", dt);
    }
}
