use parse_rust::Parser;

fn main() {
    // Named fields with different types
    let p = Parser::new("Name: {name:w}, Age: {age:d}, Score: {score:f}", true).unwrap();
    let result = p.parse("Name: Alice, Age: 25, Score: 95.5").unwrap();

    let name: &String = result.named("name").unwrap();
    let age: &i64 = result.named("age").unwrap();
    let score: &f64 = result.named("score").unwrap();

    println!("Student Info:");
    println!("  Name: {}", name);
    println!("  Age: {}", age);
    println!("  Score: {}", score);

    // Complex field names with dot notation
    let p = Parser::new("User {user.name:w} has role {user.role:w}", true).unwrap();
    let result = p.parse("User admin has role superuser").unwrap();

    let username: &String = result.named("user__name").unwrap();
    let role: &String = result.named("user__role").unwrap();

    println!("\nUser Info:");
    println!("  Username: {}", username);
    println!("  Role: {}", role);
}
