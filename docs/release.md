# 릴리스 절차

릴리스는 [cargo-dist](https://github.com/axodotdev/cargo-dist) 0.34가 만든 `.github/workflows/release.yml`이 맡는다. 설정은 `dist-workspace.toml`에 있다.

1. `Cargo.toml`의 `version`을 올리고 PR로 머지한다. CI가 통과해야 한다.
2. `main`에서 태그를 올린다.

   ```bash
   git tag v0.1.0 && git push origin v0.1.0
   ```

3. release 워크플로가 끝나면 GitHub Release에 다음이 올라갔는지 확인한다.
   - 5개 플랫폼 바이너리
   - `boj-installer.sh`, `boj-installer.ps1`
   - `boj.rb`
4. tap을 갱신한다.

   ```bash
   scripts/update-tap.sh v0.1.0
   ```

5. 설치를 확인한다.

   ```bash
   brew update && brew install seungmin-park/tap/boj   # 처음 설치
   brew upgrade boj                                    # 이미 설치된 경우
   boj --version
   ```

## 왜 tap 갱신이 수동인가

- dist의 `publish-jobs = ["homebrew"]`를 켜면 릴리스 워크플로가 tap에 직접 푸시한다.
- 그러려면 tap 레포에 쓸 수 있는 토큰을 이 레포의 `HOMEBREW_TAP_TOKEN` 비밀값으로 넣어야 한다.
- 토큰은 **tap 레포 하나에만 Contents 쓰기 권한**을 준 fine-grained PAT로 만들어야 한다.
- 그런 토큰이 생기면 `publish-jobs`에 `"homebrew"`를 넣고 `dist generate`를 다시 돌린다. 그 뒤로 4단계는 필요 없다.

## 기타

- crates.io 게시(`cargo publish`)는 crates.io 토큰이 필요하다. 아직 하지 않았다.
