mod cli;
mod error;
mod fetch;
mod problem;

use clap::Parser;
use cli::{Cli, Command};
use error::BojError;

fn run(cli: Cli) -> Result<i32, BojError> {
    match (cli.command, cli.id) {
        (Some(Command::Test { .. }), _)
        | (Some(Command::Copy { .. }), _)
        | (Some(Command::Open { .. }), _)
        | (None, Some(_)) => Err(BojError::Config("아직 구현되지 않았어요".into())),
        (None, None) => Err(BojError::Usage(
            "문제 번호나 명령이 필요해요. `boj --help`를 참고하세요".into(),
        )),
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            std::process::exit(code);
        }
    };
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(e.exit_code());
        }
    }
}
