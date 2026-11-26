//! 구조 규칙을 기계적으로 확인한다. 규칙과 이유는 docs/engineering/architecture.md.
//!
//! - 네트워크(reqwest)는 src/fetch/mod.rs 에서만 쓴다.
//! - 외부 프로세스 실행(std::process::Command)은 src/judge/runner.rs 에서만 쓴다.
//!
//! 이 경계 덕분에 파서, 설정, 템플릿, 비교 로직은 네트워크나 컴파일러 없이 테스트된다.

use std::fs;
use std::path::{Path, PathBuf};

struct Rule {
    what: &'static str,
    patterns: &'static [&'static str],
    allowed: &'static [&'static str],
}

const RULES: &[Rule] = &[
    Rule {
        what: "네트워크",
        patterns: &["reqwest"],
        allowed: &["src/fetch/mod.rs"],
    },
    Rule {
        what: "외부 프로세스 실행",
        patterns: &["process::Command", "Command::new(", "process::Stdio"],
        allowed: &["src/judge/runner.rs"],
    },
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn violations(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    assert!(
        files.len() > 10,
        "src 아래 .rs 파일을 찾지 못했어요: {}",
        files.len()
    );
    let mut found = Vec::new();
    for f in files {
        let rel = f
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&f).unwrap();
        for rule in RULES {
            if rule.allowed.contains(&rel.as_str()) {
                continue;
            }
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                if let Some(p) = rule.patterns.iter().find(|p| line.contains(**p)) {
                    found.push(format!(
                        "{rel}:{}: {}은(는) {}에서만 써요 (`{p}` 발견)",
                        n + 1,
                        rule.what,
                        rule.allowed.join(", ")
                    ));
                }
            }
        }
    }
    found
}

#[test]
fn effects_stay_behind_their_boundaries() {
    let v = violations(Path::new(env!("CARGO_MANIFEST_DIR")));
    assert!(v.is_empty(), "구조 규칙 위반:\n{}", v.join("\n"));
}
