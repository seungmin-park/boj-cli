pub mod parse;

use std::path::PathBuf;
use std::time::Duration;

use crate::error::BojError;
use crate::problem::Problem;

/// 문제 정보를 가져오는 경계. 네트워크는 이 trait 뒤에만 있다.
pub trait Fetcher {
    fn fetch(&self, id: u32) -> Result<Problem, BojError>;
}

pub const USER_AGENT: &str = concat!(
    "boj-cli/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/seungmin-park/boj-cli)"
);

pub fn problem_url(id: u32) -> String {
    format!("https://www.acmicpc.net/problem/{id}")
}

/// 백준에 문제당 한 번 요청한다.
pub struct HttpFetcher;

impl Fetcher for HttpFetcher {
    fn fetch(&self, id: u32) -> Result<Problem, BojError> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| BojError::Network(e.to_string()))?;
        let res = client
            .get(problem_url(id))
            .send()
            .map_err(|e| BojError::Network(e.to_string()))?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(BojError::ProblemNotFound(id));
        }
        if !res.status().is_success() {
            return Err(BojError::Network(format!("HTTP {}", res.status())));
        }
        let html = res.text().map_err(|e| BojError::Network(e.to_string()))?;
        parse::parse_problem(id, &html)
    }
}

/// 테스트 전용: `{dir}/{id}.html`을 읽는다.
pub struct FixtureFetcher {
    pub dir: PathBuf,
}

impl Fetcher for FixtureFetcher {
    fn fetch(&self, id: u32) -> Result<Problem, BojError> {
        let path = self.dir.join(format!("{id}.html"));
        let html = std::fs::read_to_string(&path).map_err(|_| BojError::ProblemNotFound(id))?;
        parse::parse_problem(id, &html)
    }
}

/// `BOJ_FIXTURE_DIR`가 있으면 fixture, 없으면 실제 HTTP를 쓴다 (테스트용 스위치).
pub fn fetcher_from_env() -> Box<dyn Fetcher> {
    match std::env::var_os("BOJ_FIXTURE_DIR") {
        Some(dir) => Box::new(FixtureFetcher { dir: dir.into() }),
        None => Box::new(HttpFetcher),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures() -> FixtureFetcher {
        FixtureFetcher {
            dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        }
    }

    #[test]
    fn fixture_fetcher_reads_file() {
        assert_eq!(fixtures().fetch(1260).unwrap().title, "DFS와 BFS");
    }

    #[test]
    fn fixture_fetcher_missing_file() {
        assert!(matches!(
            fixtures().fetch(424242),
            Err(BojError::ProblemNotFound(424242))
        ));
    }

    #[test]
    #[ignore = "실제 백준에 요청한다: cargo test -- --ignored"]
    fn live_fetch_1000() {
        assert_eq!(HttpFetcher.fetch(1000).unwrap().title, "A+B");
    }

    #[test]
    #[ignore = "실제 백준에 요청한다: cargo test -- --ignored"]
    fn live_fetch_missing_is_not_found() {
        assert!(matches!(
            HttpFetcher.fetch(99999999),
            Err(BojError::ProblemNotFound(99999999))
        ));
    }
}
