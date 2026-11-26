/// 백준 문제 한 개에서 이 도구가 쓰는 정보.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub id: u32,
    pub title: String,
    pub time_limit_ms: Option<u64>,
    pub special_judge: bool,
    pub samples: Vec<Sample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub input: String,
    pub output: String,
}
