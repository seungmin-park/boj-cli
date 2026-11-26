# boj

[![ci](https://github.com/seungmin-park/boj-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/seungmin-park/boj-cli/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/seungmin-park/boj-cli)](https://github.com/seungmin-park/boj-cli/releases/latest)

`boj 1000` 한 줄로 백준 예제를 파일로 받고, `boj test`로 바로 채점하는 CLI예요. Java, C++, Python, Kotlin을 지원해요.

```console
$ boj 1000
만들었어요
  1000/.boj.toml
  1000/Main.java
  1000/tests/1.in
  1000/tests/1.out
폴더: 1000

$ cd 1000 && boj test
1  틀림  42ms
    줄 1: 기대 "3" / 실제 "-1"
0/1 맞음  (시간 제한 5000ms)

$ boj test        # a - b 를 a + b 로 고친 뒤
1  맞음  42ms
1/1 맞음  (시간 제한 5000ms)

$ boj copy        # package 줄을 뺀 제출용 코드를 클립보드로
제출용 코드를 클립보드에 복사했어요
```

- **예제는 백준 원문 그대로예요.** 문제 페이지의 예제 블록을 그대로 저장해서 공백과 줄바꿈까지 같아요. 손으로 옮기거나 AI 챗에 옮겨 달라고 할 필요가 없어요.
- **틀리면 처음 달라진 줄을 보여 줘요.** 판정은 맞음, 틀림, 시간 초과, 런타임 에러, 컴파일 에러로 나뉘어요. 직접 만든 `tests/my1.in`, `my1.out`도 같이 채점해요.
- **문제당 한 번만 요청해요.** 받아 온 제목, 시간 제한, 예제는 문제 폴더에 저장해요. 그 뒤로는 네트워크 없이 채점해요.
- **평소 쓰던 템플릿으로 시작해요.** `static BufferedReader br`로 시작하는 내 뼈대를 `.boj/templates/`에 두면 그걸로 만들어요. 다른 언어도 설정 블록 하나로 추가해요.

## 설치

```bash
# macOS / Linux (Homebrew)
brew install seungmin-park/tap/boj

# macOS / Linux (설치 스크립트)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/seungmin-park/boj-cli/releases/latest/download/boj-installer.sh | sh

# Windows (PowerShell)
powershell -ExecutionPolicy Bypass -c "irm https://github.com/seungmin-park/boj-cli/releases/latest/download/boj-installer.ps1 | iex"

# 소스에서
cargo install --git https://github.com/seungmin-park/boj-cli
```

업데이트는 `brew upgrade boj`로 해요. 풀 언어의 컴파일러나 인터프리터는 PATH에 있어야 해요.

| 언어 (`-l`) | 소스 파일 | 필요한 도구 | 시간 제한 |
|---|---|---|---|
| `java` (기본) | `Main.java` | `javac`, `java` | ×2 + 1초 |
| `cpp` | `main.cpp` | `g++` | 그대로 |
| `py` | `main.py` | `python3` | ×3 + 2초 |
| `kt` | `Main.kt` | `kotlinc`, `java` | ×2 + 1초 |

- 시간 제한은 백준의 언어별 추가 시간을 흉내 낸 값이에요.
- Windows에는 `python3` 명령이 없는 경우가 많아요. 그럴 땐 설정에 `[lang.py] run = "python {{file}}"`을 넣어 주세요.

## 명령

| 명령 | 하는 일 |
|---|---|
| `boj <번호>` | 폴더, 소스 템플릿, 예제(`tests/N.in`, `N.out`), 메타(`.boj.toml`)를 만들어요. 있는 파일은 건드리지 않고 빠진 예제만 채워요 |
| `boj <번호> -l <언어>` | 언어를 지정해요 |
| `boj <번호> -t <이름>` | 같은 언어의 다른 템플릿(`<언어>-<이름>.*`)을 써요 |
| `boj <번호> --offline` | 네트워크 없이 소스와 메타만 만들어요 |
| `boj test [번호] [-l 언어]` | 컴파일하고 `tests/`의 모든 `*.in`/`*.out` 짝을 채점해요. 번호를 생략하면 현재 문제 폴더를 써요 |
| `boj copy [번호] [-l 언어] [--stdout]` | `package` 줄 같은 걸 빼고 제출용 코드를 클립보드에 복사해요 |
| `boj open [번호] [--print]` | 문제 페이지를 브라우저로 열어요 |

`test`와 `copy`는 다음 순서로 언어를 정해요.
1. `-l`로 지정한 언어
2. 폴더에 소스가 하나뿐이면 그 언어
3. 처음 만들 때의 언어

같은 문제를 두 언어로 풀 수도 있어요. `boj 1000 -l py`로 `main.py`를 더 만들면 폴더에 소스가 두 개가 돼요. 그때부터는 `boj test -l py`처럼 언어를 골라 달라고 안내해요.

종료 코드는 다음과 같아요.

| 코드 | 뜻 |
|---|---|
| 0 | 성공 |
| 1 | 채점 실패. 케이스가 하나도 없는 경우도 포함해요 |
| 2 | 도구 오류. 설정 오류, 네트워크 오류, 없는 문제, 문제 폴더를 못 찾은 경우예요 |

## 설정

설정은 아래 순서로 덮어써요. 바꾸고 싶은 키만 적으면 돼요.
1. 내장 기본값
2. 전역 설정 `~/.config/boj/config.toml`
3. 프로젝트 설정 `.boj/config.toml`. 위로 올라가며 찾아요.

```toml
default = "java"
path = "src/main/java/p{{id}}"     # 기본값은 "{{id}}"

[lang.java]
compile = "javac -encoding UTF-8 --release 8 -d {{build}} {{file}}"   # 기본 --release 11
```

- `src/main/java/` 아래에 만들면 `package p1000;` 줄이 자동으로 들어가요.
- `boj copy`는 이 줄을 빼고 복사해요.
- Java 패키지 이름은 숫자로 시작할 수 없어요. 그래서 `p{{id}}`처럼 앞에 글자를 붙여 주세요.

### 템플릿

템플릿은 아래 순서로 찾아요.
1. `.boj/templates/java.java`
2. `~/.config/boj/templates/java.java`
3. 내장 템플릿

자리 표시자는 다음과 같아요.

| 자리 표시자 | 뜻 |
|---|---|
| `{{id}}` | 문제 번호 |
| `{{title}}` | 문제 제목 |
| `{{dir}}` | 문제 폴더 |
| `{{file}}` | 소스 파일 |
| `{{build}}` | 빌드 폴더 (명령에서만) |
| `{{package}}` | Java 패키지 이름 |
| `{{main_class}}` | 실행할 클래스 |

### 다른 언어 추가

```toml
[lang.rs]
file = "main.rs"
template = "rs"                       # templates/rs.rs 를 만들어 두세요
compile = "rustc -O -o {{build}}/main {{file}}"
run = "{{build}}/main"
time = ""                             # 백준 언어별 추가 시간 근사 (예: "x2 + 1s")
```

PyPy로 채점하려면 `[lang.py] run = "pypy3 {{file}}"`처럼 내장 언어의 값만 바꾸면 돼요.

## 한계

- **스페셜 저지 문제:** 출력이 예제와 달라도 정답일 수 있어요. 이럴 땐 "확인 필요"로만 표시해요.
- **인터랙티브 문제:** 지원하지 않아요.
- **시간 제한:** 백준 채점 서버와 같지 않아서 참고용이에요.
- **macOS 첫 실행 지연:** 새로 컴파일한 실행 파일을 처음 실행하면 macOS 보안 검사 때문에 첫 케이스에만 수십~수백 ms가 더 찍혀요. C++ A+B 기준 첫 케이스는 60~230ms, 나머지는 1ms였어요.

## 개발

```bash
cargo test                    # fixture 사용, 네트워크 없음
cargo test -- --ignored       # 실제 백준 요청 테스트
```

- 구조와 규칙: [docs/engineering/architecture.md](docs/engineering/architecture.md)
- 기능 지도: [docs/features/README.md](docs/features/README.md)
- 실제 흐름 검증 절차: [.agents/skills/verify-boj/SKILL.md](.agents/skills/verify-boj/SKILL.md)
- 릴리스 절차: [docs/release.md](docs/release.md)
- 계획: [plan.md](plan.md), 진행: [tasks.md](tasks.md)

## 라이선스

MIT 또는 Apache-2.0 중 하나를 골라 쓸 수 있어요.
