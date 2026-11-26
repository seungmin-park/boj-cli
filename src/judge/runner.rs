use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

use crate::error::BojError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compiled {
    Ok,
    /// 컴파일러가 실패했다. 사용자 코드의 컴파일 에러다.
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStatus {
    Exited(i32),
    TimedOut,
    Signaled,
}

#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub status: RunStatus,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u64,
}

/// 치환이 끝난 명령 토큰으로 컴파일한다. 컴파일러를 못 찾으면 도구 오류다.
pub fn compile(cmd: &[String], dir: &Path) -> Result<Compiled, BojError> {
    let out = command(cmd, dir)?
        .stdin(Stdio::null())
        .output()
        .map_err(|e| spawn_error(cmd, e))?;
    if out.status.success() {
        Ok(Compiled::Ok)
    } else {
        let mut msg = String::from_utf8_lossy(&out.stderr).into_owned();
        msg.push_str(&String::from_utf8_lossy(&out.stdout));
        Ok(Compiled::Failed(msg))
    }
}

/// 입력을 넣고 실행한다. 시간 제한을 넘기면 종료시킨다.
///
/// stdin 쓰기와 stdout/stderr 읽기를 각각 스레드로 돌려, 파이프가 가득 차서
/// 서로 기다리며 멈추는 일을 막는다.
pub fn run_case(
    cmd: &[String],
    dir: &Path,
    input: &str,
    limit_ms: u64,
) -> Result<RunOutcome, BojError> {
    let mut child = command(cmd, dir)?
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| spawn_error(cmd, e))?;
    let started = Instant::now();

    let mut stdin = child.stdin.take().expect("piped stdin");
    let input = input.as_bytes().to_vec();
    // 프로그램이 입력을 다 읽지 않고 끝나면 BrokenPipe가 나는데, 정상이라 무시한다.
    let writer = thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let mut out_pipe = child.stdout.take().expect("piped stdout");
    let mut err_pipe = child.stderr.take().expect("piped stderr");
    let out_reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out_pipe.read_to_end(&mut buf);
        buf
    });
    let err_reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err_pipe.read_to_end(&mut buf);
        buf
    });

    let waited = child.wait_timeout(Duration::from_millis(limit_ms))?;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let status = match waited {
        Some(st) => match st.code() {
            Some(c) => RunStatus::Exited(c),
            None => RunStatus::Signaled,
        },
        None => {
            let _ = child.kill();
            let _ = child.wait();
            RunStatus::TimedOut
        }
    };
    let _ = writer.join();
    let stdout = out_reader.join().unwrap_or_default();
    let stderr = err_reader.join().unwrap_or_default();
    Ok(RunOutcome {
        status,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        elapsed_ms,
    })
}

fn command(cmd: &[String], dir: &Path) -> Result<Command, BojError> {
    let (prog, args) = cmd
        .split_first()
        .ok_or_else(|| BojError::Config("명령이 비어 있어요".into()))?;
    let mut c = Command::new(prog);
    c.args(args).current_dir(dir);
    Ok(c)
}

fn spawn_error(cmd: &[String], e: std::io::Error) -> BojError {
    let prog = cmd.first().map(String::as_str).unwrap_or("");
    if e.kind() == std::io::ErrorKind::NotFound {
        BojError::Usage(format!(
            "`{prog}`을 찾을 수 없어요. 설치돼 있고 PATH에 있는지 확인해 주세요"
        ))
    } else {
        BojError::Io(e)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::fs;

    /// 언어 도구가 없으면 테스트를 건너뛴다. CI는 `BOJ_REQUIRE_LANGS=java,cpp,...`로
    /// 적은 언어의 건너뛰기를 막는다 (도구가 없으면 실패).
    pub(crate) fn tool_available(lang: &str) -> bool {
        let probe: &[&str] = match lang {
            "java" => &["javac", "-version"],
            "cpp" => &["g++", "--version"],
            "py" => &["python3", "--version"],
            "kt" => &["kotlinc", "-version"],
            other => panic!("알 수 없는 언어: {other}"),
        };
        let ok = Command::new(probe[0])
            .args(&probe[1..])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let required = std::env::var("BOJ_REQUIRE_LANGS")
            .unwrap_or_default()
            .split(',')
            .any(|l| l.trim() == lang);
        if !ok && required {
            panic!(
                "BOJ_REQUIRE_LANGS에 {lang}이 있는데 `{}`이 없어요",
                probe[0]
            );
        }
        if !ok {
            eprintln!("{} 이 없어 {lang} 채점 테스트를 건너뜀", probe[0]);
        }
        ok
    }

    pub(crate) fn java_available() -> bool {
        tool_available("java")
    }

    fn java_program(body: &str) -> (tempfile::TempDir, Vec<String>) {
        let t = tempfile::tempdir().unwrap();
        fs::write(
            t.path().join("Main.java"),
            format!("import java.io.*;\npublic class Main {{ public static void main(String[] a) throws Exception {{ {body} }} }}\n"),
        )
        .unwrap();
        let build = t.path().join("build").to_string_lossy().into_owned();
        let compiled = compile(
            &[
                "javac".into(),
                "-d".into(),
                build.clone(),
                "Main.java".into(),
            ],
            t.path(),
        )
        .unwrap();
        assert_eq!(compiled, Compiled::Ok);
        (t, vec!["java".into(), "-cp".into(), build, "Main".into()])
    }

    #[test]
    fn echo_program_exits_zero() {
        if !java_available() {
            return;
        }
        let (t, run) = java_program(
            "BufferedReader br = new BufferedReader(new InputStreamReader(System.in)); System.out.println(br.readLine());",
        );
        let o = run_case(&run, t.path(), "hello\n", 5000).unwrap();
        assert_eq!(o.status, RunStatus::Exited(0));
        // 실행기는 원래 출력을 그대로 준다 (Windows는 \r\n). 줄바꿈 무시는 비교 단계의 몫이다.
        assert_eq!(o.stdout.replace("\r\n", "\n"), "hello\n");
    }

    #[test]
    fn infinite_loop_times_out() {
        if !java_available() {
            return;
        }
        let (t, run) = java_program("while (true) {}");
        let o = run_case(&run, t.path(), "", 500).unwrap();
        assert_eq!(o.status, RunStatus::TimedOut);
        assert!(o.elapsed_ms < 3000, "elapsed {}", o.elapsed_ms);
    }

    #[test]
    fn exception_is_nonzero_exit() {
        if !java_available() {
            return;
        }
        let (t, run) = java_program("throw new RuntimeException(\"boom\");");
        let o = run_case(&run, t.path(), "", 5000).unwrap();
        assert!(matches!(o.status, RunStatus::Exited(c) if c != 0));
        assert!(o.stderr.contains("boom"));
    }

    #[test]
    fn compile_error_returns_stderr() {
        if !java_available() {
            return;
        }
        let t = tempfile::tempdir().unwrap();
        fs::write(
            t.path().join("Main.java"),
            "public class Main { int x = ; }",
        )
        .unwrap();
        let r = compile(&["javac".into(), "Main.java".into()], t.path()).unwrap();
        assert!(matches!(r, Compiled::Failed(e) if e.contains("error")));
    }

    #[test]
    fn missing_compiler_is_tool_error() {
        let t = tempfile::tempdir().unwrap();
        let r = compile(&["definitely-not-a-compiler-xyz".into()], t.path());
        assert!(
            matches!(r, Err(BojError::Usage(m)) if m.contains("definitely-not-a-compiler-xyz"))
        );
    }

    #[test]
    fn huge_output_no_deadlock() {
        if !java_available() {
            return;
        }
        let (t, run) = java_program(
            "StringBuilder sb = new StringBuilder(); for (int i = 0; i < 1_000_000; i++) sb.append(\"123456789\\n\"); System.out.print(sb);",
        );
        let o = run_case(&run, t.path(), "", 10000).unwrap();
        assert_eq!(o.status, RunStatus::Exited(0));
        assert_eq!(o.stdout.len(), 10_000_000);
    }

    #[test]
    fn program_ignoring_stdin_finishes() {
        if !java_available() {
            return;
        }
        let (t, run) = java_program("System.out.println(1);");
        let big = "9 ".repeat(1 << 19);
        let o = run_case(&run, t.path(), &big, 5000).unwrap();
        assert_eq!(o.status, RunStatus::Exited(0));
        assert_eq!(o.stdout.replace("\r\n", "\n"), "1\n");
    }
}
