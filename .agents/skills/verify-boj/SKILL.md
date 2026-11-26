---
name: verify-boj
description: boj CLI를 실제 백준과 실제 Java로 끝까지 돌려 확인한다. 생성·채점·복사·열기 동작을 바꾼 뒤, 또는 릴리스 전에 쓴다.
---

# verify-boj

자동 테스트(`cargo test`)는 fixture를 쓴다. 이 절차는 **실제 백준 페이지와 실제 컴파일러**로 사용자 흐름을 확인한다.

## 준비 확인 (doctor)

```bash
cargo --version && javac -version && java -version
curl -sL -o /dev/null -w "%{http_code}\n" https://www.acmicpc.net/problem/1000   # 200이어야 함
```

## 빌드

```bash
cargo build --release
BOJ=$PWD/target/release/boj
```

## 실행 (임시 폴더에서, 사용자 설정이 섞이지 않게)

```bash
WORK=$(mktemp -d) && CFG=$(mktemp -d) && cd "$WORK"
export BOJ_CONFIG_DIR=$CFG

$BOJ 2309                         # 기대: 만들었어요 + 2309/Main.java, tests/1.in, 1.out, .boj.toml
$BOJ 2309                         # 기대: 이미 준비돼 있어요 (재요청·덮어쓰기 없음)
cat 2309/tests/1.in               # 기대: 백준 예제 입력 원문
cd 2309 && $BOJ test; echo "exit=$?"   # 템플릿 그대로면 기대: 확인 필요(2309는 스페셜 저지), exit=1
```

정답 코드를 넣고 다시 채점한다.

```bash
cat > Main.java <<'JAVA'
import java.util.*;
public class Main { public static void main(String[] a) { Scanner s = new Scanner(System.in);
  int[] h = new int[9]; int sum = 0; for (int i = 0; i < 9; i++) { h[i] = s.nextInt(); sum += h[i]; }
  for (int i = 0; i < 9; i++) for (int j = i + 1; j < 9; j++) if (sum - h[i] - h[j] == 100) {
    List<Integer> r = new ArrayList<>(); for (int k = 0; k < 9; k++) if (k != i && k != j) r.add(h[k]);
    Collections.sort(r); StringBuilder sb = new StringBuilder(); for (int x : r) sb.append(x).append('\n');
    System.out.print(sb); return; } } }
JAVA
$BOJ test; echo "exit=$?"          # 기대: 1/1 맞음, exit=0
$BOJ copy --stdout | head -3       # 기대: 소스 그대로 (package 줄 없음)
$BOJ open --print                  # 기대: https://www.acmicpc.net/problem/2309
cd "$WORK" && $BOJ 99999999; echo "exit=$?"   # 기대: 찾을 수 없어요, exit=2, 폴더 없음
```

## 다른 언어 (도구가 있는 것만)

```bash
cd "$WORK" && $BOJ 1000 -l py && $BOJ 1000 -l cpp && $BOJ 1000 -l kt   # 같은 폴더에 main.py, main.cpp, Main.kt 추가
cd 1000 && $BOJ test; echo "exit=$?"                                     # 기대: 여러 언어 소스 안내, exit=2
printf 'a, b = map(int, input().split())\nprint(a + b)\n' > main.py
printf '#include <iostream>\nint main(){long long a,b;std::cin>>a>>b;std::cout<<a+b<<"\\n";}\n' > main.cpp
printf 'fun main() {\n    val (a, b) = readLine()!!.trim().split(" ").map { it.toInt() }\n    println(a + b)\n}\n' > Main.kt
for l in py cpp kt; do $BOJ test -l $l; echo "$l exit=$?"; done            # 기대: 각각 1/1 맞음, exit=0
```

## 증거

명령, 출력, 종료 코드를 PR 본문이나 `docs/features/scaffold-and-test.md`의 실행 기록에 남긴다. 커밋 SHA, OS, rustc, JDK 버전도 함께 적는다.

## 정리

`rm -rf "$WORK" "$CFG"` — 이 절차가 만든 임시 폴더만 지운다.

## 관련

- 기능 지도: [docs/features/README.md](../../../docs/features/README.md)
- 구조 규칙: [docs/engineering/architecture.md](../../../docs/engineering/architecture.md)
