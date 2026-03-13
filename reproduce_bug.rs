use dbc::evaluate;

fn main() {
    match evaluate("max(1,2)") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
