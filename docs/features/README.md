# 기능 지도

| 기능 | 사용자 목적 | 문서 | 검증 |
|---|---|---|---|
| 문제 생성 → 채점 | 번호 하나로 소스·예제를 만들고 바로 채점 | [scaffold-and-test.md](scaffold-and-test.md) | `cargo test`, [verify-boj 스킬](../../.agents/skills/verify-boj/SKILL.md) |
| 제출용 복사 | 패키지 줄 없이 클립보드로 | [scaffold-and-test.md](scaffold-and-test.md#copy-open) | `cargo test --test cli e2e_copy` |
| 문제 페이지 열기 | 브라우저로 문제 보기 | [scaffold-and-test.md](scaffold-and-test.md#copy-open) | `cargo test --test cli e2e_open` |
