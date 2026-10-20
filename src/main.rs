use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("galerne {}", galerne::VERSION);
        return;
    }

    println!("Galerne {}", galerne::VERSION);
}
