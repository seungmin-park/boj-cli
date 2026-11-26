/// 출력 비교 결과. line은 1부터 센다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Compare {
    Match,
    Mismatch {
        line: usize,
        expected: String,
        actual: String,
    },
}

/// 백준 방식: 줄 끝 공백, 파일 끝 빈 줄, \r은 무시하고 줄 앞 공백은 구분한다.
pub fn compare(expected: &str, actual: &str) -> Compare {
    let e = lines(expected);
    let a = lines(actual);
    for i in 0..e.len().max(a.len()) {
        let (el, al) = (e.get(i), a.get(i));
        if el != al {
            let show = |l: Option<&&str>| l.map_or_else(|| "(없음)".to_string(), |s| s.to_string());
            return Compare::Mismatch {
                line: i + 1,
                expected: show(el),
                actual: show(al),
            };
        }
    }
    Compare::Match
}

fn lines(text: &str) -> Vec<&str> {
    let mut v: Vec<&str> = text
        .split('\n')
        .map(|l| l.trim_end_matches([' ', '\t', '\r']))
        .collect();
    while v.last() == Some(&"") {
        v.pop();
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mismatch(line: usize, expected: &str, actual: &str) -> Compare {
        Compare::Mismatch {
            line,
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    #[test]
    fn table() {
        let cases = [
            ("1 2\n", "1 2   \n\n\n", Compare::Match),
            ("1\n2\n", "1\r\n2\r\n", Compare::Match),
            ("", "\n", Compare::Match),
            ("", "", Compare::Match),
            (" 1\n", "1\n", mismatch(1, " 1", "1")),
            ("1\n2\n", "1\n", mismatch(2, "2", "(없음)")),
            ("1\n", "1\n2\n", mismatch(2, "(없음)", "2")),
            ("a\nb\n", "a\nc\n", mismatch(2, "b", "c")),
            ("1\n\n2\n", "1\n2\n", mismatch(2, "", "2")),
        ];
        for (e, a, want) in cases {
            assert_eq!(compare(e, a), want, "expected={e:?} actual={a:?}");
        }
    }
}
