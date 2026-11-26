use assert_cmd::Command;
use predicates::str::contains;

fn boj() -> Command {
    let mut cmd = Command::cargo_bin("boj").unwrap();
    // 사용자 전역 설정이 테스트에 섞이지 않게 막는다.
    let empty = tempfile::tempdir().unwrap();
    cmd.env("BOJ_CONFIG_DIR", empty.keep());
    cmd
}

#[test]
fn help_lists_commands() {
    boj()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("test"))
        .stdout(contains("copy"))
        .stdout(contains("open"));
}

#[test]
fn no_args_is_tool_error() {
    boj().assert().code(2);
}

use std::fs;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn boj_in(dir: &Path) -> Command {
    let mut cmd = boj();
    cmd.current_dir(dir).env("BOJ_FIXTURE_DIR", fixtures());
    cmd
}

fn tool_available(lang: &str) -> bool {
    let probe: &[&str] = match lang {
        "java" => &["javac", "-version"],
        "cpp" => &["g++", "--version"],
        "py" => &["python3", "--version"],
        "kt" => &["kotlinc", "-version"],
        other => panic!("알 수 없는 언어: {other}"),
    };
    let ok = std::process::Command::new(probe[0])
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
    ok
}

fn java_available() -> bool {
    tool_available("java")
}

const AB_MAIN: &str = "import java.util.*;\npublic class Main { public static void main(String[] a) { Scanner s = new Scanner(System.in); System.out.println(s.nextInt() + s.nextInt()); } }\n";

#[test]
fn e2e_new_then_test() {
    if !java_available() {
        return;
    }
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path())
        .arg("1000")
        .assert()
        .success()
        .stdout(contains("1000"));
    let dir = t.path().join("1000");
    assert!(fs::read_to_string(dir.join("Main.java"))
        .unwrap()
        .contains("static BufferedReader br"));
    fs::write(dir.join("Main.java"), AB_MAIN).unwrap();
    boj_in(&dir)
        .arg("test")
        .assert()
        .code(0)
        .stdout(contains("1/1 맞음"));
    boj_in(t.path()).args(["test", "1000"]).assert().code(0);
}

#[test]
fn e2e_wrong_answer_exits_1() {
    if !java_available() {
        return;
    }
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path()).arg("1000").assert().success();
    let dir = t.path().join("1000");
    fs::write(
        dir.join("Main.java"),
        AB_MAIN.replace("s.nextInt() + s.nextInt()", "0"),
    )
    .unwrap();
    boj_in(&dir)
        .arg("test")
        .assert()
        .code(1)
        .stdout(contains("틀림"));
}

#[test]
fn e2e_second_run_no_overwrite() {
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path()).arg("1000").assert().success();
    let main = t.path().join("1000/Main.java");
    fs::write(&main, "MY CODE").unwrap();
    boj_in(t.path()).arg("1000").assert().success();
    assert_eq!(fs::read_to_string(&main).unwrap(), "MY CODE");
}

#[test]
fn e2e_korean_space_path() {
    if !java_available() {
        return;
    }
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("알고 리즘");
    fs::create_dir_all(&root).unwrap();
    boj_in(&root).arg("1000").assert().success();
    fs::write(root.join("1000/Main.java"), AB_MAIN).unwrap();
    boj_in(&root.join("1000"))
        .arg("test")
        .assert()
        .code(0)
        .stdout(contains("1/1 맞음"));
}

#[test]
fn e2e_copy_stdout_strips_package() {
    let t = tempfile::tempdir().unwrap();
    fs::create_dir_all(t.path().join(".boj")).unwrap();
    fs::write(
        t.path().join(".boj/config.toml"),
        "path = \"src/main/java/p{{id}}\"\n",
    )
    .unwrap();
    boj_in(t.path()).arg("1000").assert().success();
    let main = fs::read_to_string(t.path().join("src/main/java/p1000/Main.java")).unwrap();
    assert!(main.starts_with("package p1000;"));
    boj_in(t.path())
        .args(["copy", "1000", "--stdout"])
        .assert()
        .success()
        .stdout(predicates::str::starts_with("import java.io.*;"));
}

#[test]
fn e2e_open_print() {
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path())
        .args(["open", "2309", "--print"])
        .assert()
        .success()
        .stdout("https://www.acmicpc.net/problem/2309\n");
}

#[test]
fn e2e_unknown_lang_and_problem() {
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path())
        .args(["1000", "-l", "brainf"])
        .assert()
        .code(2)
        .stderr(contains("brainf"));
    boj_in(t.path())
        .arg("424242")
        .assert()
        .code(2)
        .stderr(contains("424242"));
    assert!(!t.path().join("424242").exists());
}

#[test]
fn e2e_test_outside_problem_dir() {
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path())
        .arg("test")
        .assert()
        .code(2)
        .stderr(contains("문제 폴더를 찾을 수 없어요"));
}

const AB_PY: &str = "a, b = map(int, input().split())\nprint(a + b)\n";

#[test]
fn e2e_python_new_then_test() {
    if !tool_available("py") {
        return;
    }
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path())
        .args(["1000", "-l", "py"])
        .assert()
        .success()
        .stdout(contains("main.py"));
    let dir = t.path().join("1000");
    assert!(fs::read_to_string(dir.join("main.py"))
        .unwrap()
        .contains("# 1000 A+B"));
    fs::write(dir.join("main.py"), AB_PY).unwrap();
    boj_in(&dir)
        .arg("test")
        .assert()
        .code(0)
        .stdout(contains("1/1 맞음"));
}

#[test]
fn e2e_two_languages_need_flag() {
    if !(java_available() && tool_available("py")) {
        return;
    }
    let t = tempfile::tempdir().unwrap();
    boj_in(t.path()).arg("1000").assert().success();
    boj_in(t.path())
        .args(["1000", "-l", "py"])
        .assert()
        .success();
    let dir = t.path().join("1000");
    fs::write(dir.join("Main.java"), AB_MAIN).unwrap();
    fs::write(dir.join("main.py"), AB_PY).unwrap();
    boj_in(&dir)
        .arg("test")
        .assert()
        .code(2)
        .stderr(contains("-l"))
        .stderr(contains("Main.java"))
        .stderr(contains("main.py"));
    boj_in(&dir)
        .args(["test", "-l", "java"])
        .assert()
        .code(0)
        .stdout(contains("1/1 맞음"));
    boj_in(&dir)
        .args(["test", "-l", "py"])
        .assert()
        .code(0)
        .stdout(contains("1/1 맞음"));
    boj_in(&dir)
        .args(["copy", "-l", "py", "--stdout"])
        .assert()
        .success()
        .stdout(AB_PY);
}
