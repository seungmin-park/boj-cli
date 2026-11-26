# boj CLI 구현 계획

> **에이전트 작업자용:** 필수 하위 스킬은 superpowers:subagent-driven-development(권장) 또는 superpowers:executing-plans다. 이 계획을 태스크 단위로 구현할 때 쓴다. 진행 체크는 [tasks.md](tasks.md)에서 한다.

**목표:** 문제 번호 하나로 풀이 파일과 백준 예제를 만들고 바로 채점하는 공개용 Rust CLI `boj`를 만든다. v1은 Java만 지원한다.

**구조:**
- 순수 로직은 외부 통신과 프로세스 실행에서 떼어 낸다.
  - 순수 로직: HTML 파싱, 설정 병합, 템플릿, 출력 비교, 시간 공식
  - 외부 통신: `Fetcher` trait
  - 프로세스 실행: `judge::runner`
- 명령마다 `cmd/` 모듈 하나를 둔다. 각 모듈은 이 부품들을 조립만 한다.
- 언어별 동작은 코드에 넣지 않는다. 내장 `builtin.toml`과 사용자 설정만으로 정한다.

**기술 스택:**

| 분류 | 사용 |
|---|---|
| 기본 | Rust stable, edition 2021 |
| CLI | clap 4 (derive) |
| HTTP·파싱 | reqwest(blocking, rustls), scraper |
| 설정 | serde, toml, dirs |
| 명령 실행 | shell-words, wait-timeout |
| 기타 | regex, arboard, open, thiserror |
| 개발용 | tempfile, assert_cmd, predicates |

**설계 문서:** [docs/superpowers/specs/2026-11-27-boj-cli-design.md](docs/superpowers/specs/2026-11-27-boj-cli-design.md)

## 전역 제약

- crate 이름과 바이너리 이름은 `boj`, GitHub 레포는 `seungmin-park/boj-cli`다. crates.io의 `boj`는 2026-11-27 기준 비어 있다.
- 종료 코드
  - `0`: 성공
  - `1`: 채점 실패(틀림/시간 초과/런타임 에러/컴파일 에러/확인 필요)
  - `2`: 도구 오류(설정, 네트워크, 문제 없음, 폴더 못 찾음)
- 사용자가 만든 파일은 절대 덮어쓰지 않는다.
- 문제 페이지 요청은 문제당 1회로 한다.
  - 결과는 `.boj.toml`과 `tests/`에 저장한다. 이후에는 요청하지 않는다.
  - User-Agent는 `boj-cli/{CARGO_PKG_VERSION} (+https://github.com/seungmin-park/boj-cli)`다.
- 문제 URL은 `https://www.acmicpc.net/problem/{id}`다. 리다이렉트를 따라간다.
- 기본 경로
  - 문제 폴더: `path = "{{id}}"`
  - 예제 폴더: `tests = "{{dir}}/tests"`
  - 메타 파일: `{{dir}}/.boj.toml`
- 자리 표시자는 아래 7개뿐이다. 모르는 자리 표시자는 오류다.
  - `{{id}}`, `{{title}}`, `{{dir}}`, `{{file}}`, `{{build}}`, `{{package}}`, `{{main_class}}`
- Java 내장값

  | 항목 | 값 |
  |---|---|
  | 기본 `--release` | 11. 사용자가 설정으로 8 등으로 덮어쓴다 |
  | `file` | `Main.java` |
  | `compile` | `javac -encoding UTF-8 --release 11 -d {{build}} {{file}}` |
  | `run` | `java -Xss256m -cp {{build}} {{main_class}}` |
  | `strip` | `['^\s*package\s+[\w.]+;\s*$']` |
  | `time` | `x2 + 1s` |

- 시간 제한을 모르면 5000ms를 쓴다. `--offline`으로 만든 경우가 그렇다.
- 출력 비교 규칙
  - 무시하는 것: 각 줄 끝 공백, 파일 끝 빈 줄, `\r`
  - 무시하지 않는 것: 줄 앞 공백
- 모든 테스트는 네트워크 없이 통과해야 한다. 실제 요청 테스트는 `#[ignore]`로 둔다.
- 사용자에게 보이는 메시지는 한국어로 쓴다.

## 리뷰 초점

| 입력이나 상황 | 기대 동작 | 테스트 위치 |
|---|---|---|
| 사용자 프로그램이 Windows 줄바꿈(`\r\n`)으로 출력함 | 맞음으로 판정한다 | Task 5 |
| 사용자 프로그램이 출력을 아주 많이 하거나 입력을 읽지 않음 | 멈추지 않고 끝까지 판정한다 | Task 9 |
| 문제 폴더 밖에서 번호 없이 `boj test`를 실행함 | "문제 폴더를 찾을 수 없어요"와 종료 코드 2 | Task 10 |
| 프로젝트 경로에 한글이나 공백이 있음 (예: `~/알고 리즘`) | 생성과 채점이 정상 동작한다 | Task 12 |
| 예제가 없는 문제, 또는 예제 출력이 빈 문제 | 예제가 없으면 경고만 하고 소스는 만든다. 빈 출력은 빈 `.out` 파일로 저장한다 | Task 2, Task 8 |

---

## 파일 구조

```
Cargo.toml
src/
  main.rs            # 진입점, 명령 분기, 오류를 종료 코드로 변환
  cli.rs             # clap 정의
  error.rs           # BojError, exit_code()
  problem.rs         # Problem, Sample
  fetch/mod.rs       # Fetcher trait, HttpFetcher, FixtureFetcher, fetcher_from_env()
  fetch/parse.rs     # parse_problem()
  config/mod.rs      # Config, LangConfig, 병합, 프로젝트 루트 탐색
  config/builtin.toml
  template.rs        # render(), java_package(), 템플릿 탐색
  templates/java.java
  meta.rs            # Meta(.boj.toml), 문제 폴더 탐색
  judge/time.rs      # TimeRule
  judge/compare.rs   # compare()
  judge/runner.rs    # compile(), run_case()
  cmd/new.rs  cmd/test.rs  cmd/copy.rs  cmd/open.rs
tests/
  fixtures/1000.html 1260.html 2580.html 15552.html notfound.html nosample.html
  cli.rs             # end-to-end
.github/workflows/ci.yml
```

## Phase 0. 기반

### Task 1: 프로젝트 뼈대와 CLI 정의

**파일:**
- 만들기: `Cargo.toml`, `src/main.rs`, `src/cli.rs`, `src/error.rs`, `.gitignore`, `.github/workflows/ci.yml`
- 테스트: `tests/cli.rs`

**인터페이스:**
- `cli::Cli`
  - 최상위 필드
    - `id: Option<u32>`
    - `-l/--lang: Option<String>`
    - `-t/--template: Option<String>`
    - `--offline: bool`
  - 하위 명령
    - `Test { id: Option<u32> }`
    - `Copy { id: Option<u32>, --stdout: bool }`
    - `Open { id: Option<u32>, --print: bool }`
  - `args_conflicts_with_subcommands = true`를 쓴다. `boj 2309`와 `boj test 2309`가 둘 다 동작해야 한다.
- `error::BojError`
  - thiserror enum
  - 변형: `Config(String)`, `Network(String)`, `ProblemNotFound(u32)`, `Parse(String)`, `NoProblemDir`, `Io(std::io::Error)`, `Template(String)`
  - `fn exit_code(&self) -> i32`는 항상 2를 돌려준다.
- `main`: 결과가 `Err`면 메시지를 stderr에 쓰고 `exit_code()`로 종료한다.

- [ ] **1단계: 실패하는 테스트를 쓴다** (`tests/cli.rs`)
  - `help_lists_commands`: `boj --help`가 성공하고 출력에 `test`, `copy`, `open`이 있다.
  - `no_args_is_tool_error`: `boj`만 실행하면 종료 코드가 2다.
- [ ] **2단계:** `cargo test --test cli` → 실패를 확인한다.
- [ ] **3단계:** 위 인터페이스를 구현한다.
  - 각 명령의 본문은 `todo!()` 대신 `Err(BojError::Config("미구현".into()))`로 둔다.
  - CI는 ubuntu, macos, windows에서 `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`를 돌린다.
- [ ] **4단계:** `cargo test` → 통과를 확인한다.
- [ ] **5단계:** 커밋한다. `chore: scaffold boj cli`

## Phase 1. 순수 로직

### Task 2: 문제 모델과 HTML 파서

**파일:**
- 만들기: `src/problem.rs`, `src/fetch/parse.rs`, `tests/fixtures/*.html`

**인터페이스:**
- `Problem`
  - `id: u32`
  - `title: String`
  - `time_limit_ms: Option<u64>`
  - `special_judge: bool`
  - `samples: Vec<Sample>`
- `Sample { input: String, output: String }`
- `fn parse_problem(id: u32, html: &str) -> Result<Problem, BojError>`

**파싱 규칙:**
- 제목은 `#problem_title`에서 읽는다.
- 예제는 `#sample-input-N`, `#sample-output-N`에서 읽는다. N=1부터 짝이 끊길 때까지 읽는다.
- 시간 제한은 `#problem-info tbody td` 첫 칸에서 읽는다. 앞의 숫자(소수 포함)에 초 단위로 1000을 곱한다.
- 스페셜 저지는 `.problem-label-spj`가 있으면 true다.
- 예제 텍스트 처리
  - `\r\n`은 `\n`으로 바꾼다.
  - 비어 있지 않으면 끝에 `\n`이 하나 있도록 맞춘다.
- 제목이 없으면 `ProblemNotFound(id)`다.

**Fixture 준비:**
- 다음 페이지를 `curl -sL -A "boj-cli-dev" https://www.acmicpc.net/problem/{id}`로 받아 저장한다.
  - 1000: 기본
  - 1260: 예제 3개
  - 2580: 스페셜 저지
  - 15552: 시간 제한이 "1 초 (하단 참고)" 형식
  - 없는 번호 하나 → `notfound.html`
- `nosample.html`은 1000.html에서 예제 블록을 지우고, 예제 출력 하나는 비워서 만든다.

- [ ] **1단계: 실패하는 테스트를 쓴다** (`parse.rs` 안 `#[cfg(test)]`)
  - `parses_1260`: 제목 `"DFS와 BFS"`, 예제 3개, `time_limit_ms == Some(2000)`, `special_judge == false`, `samples[0].input`이 `"4 5 1\n"`으로 시작한다.
  - `parses_spj_2580`: `special_judge == true`, `time_limit_ms == Some(1000)`
  - `parses_time_with_note_15552`: `Some(1000)`
  - `not_found_page`: `Err(BojError::ProblemNotFound(_))`
  - `no_samples_and_empty_output`: 예제가 없으면 빈 Vec이다. 빈 출력 예제는 `output == ""`다.
- [ ] **2단계:** 실패를 확인한다.
- [ ] **3단계:** 구현한다.
- [ ] **4단계:** `cargo test parse` → 통과를 확인한다.
- [ ] **5단계:** 커밋한다. `feat: parse BOJ problem page`

### Task 3: Fetcher

**파일:**
- 만들기: `src/fetch/mod.rs`

**인터페이스:**
- `trait Fetcher { fn fetch(&self, id: u32) -> Result<Problem, BojError>; }`
- `HttpFetcher`
  - reqwest blocking을 쓴다.
  - 타임아웃은 10초, User-Agent는 전역 제약대로다.
  - 404는 `ProblemNotFound`, 그 밖의 실패는 `Network(이유)`다.
- `FixtureFetcher { dir: PathBuf }`: `{dir}/{id}.html`을 읽어 파싱한다.
- `fn fetcher_from_env() -> Box<dyn Fetcher>`
  - 환경 변수 `BOJ_FIXTURE_DIR`가 있으면 FixtureFetcher, 없으면 HttpFetcher를 돌려준다.
  - 테스트 전용이다. README에는 쓰지 않는다.

- [ ] **1단계: 테스트를 쓴다**
  - `fixture_fetcher_reads_file`: id 1260이 제목을 돌려준다.
  - `fixture_fetcher_missing_file`: 없는 id면 `ProblemNotFound`다.
  - `#[ignore] live_fetch_1000`: 제목 `"A+B"`
- [ ] **2~4단계:** 실패 확인 → 구현 → `cargo test fetch` 통과
- [ ] **5단계:** 커밋한다. `feat: add fetcher`

### Task 4: 시간 공식

**파일:**
- 만들기: `src/judge/time.rs`

**인터페이스:**
- `TimeRule { mul: f64, add_ms: u64 }`
- `fn parse(s: &str) -> Result<TimeRule, BojError>`
  - 형식: `x<실수>`, `+ <정수>s`, `+ <정수>ms`를 조합한다. 공백은 자유다.
  - 빈 문자열이면 `x1`이다.
- `fn apply(&self, base_ms: u64) -> u64`

- [ ] **1단계: 테스트를 쓴다**
  - `"x2 + 1s"` 적용(1000) → 3000
  - `"x3 + 2s"` 적용(2000) → 8000
  - `"x1.5"` 적용(1000) → 1500
  - `"+ 500ms"` 적용(1000) → 1500
  - `""` 적용(700) → 700
  - `"fast"` → `Err(Config)`
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: time limit rule`

### Task 5: 출력 비교

**파일:**
- 만들기: `src/judge/compare.rs`

**인터페이스:**
- `enum Compare { Match, Mismatch { line: usize, expected: String, actual: String } }`
  - `line`은 1부터 센다.
  - 한쪽 줄이 모자라면 그쪽 값은 `"(없음)"`이다.
- `fn compare(expected: &str, actual: &str) -> Compare`

- [ ] **1단계: 표 기반 테스트를 쓴다**
  - `"1 2\n"` vs `"1 2   \n\n\n"` → Match
  - `"1\n2\n"` vs `"1\r\n2\r\n"` → Match (리뷰 초점 1)
  - `""` vs `"\n"` → Match
  - `" 1\n"` vs `"1\n"` → Mismatch line 1
  - `"1\n2\n"` vs `"1\n"` → Mismatch line 2, actual `"(없음)"`
  - `"a\nb\n"` vs `"a\nc\n"` → Mismatch line 2
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: output comparison`

### Task 6: 설정 병합

**파일:**
- 만들기: `src/config/mod.rs`, `src/config/builtin.toml`

**인터페이스:**
- `Config`
  - `default: String`
  - `path: String`
  - `tests: String`
  - `langs: BTreeMap<String, LangConfig>`
- `LangConfig`
  - `file: String`
  - `template: String`
  - `compile: Option<String>`
  - `run: String`
  - `strip: Vec<String>`
  - `time: String`
- 원본 TOML은 모든 필드를 `Option`으로 받는 `RawConfig`/`RawLang`으로 읽는다. 병합한 뒤 필수 필드가 비었으면 `Config` 오류다.
- `fn find_project_root(start: &Path) -> Option<PathBuf>`: 위로 올라가며 `.boj/` 디렉터리를 찾는다.
- `fn global_dir() -> Option<PathBuf>`: `BOJ_CONFIG_DIR` 환경 변수가 우선이다. 없으면 `dirs::config_dir()/boj`다.
- `fn load(project_root: Option<&Path>, global_dir: Option<&Path>) -> Result<Config, BojError>`
  - 병합 순서는 내장 → 전역 → 프로젝트다.
  - 언어 표는 필드 단위로 덮어쓴다.
- `fn lang(&self, name: &str) -> Result<&LangConfig, BojError>`: 없는 언어면 `Config("지원하지 않는 언어: {name}")`다.

- [ ] **1단계: 테스트를 쓴다** (tempdir로 설정 파일을 만든다)
  - `builtin_only_has_java`: java의 compile에 `--release 11`이 있다.
  - `global_overrides_one_field`: 전역에 `[lang.java] compile = "javac --release 8 ..."`를 넣으면 compile만 바뀌고 run은 내장값이다.
  - `project_beats_global`: 프로젝트 `path`가 전역 `path`를 이긴다.
  - `user_adds_language`: `[lang.rust]` 블록을 추가하면 `lang("rust")`가 Ok다.
  - `missing_required_field_errors`: 새 언어에 `run`이 없으면 `Err(Config)`다.
  - `find_root_walks_up`: `a/.boj/` 아래 `a/b/c`에서 시작하면 `a`를 찾는다.
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: layered config`

### Task 7: 템플릿

**파일:**
- 만들기: `src/template.rs`, `src/templates/java.java`

**인터페이스:**
- `type Vars = BTreeMap<&'static str, String>`
- `fn render(src: &str, vars: &Vars) -> Result<String, BojError>`
  - `{{name}}`을 치환한다. 모르는 이름은 `Template("알 수 없는 자리 표시자: {{name}}")`이다.
  - 치환한 뒤 `^\s*package\s*;\s*$`에 맞는 줄은 지운다. 패키지가 비었을 때를 처리하기 위해서다.
- `fn java_package(rel_dir: &Path) -> Option<String>`
  - 경로 안에서 `src/main/java/`가 나오면 그 뒤 경로 구성 요소를 `.`으로 잇는다. 없으면 None이다.
- `fn main_class(package: Option<&str>) -> String`: 패키지가 있으면 `pkg.Main`, 없으면 `Main`이다.
- `fn package_warning(pkg: &str) -> Option<String>`: 구성 요소가 숫자로 시작하면 경고 문자열을 돌려준다.
- `fn load_template(name: &str, variant: Option<&str>, project_root: Option<&Path>, global_dir: Option<&Path>) -> Result<String, BojError>`
  - 찾는 순서
    1. `{project}/.boj/templates/`
    2. `{global}/templates/`
    3. 내장 템플릿
  - 파일 이름은 `{name}` 또는 `{name}-{variant}`에 아무 확장자나 붙은 것이다.
  - 내장 템플릿은 `include_str!("templates/java.java")`로 넣는다. 사용자 요청 템플릿(static `br`, `sb`, `st` 필드를 쓰는 BufferedReader 뼈대)에 `package {{package}};` 줄과 `// {{id}} {{title}}` 주석을 더한다.

- [ ] **1단계: 테스트를 쓴다**
  - `renders_vars`
  - `unknown_placeholder_errors`
  - `empty_package_line_removed`
  - `java_package("src/main/java/p1000") == Some("p1000")`
  - `java_package("1000") == None`
  - `main_class(Some("p1000")) == "p1000.Main"`
  - `package_warning("1000")`는 Some이다.
  - `project_template_beats_builtin`
  - `variant_lookup_scanner`
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: templates`

## Phase 2. 명령

### Task 8: 메타와 `boj <번호>` 생성

**파일:**
- 만들기: `src/meta.rs`, `src/cmd/new.rs`

**인터페이스:**
- `Meta`
  - `id: u32`
  - `title: String`
  - `time_limit_ms: Option<u64>`
  - `special_judge: bool`
  - `samples: usize`
  - `lang: String`
  - serde로 `.boj.toml`에 읽고 쓴다.
- `fn problem_dir(cfg: &Config, root: &Path, id: u32) -> PathBuf`: `path`를 렌더링해 root 기준 경로로 만든다.
- `fn find_problem_dir(start: &Path) -> Option<PathBuf>`: 위로 올라가며 `.boj.toml`을 찾는다.
- `fn run_new(id: u32, lang: Option<&str>, variant: Option<&str>, offline: bool, fetcher: &dyn Fetcher, cwd: &Path) -> Result<NewReport, BojError>`
  - `NewReport`: `dir: PathBuf`, `created: Vec<PathBuf>`, `skipped: Vec<PathBuf>`, `warnings: Vec<String>`
- 루트는 `find_project_root(cwd)`로 찾는다. 없으면 `cwd`를 쓴다.

**동작 규칙:**
- 파일이 이미 있으면 skipped에 넣고 건드리지 않는다.
- fetch는 다음 중 하나일 때만 1회 한다. 그 밖에는 하지 않는다.
  - `.boj.toml`이 없을 때
  - 메타의 예제 수(`Meta`에 `samples: usize`로 저장)만큼의 `tests/N.in`/`N.out` 중 빠진 게 있을 때. 이때는 빠진 파일만 채운다.
- `--offline`이면 fetch하지 않는다. 메타는 `title = ""`, `time_limit_ms = None`, `samples = 0`으로 만든다.
- 예제가 0개면 warnings에 "예제가 없어요"를 넣는다.
- 출력은 만든 파일 목록과 폴더 경로다.

- [ ] **1단계: 테스트를 쓴다** (FixtureFetcher와 tempdir 사용)
  - `creates_source_tests_meta`: 1260 → `1260/Main.java`, `1260/tests/1.in`~`3.out`, `1260/.boj.toml`이 생긴다. `Main.java`에 `// 1260 DFS와 BFS`가 있고 `package` 줄은 없다.
  - `never_overwrites`: `Main.java`에 "MY CODE"를 쓴 뒤 다시 실행해도 내용이 그대로이고, skipped에 들어 있다.
  - `refills_missing_test_only`: `tests/2.in`을 지우고 다시 실행하면 그 파일만 created에 들어 있다.
  - `java_package_under_src_main_java`: 프로젝트 설정 `path = "src/main/java/p{{id}}"`이면 `package p1260;`이 들어간다.
  - `offline_creates_source_only`: tests가 없고 `time_limit_ms`는 None이다.
  - `no_samples_warns`: nosample 픽스처면 warnings가 1개이고, 빈 출력 예제는 빈 `.out` 파일이다.
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: boj <id> scaffolds problem`

### Task 9: 컴파일과 실행기

**파일:**
- 만들기: `src/judge/runner.rs`

**인터페이스:**
- `fn compile(cmd: &str, dir: &Path) -> Result<(), CompileError>`
  - `CompileError { stderr: String }`
  - `cmd`는 렌더링이 끝난 문자열이다. `shell-words`로 나눠서 `dir`에서 실행한다.
- `fn run_case(cmd: &str, dir: &Path, input: &str, limit_ms: u64) -> RunOutcome`
  - `RunOutcome`: `status: RunStatus { Exited(i32), TimedOut, Signaled }`, `stdout: String`, `stderr: String`, `elapsed_ms: u64`
  - stdin 쓰기, stdout 읽기, stderr 읽기는 각각 별도 스레드에서 한다. 파이프가 차서 멈추는 걸 막기 위해서다.
  - `wait-timeout`으로 기다린다. 시간을 넘기면 kill하고 `TimedOut`이다.
  - 빌드 폴더는 호출하는 쪽이 `tempfile::TempDir`로 만든다.

- [ ] **1단계: 테스트를 쓴다**
  - 작은 Java 파일을 tempdir에 만들어 쓴다.
  - `javac`가 없으면 `eprintln!` 후 return해서 건너뛴다.
  - `echo_program_exits_zero`
  - `infinite_loop_times_out`: limit 500ms → TimedOut, elapsed < 3000
  - `exception_is_nonzero_exit`
  - `compile_error_returns_stderr`: stderr에 `error`가 있다.
  - `huge_output_no_deadlock`: 10MB를 출력하는 프로그램이 Exited(0)으로 끝난다. (리뷰 초점 2)
  - `ignores_stdin`: 입력을 안 읽는 프로그램도 큰 입력 1MB와 함께 정상 종료한다.
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: compile and run with timeout`

### Task 10: `boj test`

**파일:**
- 만들기: `src/cmd/test.rs`

**인터페이스:**
- `fn run_test(id: Option<u32>, cwd: &Path) -> Result<TestReport, BojError>`
  - `TestReport { cases: Vec<CaseResult>, compile_error: Option<String> }`
  - `fn exit_code(&self) -> i32`: 전부 맞음이면 0, 아니면 1이다.
- `CaseResult`: `name: String`, `verdict: Verdict { Ok, Wrong(Compare), TimeLimit, Runtime(i32, String), Check(Compare) }`, `elapsed_ms: u64`
  - `Check`는 스페셜 저지 문제의 불일치다. 종료 코드 1로 친다.
- `fn collect_cases(tests_dir: &Path) -> Vec<(String, PathBuf, PathBuf)>`
  - `.in`과 `.out`이 짝인 것만 고른다.
  - 정렬 순서: 숫자 이름을 숫자 순으로 먼저(1, 2, 10), 그다음 나머지를 사전순으로 둔다.

**동작 규칙:**
- 문제 폴더 찾기
  - id가 있으면 `problem_dir`로 찾는다.
  - 없으면 `find_problem_dir(cwd)`로 찾는다. 실패하면 `NoProblemDir`(종료 코드 2)이다.
- 시간 제한은 `TimeRule::parse(lang.time)?.apply(meta.time_limit_ms.unwrap_or(5000))`다.
- 출력 형식은 케이스마다 한 줄이다. 예: `1  맞음  83ms`
  - 틀림이면 아래에 `줄 N: 기대 "..." / 실제 "..."`를 출력한다.
  - 마지막 줄은 `3/3 맞음` 형식이다.

- [ ] **1단계: 테스트를 쓴다**
  - `collect_cases_orders_numeric_first`: 1, 2, 10, my1 순서다.
  - `run_test_all_ok`: FixtureFetcher로 1000을 생성하고 A+B 정답 Main.java를 쓰면 exit 0이다.
  - `run_test_wrong`: `a-b`를 출력하는 코드면 Wrong이고 exit 1이다.
  - `outside_problem_dir_errors`: 빈 tempdir에서 실행하면 `Err(NoProblemDir)`이다. (리뷰 초점 3)
  - `spj_mismatch_is_check`: 2580 메타와 다른 출력이면 Check다.
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: boj test`

### Task 11: `boj copy`, `boj open`

**파일:**
- 만들기: `src/cmd/copy.rs`, `src/cmd/open.rs`

**인터페이스:**
- `fn strip_lines(src: &str, patterns: &[String]) -> Result<String, BojError>`
  - 패턴에 맞는 줄을 지운다.
  - 지운 뒤 맨 앞에 남은 빈 줄도 지운다.
- `run_copy(id, stdout_only, cwd)`
  - arboard로 복사한다.
  - 실패하거나 `--stdout`이면 표준 출력으로 내보낸다.
- `fn problem_url(id: u32) -> String`
- `run_open(id, print_only, cwd)`
  - `open::that(url)`으로 연다.
  - `--print`면 URL만 출력한다.
  - id가 없으면 문제 폴더의 메타에서 읽는다.

- [ ] **1단계: 테스트를 쓴다**
  - `strips_package_line`: `"package p1;\n\nimport java.io.*;"` → `"import java.io.*;"`로 시작한다.
  - `keeps_other_lines`
  - `bad_regex_errors`
  - `problem_url(2309) == "https://www.acmicpc.net/problem/2309"`
- [ ] **2~4단계:** 실패 확인 → 구현 → 통과
- [ ] **5단계:** 커밋한다. `feat: boj copy and open`

### Task 12: 전체 CLI end-to-end

**파일:**
- 수정: `tests/cli.rs`, `src/main.rs`(명령 연결)

- [ ] **1단계: 테스트를 쓴다**
  - assert_cmd를 쓴다.
  - `BOJ_FIXTURE_DIR=tests/fixtures`로 실행한다.
  - 설정이 새지 않게 `BOJ_CONFIG_DIR`는 빈 tempdir로 둔다.
  - `e2e_new_then_test`
    1. `boj 1000` → exit 0, stdout에 `1000`
    2. 정답 Main.java를 쓴다.
    3. `boj test`(cwd=`1000/`) → exit 0, stdout에 `1/1 맞음`이 있다. 1000번은 예제가 1개다.
  - `e2e_second_run_no_overwrite`: 다시 실행해도 exit 0이고 소스가 그대로다.
  - `e2e_korean_space_path`: `알고 리즘` 폴더 안에서 생성과 채점이 성공한다. (리뷰 초점 4)
  - `e2e_copy_stdout`
    1. 새 tempdir에 `.boj/config.toml`을 만들고 `path = "src/main/java/p{{id}}"`로 둔다.
    2. `boj 1000`을 실행한다.
    3. `boj copy 1000 --stdout`의 출력이 `import`로 시작한다. 패키지 줄이 빠졌는지 보는 테스트다.
  - `e2e_open_print`: `boj open 1000 --print`
  - `e2e_unknown_lang`: `boj 1000 -l brainf` → exit 2
- [ ] **2~4단계:** 실패 확인 → `main.rs`에 연결 → `cargo test` 전체 통과
- [ ] **5단계:** 커밋한다. `test: end-to-end cli`

## Phase 3. 공개 준비

### Task 13: 문서와 배포

**파일:**
- 만들기: `README.md`, `LICENSE`(종류는 사용자가 정한다), `dist` 설정, `.github/workflows/release.yml`

- [ ] README를 쓴다. 설치 방법, 명령 4개, 설정과 템플릿 예시, 지원 언어, 한계(스페셜 저지와 인터랙티브)를 넣는다.
- [ ] `cargo dist init`으로 macOS, Linux, Windows 바이너리와 Homebrew tap(`seungmin-park/homebrew-tap`)을 설정한다.
- [ ] `v0.1.0` 태그를 단다. 릴리스 바이너리로 `brew install`이 되는지 확인한다.
- [ ] 레포 공개 전환과 crates.io 게시는 사용자가 확인한 뒤에 한다.

## Phase 4. 다언어 (v2)

### Task 14: C++, Python, Kotlin 내장값

- [ ] `builtin.toml`과 `templates/`에 cpp, py, kt를 추가한다.

  | 언어 | 컴파일 | 실행 | 시간 공식 |
  |---|---|---|---|
  | cpp | `g++ -O2 -std=gnu++17 -o {{build}}/a.out {{file}}` | `{{build}}/a.out` | 없음 |
  | py | 없음 | `python3 {{file}}` | `x3 + 2s` |
  | kt | `kotlinc {{file}} -include-runtime -d {{build}}/main.jar` | `java -jar {{build}}/main.jar` | `x2 + 1s` |

- [ ] 언어마다 Task 9 형식의 통합 테스트를 추가한다. 도구가 없으면 건너뛴다.
- [ ] `boj test`의 언어 판별: 폴더에 소스 파일이 여러 언어로 있으면 `-l`을 요구한다.
