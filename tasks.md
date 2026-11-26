# 작업 체크리스트

[plan.md](plan.md)를 기준으로 진행 상황을 관리한다.
- 각 항목의 자세한 내용(인터페이스, 테스트 이름, 값)은 plan.md의 같은 Task 번호에 있다.
- 항목을 끝내면 `[x]`로 바꾸고 커밋 해시를 적는다.

**상태 요약**

| 페이즈 | 내용 | 진행 |
|---|---|---|
| 0 | 기반 | 1/1 |
| 1 | 순수 로직 | 6/6 |
| 2 | 명령 | 5/5 |
| 3 | 공개 준비 | 진행 중 |
| 4 | 다언어 (v2) | 0/1 |

## Phase 0. 기반

- [x] **Task 1** 프로젝트 뼈대와 CLI 정의 — `de01884`
  - [x] Cargo 프로젝트와 의존성 추가
  - [x] clap 정의
    - [x] `boj <번호>` 형식
    - [x] 하위 명령 `test`/`copy`/`open`
    - [x] 옵션 `-l`/`-t`/`--offline`
  - [x] `BojError`와 종료 코드 규칙(0/1/2)
  - [x] CI(fmt, clippy, test × macOS/Linux/Windows)
  - [x] 테스트: `help_lists_commands`, `no_args_is_tool_error`

## Phase 1. 순수 로직

- [x] **Task 2** 문제 모델과 HTML 파서 — `cdfc6a7`
  - [x] fixture 받기
    - [x] 1000, 1260(예제 3개), 2580(스페셜 저지), 15552(시간 표기 변형)
    - [x] 없는 문제, 예제 없음/빈 출력
  - [x] 제목, 예제, 시간 제한, 스페셜 저지 파싱
  - [x] 줄바꿈 정규화(`\r\n` → `\n`, 끝 개행 보장)
  - [x] 없는 문제는 `ProblemNotFound`
- [x] **Task 3** Fetcher — `409683a`
  - [x] `Fetcher` trait
  - [x] `HttpFetcher`(User-Agent, 10초 타임아웃, 404 처리)
  - [x] `FixtureFetcher`와 `BOJ_FIXTURE_DIR`
  - [x] 실제 요청 테스트는 `#[ignore]`로
- [x] **Task 4** 시간 공식 `x2 + 1s` — `e73c28f`
  - [x] 파싱(`x`, `s`, `ms`, 빈 문자열)
  - [x] 잘못된 형식은 설정 오류로
- [x] **Task 5** 출력 비교 — `e73c28f`
  - [x] 줄 끝 공백, 끝 빈 줄, `\r` 무시
  - [x] 줄 앞 공백은 차이로 판정
  - [x] 첫 차이 줄 번호와 기대/실제 값 보고
- [x] **Task 6** 설정 병합 — `c7de1ab`
  - [x] 내장 `builtin.toml`(Java, `--release 11`)
  - [x] 병합 순서: 내장 → 전역(`BOJ_CONFIG_DIR` 또는 설정 폴더) → 프로젝트(`.boj/`)
  - [x] 언어 표는 필드 단위로 덮어쓰기
  - [x] 새 언어 추가, 필수 필드 누락은 오류
  - [x] 프로젝트 루트를 위로 올라가며 탐색
- [x] **Task 7** 템플릿 — `a40c271`
  - [x] 자리 표시자 7개 치환, 모르는 자리 표시자는 오류
  - [x] 빈 `package ;` 줄 제거
  - [x] `java_package`, `main_class`, 숫자로 시작하는 패키지 경고
  - [x] 템플릿 탐색 순서(프로젝트 → 전역 → 내장)와 `-t` 변형
  - [x] 내장 Java 템플릿(static BufferedReader 뼈대)

## Phase 2. 명령

- [x] **Task 8** `boj <번호>` 생성 — `ccc949b`
  - [x] `.boj.toml` 메타 읽기/쓰기
  - [x] 소스, 예제, 메타 생성
  - [x] 덮어쓰기 금지, 빠진 예제만 채우기
  - [x] `src/main/java/` 아래면 패키지 줄 넣기
  - [x] `--offline`
  - [x] 예제 없음 경고, 빈 출력 예제 처리
- [x] **Task 9** 컴파일과 실행기 — `c692395`
  - [x] 컴파일 오류 stderr 전달
  - [x] 시간 초과 시 kill
  - [x] 비정상 종료 코드 처리
  - [x] 대량 출력, 입력을 안 읽는 프로그램에서도 멈추지 않음
- [x] **Task 10** `boj test` — `1622335`
  - [x] 케이스 수집과 정렬(숫자 먼저, 그다음 이름순)
  - [x] 판정(맞음/틀림/시간 초과/런타임 에러/확인 필요)과 실행 시간 출력
  - [x] 언어 보정 시간 제한, 모르면 5000ms
  - [x] 문제 폴더 밖이면 종료 코드 2
- [x] **Task 11** `boj copy`, `boj open` — `914dd74`
  - [x] `strip` 정규식으로 줄 제거
  - [x] 클립보드 복사, 실패하거나 `--stdout`이면 표준 출력
  - [x] 문제 페이지 열기, `--print`
- [x] **Task 12** end-to-end — `914dd74`
  - [x] 생성 → 채점
  - [x] 재실행해도 덮어쓰지 않음
  - [x] 한글과 공백이 든 경로
  - [x] `copy --stdout`, `open --print`
  - [x] 모르는 언어는 종료 코드 2

## 프로젝트 기반 (agent-engineering)

- [x] 구조 규칙과 기계 검사 `tests/architecture.rs` (네트워크는 fetch, 프로세스는 runner에서만). 위반을 넣으면 실패하는 것 확인
- [x] 기능 지도 `docs/features/`
- [x] 실제 흐름 검증 스킬 `.agents/skills/verify-boj/` — 실제 백준과 javac로 실행, 기록은 기능 문서에
- [x] CI(fmt, clippy, test × 3 OS, `BOJ_REQUIRE_JAVA=1`)

## Phase 3. 공개 준비

- [ ] **Task 13** 문서와 배포
  - [x] README(설치, 명령, 설정과 템플릿 예시, 한계)
  - [x] LICENSE: MIT OR Apache-2.0 (Rust 생태계 관례, 사용자 위임으로 결정)
  - [ ] cargo-dist 릴리스 워크플로와 Homebrew tap
  - [ ] `v0.1.0` 릴리스, `brew install` 확인
  - [x] 레포 공개 전환 (2026-11-27)
  - [ ] crates.io 게시 (crates.io 로그인 토큰 필요: 사용자)

## Phase 4. 다언어 (v2)

- [ ] **Task 14** C++, Python, Kotlin 내장값
  - [ ] 언어별 설정과 템플릿
  - [ ] 언어별 통합 테스트(도구가 없으면 건너뜀)
  - [ ] 여러 언어 소스가 같이 있으면 `-l` 요구
