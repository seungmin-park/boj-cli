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

fn java_available() -> bool {
    let ok = std::process::Command::new("javac")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok && std::env::var_os("BOJ_REQUIRE_JAVA").is_some() {
        panic!("BOJ_REQUIRE_JAVA가 설정됐는데 javac가 없어요");
    }
    ok
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
