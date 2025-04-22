use crate::ast::Expr;
use crate::interpreter::Interpreter;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

mod ast;
mod parser;
mod code;
mod interpreter;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the REPL
    Repl { file: Option<PathBuf> },
    /// Run the parser on a file
    Parse { file: PathBuf },
}


fn main() {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Repl { file: None });

    match command {
        Commands::Parse { file } => {
            println!("Parsing file: {:?}", file);
        }
        Commands::Repl { file } => {
            let mut interpreter = Interpreter::new();
            if let Some(file) = file {
                println!("Running REPL with file: {:?}", file);
            } else {
                println!("Running REPL without file");
            }
            repl(&mut interpreter);
        }
    }
}

fn repl(interpreter: &mut Interpreter) {
    let mut input = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }
        if input.trim() == "exit" {
            break;
        }
        let expr = Expr::from_str(&input).expect("Failed to parse input");
        match interpreter.eval(&expr) {
            Ok(result) => println!("{:?}", result),
            Err(e) => eprintln!("Error: {:?}", e),
        }
        input.clear();
    }
}