mod basic;
mod basic_async;
mod http_server;

fn main() -> anyhow::Result<()> {
    let example = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .cloned()
        .unwrap_or("basic".to_string());

    match example.as_str() {
        "basic" => basic::main(),
        "basic_async" => basic_async::main(),
        "http_server" => http_server::main(),
        _ => Err(anyhow::anyhow!("No example for: \"{}\"", example)),
    }
}
