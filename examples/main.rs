mod basic;

fn main() {
  let example = std::env::args()
    .collect::<Vec<String>>()
    .get(1)
    .cloned()
    .unwrap_or("basic".to_string());

  match example.as_str() {
    "basic" => basic::main(),
    _ => eprintln!("No example for: \"{}\"", example),
  }
}
