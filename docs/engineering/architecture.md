# 구조

```
main.rs ── cli.rs (clap)
   │
   └─ cmd/ (명령마다 하나: new, test, copy, open)
        │  Ctx = 현재 폴더 + 프로젝트 루트 + 병합된 설정
        ├─ config/   설정 병합 (내장 → 전역 → 프로젝트)        순수
        ├─ template  자리 표시자 치환, 템플릿 탐색             순수(파일 읽기만)
        ├─ meta      문제 폴더의 .boj.toml 읽기/쓰기, 폴더 탐색
        ├─ fetch/    parse.rs (순수 HTML 파싱) / mod.rs (Fetcher, 네트워크) ◀ 네트워크 경계
        └─ judge/    time, compare (순수) / runner.rs (컴파일·실행)         ◀ 프로세스 경계
```

## 규칙

| 규칙 | 이유 | 깨지면 |
|---|---|---|
| 네트워크(`reqwest`)는 `src/fetch/mod.rs`에서만 쓴다 | 파서와 명령 로직을 fixture로 네트워크 없이 테스트하고, CI가 백준에 요청하지 않게 한다 | `cargo test --test architecture` 실패 |
| 외부 프로세스 실행(`std::process::Command`, `Stdio`)은 `src/judge/runner.rs`에서만 쓴다 | 시간 제한, 파이프 교착 방지, 컴파일러 없음 처리를 한 곳에서 보장한다 | 같은 테스트 실패 |
| 언어별 동작은 코드에 분기하지 않고 `config/builtin.toml`과 사용자 설정으로만 정한다 | 새 언어 추가가 설정 한 블록으로 끝나게 한다 | 리뷰로 확인 (기계 검사 없음) |
| 사용자 파일은 덮어쓰지 않는다. `.boj.toml`이 있으면 문제 정보는 다시 받지 않는다 | 반목표: 사용자 코드 손실, 백준 과다 요청 | `cmd::new` 테스트 `never_overwrites`, `refills_missing_test_only` |

## 상태 소유자

- 문제 정보: 문제 폴더의 `.boj.toml`. 쓰는 곳은 `cmd::new`뿐이다.
- 예제: `tests/N.in`, `N.out`. 쓰는 곳은 `cmd::new`뿐이고, 없는 파일만 만든다.
- 설정: 사용자 파일. 도구는 읽기만 한다.

## 예외

없음. 예외가 필요하면 `tests/architecture.rs`의 `allowed`에 파일을 추가한다. 이유와 함께 이 표에 적는다.
