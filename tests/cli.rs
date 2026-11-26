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
