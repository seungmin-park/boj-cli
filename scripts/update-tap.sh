#!/usr/bin/env bash
# 릴리스가 만든 boj.rb를 Homebrew tap 레포(seungmin-park/homebrew-tap)의 Formula/로 올린다.
# 사용: scripts/update-tap.sh v0.1.0
set -euo pipefail

TAG="${1:?사용법: scripts/update-tap.sh <태그, 예: v0.1.0>}"
REPO="seungmin-park/boj-cli"
TAP="seungmin-park/homebrew-tap"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

gh release download "$TAG" --repo "$REPO" --pattern 'boj.rb' --dir "$WORK"
grep -q "version \"${TAG#v}\"" "$WORK/boj.rb" || { echo "boj.rb의 버전이 $TAG와 달라요" >&2; exit 1; }

gh repo clone "$TAP" "$WORK/tap" -- --quiet
mkdir -p "$WORK/tap/Formula"
cp "$WORK/boj.rb" "$WORK/tap/Formula/boj.rb"
cd "$WORK/tap"
if git diff --quiet -- Formula/boj.rb && git ls-files --error-unmatch Formula/boj.rb >/dev/null 2>&1; then
  echo "tap이 이미 $TAG 상태예요"
  exit 0
fi
git add Formula/boj.rb
git commit -q -m "boj ${TAG#v}"
git push -q
echo "tap을 $TAG로 갱신했어요: brew install seungmin-park/tap/boj"
