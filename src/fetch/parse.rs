use scraper::{Html, Selector};

use crate::error::BojError;
use crate::problem::{Problem, Sample};

/// 백준 문제 페이지 HTML에서 제목, 시간 제한, 스페셜 저지 여부, 예제를 읽는다.
pub fn parse_problem(id: u32, html: &str) -> Result<Problem, BojError> {
    let doc = Html::parse_document(html);

    let title = select_text(&doc, "#problem_title")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .ok_or(BojError::ProblemNotFound(id))?;

    let time_limit_ms = select_text(&doc, "#problem-info tbody td").and_then(|t| parse_seconds(&t));
    let special_judge = doc.select(&sel(".problem-label-spj")).next().is_some();

    let mut samples = Vec::new();
    for n in 1.. {
        let input = select_text(&doc, &format!("#sample-input-{n}"));
        let output = select_text(&doc, &format!("#sample-output-{n}"));
        match (input, output) {
            (Some(i), Some(o)) => samples.push(Sample {
                input: normalize(&i),
                output: normalize(&o),
            }),
            _ => break,
        }
    }

    Ok(Problem {
        id,
        title,
        time_limit_ms,
        special_judge,
        samples,
    })
}

fn sel(css: &str) -> Selector {
    Selector::parse(css).expect("고정된 CSS 선택자")
}

fn select_text(doc: &Html, css: &str) -> Option<String> {
    doc.select(&sel(css)).next().map(|e| e.text().collect())
}

/// "2 초", "0.5 초", "1 초 (하단 참고)" → 밀리초
fn parse_seconds(text: &str) -> Option<u64> {
    let num: String = text
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let secs: f64 = num.parse().ok()?;
    Some((secs * 1000.0).round() as u64)
}

/// 줄바꿈을 \n으로 맞추고, 비어 있지 않으면 끝 개행을 하나 보장한다.
fn normalize(text: &str) -> String {
    let mut s = text.replace("\r\n", "\n").replace('\r', "\n");
    if !s.is_empty() && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}.html", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(path).unwrap()
    }

    #[test]
    fn parses_1260() {
        let p = parse_problem(1260, &fixture("1260")).unwrap();
        assert_eq!(p.id, 1260);
        assert_eq!(p.title, "DFS와 BFS");
        assert_eq!(p.samples.len(), 3);
        assert_eq!(p.time_limit_ms, Some(2000));
        assert!(!p.special_judge);
        assert!(p.samples[0].input.starts_with("4 5 1\n"));
        assert!(p.samples[0].output.ends_with('\n'));
        assert!(!p.samples[0].input.contains('\r'));
    }

    #[test]
    fn parses_spj_2580() {
        let p = parse_problem(2580, &fixture("2580")).unwrap();
        assert!(p.special_judge);
        assert_eq!(p.time_limit_ms, Some(1000));
    }

    #[test]
    fn parses_time_with_note_15552() {
        let p = parse_problem(15552, &fixture("15552")).unwrap();
        assert_eq!(p.time_limit_ms, Some(1000));
    }

    #[test]
    fn not_found_page() {
        let r = parse_problem(99999999, &fixture("notfound"));
        assert!(matches!(r, Err(BojError::ProblemNotFound(99999999))));
    }

    #[test]
    fn no_samples() {
        let p = parse_problem(1000, &fixture("nosample")).unwrap();
        assert!(p.samples.is_empty());
    }

    #[test]
    fn empty_output_sample() {
        let p = parse_problem(1000, &fixture("emptyout")).unwrap();
        assert_eq!(p.samples.len(), 1);
        assert_eq!(p.samples[0].output, "");
        assert_eq!(p.samples[0].input, "1 2\n");
    }

    #[test]
    fn crlf_samples_are_normalized() {
        let html = "<span id=\"problem_title\">X</span><pre id=\"sample-input-1\">1 2\r\n3\r\n</pre><pre id=\"sample-output-1\">4</pre>";
        let p = parse_problem(1, html).unwrap();
        assert_eq!(p.samples[0].input, "1 2\n3\n");
        assert_eq!(p.samples[0].output, "4\n");
    }

    #[test]
    fn fractional_seconds() {
        let html = r#"<span id="problem_title">X</span>
            <table id="problem-info"><tbody><tr><td>0.5 초 </td><td>256 MB</td></tr></tbody></table>"#;
        assert_eq!(parse_problem(1, html).unwrap().time_limit_ms, Some(500));
    }
}
