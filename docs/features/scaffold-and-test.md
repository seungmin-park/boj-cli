# 문제 생성 → 채점

**목적:** 문제 번호 하나로 소스 템플릿과 백준 예제를 만들고, 로컬에서 바로 채점한다.

**전제:** `boj` 바이너리, 언어 도구(Java면 `javac`, `java`). 생성할 때만 네트워크가 필요하다.

## 흐름

1. `boj 1000`
   - `fetch::HttpFetcher`가 `https://www.acmicpc.net/problem/1000`을 1회 요청한다.
   - `fetch::parse`가 제목, 시간 제한, 스페셜 저지, 예제를 읽는다.
   - `cmd::new`가 다음을 만든다. 있는 파일은 건너뛴다.
     - `1000/.boj.toml`
     - `1000/Main.java`
     - `1000/tests/1.in`, `1.out`
2. 사용자가 `Main.java`를 푼다.
3. `boj test` (문제 폴더 안) 또는 `boj test 1000` (프로젝트 루트)
   - `cmd::test`가 임시 폴더에 컴파일한다.
   - 케이스마다 `judge::runner`로 실행하고 `judge::compare`로 비교한다.

## 관찰 가능한 결과

- 생성: "만들었어요" 목록과 폴더 경로, 종료 코드 0
- 채점: 케이스마다 `이름  판정  시간`, 마지막 줄 `N/M 맞음  (시간 제한 Xms)`
- 종료 코드: 0 전부 맞음 / 1 채점 실패(케이스 0개 포함) / 2 도구 오류

## 실패 경우

| 상황 | 결과 |
|---|---|
| 없는 문제 번호 | "N번 문제를 찾을 수 없어요", 코드 2, 폴더 안 만듦 |
| 문제 폴더 밖에서 `boj test` | "문제 폴더를 찾을 수 없어요", 코드 2 |
| 컴파일 에러 | "컴파일 에러"와 컴파일러 출력, 코드 1 |
| 스페셜 저지 불일치 | "확인 필요", 코드 1 |
| `javac` 없음 | "`javac`을 찾을 수 없어요", 코드 2 |

## copy, open

- `boj copy [번호] [--stdout]`: 언어 설정의 `strip` 정규식에 맞는 줄(Java는 `package` 줄)을 빼고 클립보드에 넣는다. 클립보드를 못 쓰면 표준 출력으로 낸다.
- `boj open [번호] [--print]`: 문제 페이지를 연다. `--print`면 URL만 출력한다.

## 검증

- 자동: `cargo test` (fixture 사용, 네트워크 없음). 실제 백준 요청은 `cargo test -- --ignored`.
- 실제 사용 흐름: [verify-boj 스킬](../../.agents/skills/verify-boj/SKILL.md)
- 실행 기록 (verify-boj 절차, 커밋 914dd74, macOS, rustc 1.100.0, javac 21.0.13, 실제 백준):

  | 명령 | 결과 | 종료 코드 |
  |---|---|---|
  | `boj 2309` | `.boj.toml`, `Main.java`, `tests/1.in`, `1.out` 생성. 메타: 일곱 난쟁이, 2000ms, 스페셜 저지 | 0 |
  | `boj 2309` (재실행) | "이미 준비돼 있어요" | 0 |
  | `boj test` (템플릿) | 확인 필요 | 1 |
  | `boj test` (정답) | 1/1 맞음 | 0 |
  | `boj copy --stdout` | `import`로 시작 | 0 |
  | `boj open --print` | `https://www.acmicpc.net/problem/2309` | 0 |
  | `boj 99999999` | "찾을 수 없어요", 폴더 안 생김 | 2 |
