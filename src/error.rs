use thiserror::Error;

/// 도구 자체의 오류. 채점 실패(틀림 등)는 오류가 아니라 결과라서 여기 없다.
#[derive(Debug, Error)]
pub enum BojError {
    #[error("설정 오류: {0}")]
    Config(String),
    #[error("네트워크 오류: {0}")]
    Network(String),
    #[error("{0}번 문제를 찾을 수 없어요")]
    ProblemNotFound(u32),
    #[error("문제 폴더를 찾을 수 없어요. 문제 폴더 안에서 실행하거나 번호를 같이 적어 주세요")]
    NoProblemDir,
    #[error("파일 오류: {0}")]
    Io(#[from] std::io::Error),
    #[error("템플릿 오류: {0}")]
    Template(String),
    #[error("사용법: {0}")]
    Usage(String),
}

impl BojError {
    /// 도구 오류는 모두 2. 0은 성공, 1은 채점 실패다.
    pub fn exit_code(&self) -> i32 {
        2
    }
}
