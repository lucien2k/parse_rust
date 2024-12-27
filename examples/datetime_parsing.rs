use parse_rust::Parser;

fn main() {
    // Generic datetime format
    let p = Parser::new("Event time: {:tg}", true).unwrap();
    let result = p.parse("Event time: 27/12/2024 19:57:55").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    println!("Generic datetime: {}", dt);

    // American format
    let p = Parser::new("Meeting at {:ta}", true).unwrap();
    let result = p.parse("Meeting at 12/27/2024 07:57:55 PM").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    println!("American datetime: {}", dt);

    // Email format
    let p = Parser::new("Sent: {:te}", true).unwrap();
    let result = p.parse("Sent: Fri, 27 Dec 2024 19:57:55 +0000").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    println!("Email datetime: {}", dt);

    // ISO format
    let p = Parser::new("Timestamp: {:ti}", true).unwrap();
    let result = p.parse("Timestamp: 2024-12-27T19:57:55.000+00:00").unwrap();
    let dt: &chrono::NaiveDateTime = result.get(0).unwrap();
    println!("ISO datetime: {}", dt);
}
