fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = argv[1..].iter().map(String::as_str).collect();
    std::process::exit(ansiblek8s_rs::cli::run(&args));
}
