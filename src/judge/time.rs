use crate::error::BojError;

/// 백준 언어별 추가 시간 근사: 기본 시간 × mul + add_ms
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRule {
    pub mul: f64,
    pub add_ms: u64,
}

impl TimeRule {
    /// "x2 + 1s", "x1.5", "+ 500ms", "" 형식
    pub fn parse(s: &str) -> Result<TimeRule, BojError> {
        let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        let re =
            regex::Regex::new(r"^(?:x(\d+(?:\.\d+)?))?(?:\+(\d+)(ms|s))?$").expect("고정된 정규식");
        let caps = re.captures(&compact).ok_or_else(|| {
            BojError::Config(format!(
                "시간 공식을 읽을 수 없어요: {s:?} (예: \"x2 + 1s\")"
            ))
        })?;
        let mul = caps
            .get(1)
            .map_or(Ok(1.0), |m| m.as_str().parse::<f64>())
            .map_err(|e| BojError::Config(e.to_string()))?;
        let add_ms = match (caps.get(2), caps.get(3).map(|m| m.as_str())) {
            (Some(n), Some(unit)) => {
                let n: u64 = n
                    .as_str()
                    .parse()
                    .map_err(|_| BojError::Config(format!("시간 공식 숫자가 너무 커요: {s:?}")))?;
                if unit == "s" {
                    n * 1000
                } else {
                    n
                }
            }
            _ => 0,
        };
        Ok(TimeRule { mul, add_ms })
    }

    pub fn apply(&self, base_ms: u64) -> u64 {
        (base_ms as f64 * self.mul).round() as u64 + self.add_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(rule: &str, base: u64) -> u64 {
        TimeRule::parse(rule).unwrap().apply(base)
    }

    #[test]
    fn rules() {
        assert_eq!(ms("x2 + 1s", 1000), 3000);
        assert_eq!(ms("x3 + 2s", 2000), 8000);
        assert_eq!(ms("x1.5", 1000), 1500);
        assert_eq!(ms("+ 500ms", 1000), 1500);
        assert_eq!(ms("", 700), 700);
        assert_eq!(ms("x2+1s", 500), 2000);
    }

    #[test]
    fn invalid_rule_is_config_error() {
        assert!(matches!(TimeRule::parse("fast"), Err(BojError::Config(_))));
        assert!(matches!(TimeRule::parse("x"), Err(BojError::Config(_))));
        assert!(matches!(TimeRule::parse("+ 3h"), Err(BojError::Config(_))));
    }
}
