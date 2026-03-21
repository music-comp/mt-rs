use clap::Parser;
use mt_rs::cli::{Cli, run};

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(output) => print!("{}", output),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}
