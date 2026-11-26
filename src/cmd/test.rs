use std::path::{Path, PathBuf};

use crate::cmd::Ctx;
use crate::error::BojError;
use crate::judge::compare::{compare, Compare};
use crate::judge::runner::{self, Compiled, RunStatus};
use crate::judge::time::TimeRule;
use crate::meta::Meta;
use crate::template::{self, Vars};

/// 시간 제한을 모를 때(오프라인 생성 등) 쓰는 값.
pub const DEFAULT_LIMIT_MS: u64 = 5000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Wrong(Compare),
    TimeLimit,
    Runtime {
        code: Option<i32>,
        stderr: String,
    },
    /// 스페셜 저지 문제라 출력이 달라도 정답일 수 있다.
    Check(Compare),
}

#[derive(Debug, Clone)]
pub struct CaseResult {
    pub name: String,
    pub verdict: Verdict,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TestReport {
    pub dir: PathBuf,
    pub limit_ms: u64,
    pub limit_assumed: bool,
    pub compile_error: Option<String>,
    pub cases: Vec<CaseResult>,
}

impl TestReport {
    pub fn passed(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.verdict == Verdict::Ok)
            .count()
    }

    /// 전부 맞으면 0, 컴파일 에러나 하나라도 맞지 않으면 1.
    pub fn exit_code(&self) -> i32 {
        if self.compile_error.is_none()
            && !self.cases.is_empty()
            && self.passed() == self.cases.len()
        {
            0
        } else {
            1
        }
    }
}

/// `.in`과 `.out`이 짝인 케이스. 숫자 이름을 숫자 순으로 먼저, 나머지는 이름순.
pub fn collect_cases(tests_dir: &Path) -> Vec<(String, PathBuf, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(tests_dir) else {
        return Vec::new();
    };
    let mut cases: Vec<(String, PathBuf, PathBuf)> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "in"))
        .filter_map(|input| {
            let out = input.with_extension("out");
            let name = input.file_stem()?.to_string_lossy().into_owned();
            out.is_file().then_some((name, input, out))
        })
        .collect();
    cases.sort_by(|a, b| match (a.0.parse::<u64>(), b.0.parse::<u64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.0.cmp(&b.0),
    });
    cases
}

/// `boj test`: 컴파일하고 모든 케이스를 채점한다.
pub fn run_test(ctx: &Ctx, id: Option<u32>) -> Result<TestReport, BojError> {
    let dir = ctx.resolve_problem_dir(id)?;
    let meta = Meta::read(&dir)?.ok_or(BojError::NoProblemDir)?;
    let lang = ctx.config.lang(&meta.lang)?;
    let tests_dir = ctx.tests_dir(&dir, meta.id)?;
    let rule = TimeRule::parse(&lang.time)?;
    let limit_ms = rule.apply(meta.time_limit_ms.unwrap_or(DEFAULT_LIMIT_MS));
    let mut report = TestReport {
        dir: dir.clone(),
        limit_ms,
        limit_assumed: meta.time_limit_ms.is_none(),
        compile_error: None,
        cases: Vec::new(),
    };
    let cases = collect_cases(&tests_dir);
    if cases.is_empty() {
        return Ok(report);
    }

    let src_path = dir.join(&lang.file);
    let source = std::fs::read_to_string(&src_path)
        .map_err(|_| BojError::Usage(format!("소스 파일이 없어요: {}", src_path.display())))?;
    // 경로가 아니라 소스의 package 줄이 실제 클래스 이름을 정한다.
    let package = declared_package(&source);
    let build = tempfile::tempdir()?;
    let vars: Vars = [
        ("id", meta.id.to_string()),
        ("title", meta.title.clone()),
        ("dir", dir.to_string_lossy().into_owned()),
        ("file", lang.file.clone()),
        ("build", build.path().to_string_lossy().into_owned()),
        ("package", package.clone().unwrap_or_default()),
        ("main_class", template::main_class(package.as_deref())),
    ]
    .into_iter()
    .collect();

    if let Some(c) = &lang.compile {
        if let Compiled::Failed(err) = runner::compile(&template::render_command(c, &vars)?, &dir)?
        {
            report.compile_error = Some(err);
            return Ok(report);
        }
    }
    let run = template::render_command(&lang.run, &vars)?;
    for (name, input, output) in cases {
        let input = std::fs::read_to_string(&input)?;
        let expected = std::fs::read_to_string(&output)?;
        let o = runner::run_case(&run, &dir, &input, limit_ms)?;
        let verdict = match o.status {
            RunStatus::TimedOut => Verdict::TimeLimit,
            RunStatus::Exited(0) => match compare(&expected, &o.stdout) {
                Compare::Match => Verdict::Ok,
                diff if meta.special_judge => Verdict::Check(diff),
                diff => Verdict::Wrong(diff),
            },
            RunStatus::Exited(c) => Verdict::Runtime {
                code: Some(c),
                stderr: o.stderr,
            },
            RunStatus::Signaled => Verdict::Runtime {
                code: None,
                stderr: o.stderr,
            },
        };
        report.cases.push(CaseResult {
            name,
            verdict,
            elapsed_ms: o.elapsed_ms,
        });
    }
    Ok(report)
}

fn declared_package(source: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?m)^\s*package\s+([\w.]+)\s*;").expect("고정된 정규식");
    re.captures(source).map(|c| c[1].to_string())
}

/// 사람이 읽는 결과 출력.
pub fn render_report(r: &TestReport) -> String {
    let mut out = String::new();
    if let Some(err) = &r.compile_error {
        out.push_str("컴파일 에러\n");
        out.push_str(err.trim_end());
        out.push('\n');
        return out;
    }
    if r.cases.is_empty() {
        out.push_str(&format!(
            "채점할 케이스가 없어요. {}/tests 에 N.in / N.out을 넣어 주세요\n",
            r.dir.display()
        ));
        return out;
    }
    let width = r
        .cases
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(1);
    for c in &r.cases {
        let label = match &c.verdict {
            Verdict::Ok => "맞음",
            Verdict::Wrong(_) => "틀림",
            Verdict::TimeLimit => "시간 초과",
            Verdict::Runtime { .. } => "런타임 에러",
            Verdict::Check(_) => "확인 필요",
        };
        out.push_str(&format!(
            "{:<width$}  {label}  {}ms\n",
            c.name, c.elapsed_ms
        ));
        match &c.verdict {
            Verdict::Wrong(Compare::Mismatch {
                line,
                expected,
                actual,
            })
            | Verdict::Check(Compare::Mismatch {
                line,
                expected,
                actual,
            }) => {
                out.push_str(&format!(
                    "    줄 {line}: 기대 {expected:?} / 실제 {actual:?}\n"
                ));
                if matches!(c.verdict, Verdict::Check(_)) {
                    out.push_str(
                        "    스페셜 저지 문제라 답이 여러 개일 수 있어요. 직접 확인해 주세요\n",
                    );
                }
            }
            Verdict::Runtime { code, stderr } => {
                let code = code.map_or("시그널".to_string(), |c| c.to_string());
                out.push_str(&format!("    종료 코드 {code}\n"));
                for l in stderr.lines().take(5) {
                    out.push_str(&format!("    {l}\n"));
                }
            }
            _ => {}
        }
    }
    out.push_str(&format!(
        "{}/{} 맞음  (시간 제한 {}ms{})\n",
        r.passed(),
        r.cases.len(),
        r.limit_ms,
        if r.limit_assumed {
            ", 문제 정보가 없어 기본값 사용"
        } else {
            ""
        }
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::new::{run_new, NewArgs};
    use crate::fetch::FixtureFetcher;
    use crate::judge::runner::tests::java_available;
    use std::fs;

    fn fixtures() -> FixtureFetcher {
        FixtureFetcher {
            dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        }
    }

    /// 문제를 만들고 Main.java를 body로 바꾼 뒤 (tempdir, 문제 폴더)를 돌려준다.
    fn setup(id: u32, main_body: &str) -> (tempfile::TempDir, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let ctx = Ctx::new(t.path(), None).unwrap();
        let args = NewArgs {
            id,
            lang: None,
            variant: None,
            offline: false,
        };
        let r = run_new(&ctx, &args, &fixtures()).unwrap();
        fs::write(
            r.dir.join("Main.java"),
            format!("import java.util.*;\npublic class Main {{ public static void main(String[] a) {{ Scanner s = new Scanner(System.in); {main_body} }} }}\n"),
        )
        .unwrap();
        (t, r.dir)
    }

    const AB: &str = "int x = s.nextInt(), y = s.nextInt(); System.out.println(x + y);";

    #[test]
    fn collect_cases_orders_numeric_first() {
        let t = tempfile::tempdir().unwrap();
        for n in ["10", "2", "1", "my1"] {
            fs::write(t.path().join(format!("{n}.in")), "").unwrap();
            fs::write(t.path().join(format!("{n}.out")), "").unwrap();
        }
        fs::write(t.path().join("lonely.in"), "").unwrap();
        let names: Vec<String> = collect_cases(t.path()).into_iter().map(|c| c.0).collect();
        assert_eq!(names, ["1", "2", "10", "my1"]);
    }

    #[test]
    fn run_test_all_ok() {
        if !java_available() {
            return;
        }
        let (_t, dir) = setup(1000, AB);
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert_eq!(r.cases.len(), 1);
        assert_eq!(r.cases[0].verdict, Verdict::Ok);
        assert_eq!(r.limit_ms, 2000 * 2 + 1000);
        assert_eq!(r.exit_code(), 0);
        assert!(render_report(&r).contains("1/1 맞음"));
    }

    #[test]
    fn run_test_wrong() {
        if !java_available() {
            return;
        }
        let (_t, dir) = setup(
            1000,
            "int x = s.nextInt(), y = s.nextInt(); System.out.println(x - y);",
        );
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert!(matches!(
            &r.cases[0].verdict,
            Verdict::Wrong(Compare::Mismatch { line: 1, .. })
        ));
        assert_eq!(r.exit_code(), 1);
        let text = render_report(&r);
        assert!(text.contains("틀림") && text.contains("줄 1"));
    }

    #[test]
    fn compile_error_is_reported() {
        if !java_available() {
            return;
        }
        let (_t, dir) = setup(1000, "int x = ;");
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert!(r.compile_error.is_some());
        assert_eq!(r.exit_code(), 1);
        assert!(render_report(&r).contains("컴파일 에러"));
    }

    #[test]
    fn runtime_error_and_timeout() {
        if !java_available() {
            return;
        }
        let (_t, dir) = setup(1000, "throw new RuntimeException(\"boom\");");
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert!(
            matches!(&r.cases[0].verdict, Verdict::Runtime { stderr, .. } if stderr.contains("boom"))
        );

        let (_t2, dir2) = setup(1000, "while (true) {}");
        let meta_path = dir2.join(".boj.toml");
        let meta = fs::read_to_string(&meta_path)
            .unwrap()
            .replace("time_limit_ms = 2000", "time_limit_ms = 100");
        fs::write(&meta_path, meta).unwrap();
        let r2 = run_test(&Ctx::new(&dir2, None).unwrap(), None).unwrap();
        assert_eq!(r2.cases[0].verdict, Verdict::TimeLimit);
    }

    #[test]
    fn by_id_from_project_root() {
        if !java_available() {
            return;
        }
        let (t, _dir) = setup(1000, AB);
        let r = run_test(&Ctx::new(t.path(), None).unwrap(), Some(1000)).unwrap();
        assert_eq!(r.exit_code(), 0);
    }

    #[test]
    fn no_cases_is_not_success() {
        let t = tempfile::tempdir().unwrap();
        let ctx = Ctx::new(t.path(), None).unwrap();
        let args = NewArgs {
            id: 1000,
            lang: None,
            variant: None,
            offline: true,
        };
        let dir = run_new(&ctx, &args, &fixtures()).unwrap().dir;
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert!(r.cases.is_empty());
        assert_eq!(r.exit_code(), 1);
        assert!(render_report(&r).contains("케이스가 없어요"));
    }

    #[test]
    fn outside_problem_dir_errors() {
        let t = tempfile::tempdir().unwrap();
        let r = run_test(&Ctx::new(t.path(), None).unwrap(), None);
        assert!(matches!(r, Err(BojError::NoProblemDir)));
    }

    #[test]
    fn spj_mismatch_is_check() {
        if !java_available() {
            return;
        }
        let (_t, dir) = setup(2580, "System.out.println(\"no\");");
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert!(matches!(&r.cases[0].verdict, Verdict::Check(_)));
        assert_eq!(r.exit_code(), 1);
        assert!(render_report(&r).contains("확인 필요"));
    }

    #[test]
    fn packaged_source_runs() {
        if !java_available() {
            return;
        }
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join(".boj")).unwrap();
        fs::write(
            t.path().join(".boj/config.toml"),
            "path = \"src/main/java/p{{id}}\"\n",
        )
        .unwrap();
        let ctx = Ctx::new(t.path(), None).unwrap();
        let args = NewArgs {
            id: 1000,
            lang: None,
            variant: None,
            offline: false,
        };
        let dir = run_new(&ctx, &args, &fixtures()).unwrap().dir;
        fs::write(
            dir.join("Main.java"),
            format!("package p1000;\nimport java.util.*;\npublic class Main {{ public static void main(String[] a) {{ Scanner s = new Scanner(System.in); {AB} }} }}\n"),
        )
        .unwrap();
        let r = run_test(&Ctx::new(&dir, None).unwrap(), None).unwrap();
        assert_eq!(r.exit_code(), 0, "{}", render_report(&r));
    }
}
