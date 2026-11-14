use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about = "Minimal Rust CLI starter")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Greet { name: String },
}

fn message(name: &str) -> String {
    format!("Hello, {}!", name.trim())
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::Greet { name } => println!("{}", message(&name)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting() {
        assert_eq!(message(" Rust "), "Hello, Rust!");
    }

    #[test]
    fn valid_cli() {
        assert!(Cli::try_parse_from(["app", "greet", "Jhon"]).is_ok());
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(Cli::try_parse_from(["app", "oops"]).is_err());
    }
}
