# boj

문제 번호 하나로 백준 예제를 받아 오고, 바로 로컬에서 채점하는 CLI예요.

```console
$ boj 2309
만들었어요
  2309/.boj.toml
  2309/Main.java
  2309/tests/1.in
  2309/tests/1.out
폴더: 2309

$ cd 2309 && boj test
1  맞음  59ms
1/1 맞음  (시간 제한 5000ms)
```

- **예제를 원문 그대로 가져와요.** 백준 페이지에서 직접 읽어서, 손으로 옮기다 공백이나 줄바꿈을 틀릴 일이 없어요.
- **채점기가 들어 있어요.** 판정은 맞음, 틀림(첫 차이 줄 표시), 시간 초과, 런타임 에러, 컴파일 에러로 나와요. Gradle이나 IDE는 필요 없어요.
- **같은 입력이면 항상 같은 결과가 나오고 빨라요.** 문제 정보는 한 번만 받아서 저장하고, 그다음부터는 네트워크를 쓰지 않아요.
- **템플릿과 언어를 설정 파일로 바꿀 수 있어요.** v1은 Java를 기본 지원하고, 다른 언어는 설정 블록 하나로 추가해요.

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

Java 문제를 풀려면 `javac`와 `java`가 PATH에 있어야 해요.

## 명령

| 명령 | 하는 일 |
|---|---|
| `boj <번호>` | 폴더, 소스 템플릿, 예제(`tests/N.in`, `N.out`), 메타(`.boj.toml`)를 만들어요. 있는 파일은 건드리지 않고 빠진 예제만 채워요 |
| `boj <번호> -l <언어>` | 언어를 지정해요 |
| `boj <번호> -t <이름>` | 같은 언어의 다른 템플릿(`<언어>-<이름>.*`)을 써요 |
| `boj <번호> --offline` | 네트워크 없이 소스와 메타만 만들어요 |
| `boj test [번호]` | 컴파일하고 `tests/`의 모든 `*.in`/`*.out` 짝을 채점해요. 번호를 생략하면 현재 문제 폴더를 써요 |
| `boj copy [번호] [--stdout]` | `package` 줄 같은 걸 빼고 제출용 코드를 클립보드에 복사해요 |
| `boj open [번호] [--print]` | 문제 페이지를 브라우저로 열어요 |

`tests/`에 `my1.in`, `my1.out`처럼 직접 만든 케이스를 넣으면 같이 채점해요.

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
[lang.cpp]
file = "main.cpp"
template = "cpp"                      # templates/cpp.cpp 를 만들어 두세요
compile = "g++ -O2 -std=gnu++17 -o {{build}}/a.out {{file}}"
run = "{{build}}/a.out"

[lang.py]
file = "main.py"
template = "py"
run = "python3 {{file}}"
time = "x3 + 2s"                      # 백준 언어별 추가 시간 근사
```

C++, Python, Kotlin 내장 지원은 다음 버전에 들어가요.

## 한계

- **스페셜 저지 문제:** 출력이 예제와 달라도 정답일 수 있어요. 이럴 땐 "확인 필요"로만 표시해요.
- **인터랙티브 문제:** 지원하지 않아요.
- **시간 제한:** 백준 채점 서버와 같지 않아서 참고용이에요.

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
