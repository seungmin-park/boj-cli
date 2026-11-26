mod cli;
mod cmd;
mod config;
mod error;
mod fetch;
mod judge;
mod meta;
mod problem;
mod template;

use std::path::Path;

use clap::Parser;
use cli::{Cli, Command};
use cmd::copy::Copied;
use cmd::new::{NewArgs, NewReport};
use cmd::open::Opened;
use cmd::Ctx;
use error::BojError;

fn run(cli: Cli) -> Result<i32, BojError> {
    let cwd = std::env::current_dir()?;
    match (cli.command, cli.id) {
        (None, None) => Err(BojError::Usage(
            "문제 번호나 명령이 필요해요. `boj --help`를 참고하세요".into(),
        )),
        (None, Some(id)) => {
            let ctx = Ctx::new(&cwd, config::global_dir())?;
            let args = NewArgs {
                id,
                lang: cli.lang.as_deref(),
                variant: cli.template.as_deref(),
                offline: cli.offline,
            };
            let report = cmd::new::run_new(&ctx, &args, fetch::fetcher_from_env().as_ref())?;
            print_new(&report, &cwd);
            Ok(0)
        }
        (Some(Command::Test { id, lang }), _) => {
            let ctx = Ctx::new(&cwd, config::global_dir())?;
            let report = cmd::test::run_test(&ctx, id, lang.as_deref())?;
            print!("{}", cmd::test::render_report(&report));
            Ok(report.exit_code())
        }
        (Some(Command::Copy { id, lang, stdout }), _) => {
            let ctx = Ctx::new(&cwd, config::global_dir())?;
            match cmd::copy::run_copy(&ctx, id, lang.as_deref(), stdout)? {
                Copied::Clipboard => println!("제출용 코드를 클립보드에 복사했어요"),
                Copied::Stdout(code) => print!("{code}"),
            }
            Ok(0)
        }
        (Some(Command::Open { id, print }), _) => {
            let ctx = Ctx::new(&cwd, config::global_dir())?;
            match cmd::open::run_open(&ctx, id, print)? {
                Opened::Browser(url) => println!("열었어요: {url}"),
                Opened::Printed(url) => println!("{url}"),
            }
            Ok(0)
        }
    }
}

fn print_new(r: &NewReport, cwd: &Path) {
    let show = |p: &Path| p.strip_prefix(cwd).unwrap_or(p).display().to_string();
    if r.created.is_empty() {
        println!("이미 준비돼 있어요: {}", show(&r.dir));
    } else {
        println!("만들었어요");
        for p in &r.created {
            println!("  {}", show(p));
        }
        if !r.skipped.is_empty() {
            println!("이미 있어서 그대로 둔 파일: {}개", r.skipped.len());
        }
        println!("폴더: {}", show(&r.dir));
    }
    for w in &r.warnings {
        eprintln!("경고: {w}");
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
