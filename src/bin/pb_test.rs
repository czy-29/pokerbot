use std::io::Write;

fn main() {
    print!("\x1b[107m\x1b[0J\x1b[30m");

    print!("input: ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    println!("output: {}", input.trim());

    print!("\x1b[0m\x1b[0J");
}
