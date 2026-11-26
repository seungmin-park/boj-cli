use crate::cmd::Ctx;
use crate::error::BojError;
use crate::meta::Meta;

/// 패턴에 맞는 줄을 지우고, 그 뒤 맨 앞에 남은 빈 줄도 지운다.
pub fn strip_lines(src: &str, patterns: &[String]) -> Result<String, BojError> {
    let res = patterns
        .iter()
        .map(|p| {
            regex::Regex::new(p).map_err(|e| BojError::Config(format!("strip 정규식 {p:?}: {e}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if res.is_empty() {
        return Ok(src.to_string());
    }
    let kept: Vec<&str> = src
        .split_inclusive('\n')
        .filter(|l| {
            let line = l.trim_end_matches(['\n', '\r']);
            !res.iter().any(|r| r.is_match(line))
        })
        .collect();
    let first = kept
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(kept.len());
    Ok(kept[first..].concat())
}

pub enum Copied {
    Clipboard,
    /// 클립보드를 못 쓰거나 --stdout이면 내용을 돌려받아 출력한다.
    Stdout(String),
}

/// `boj copy`: 제출용 코드를 클립보드(또는 표준 출력)로.
pub fn run_copy(
    ctx: &Ctx,
    id: Option<u32>,
    lang: Option<&str>,
    stdout_only: bool,
) -> Result<Copied, BojError> {
    let dir = ctx.resolve_problem_dir(id)?;
    let meta = Meta::read(&dir)?.ok_or(BojError::NoProblemDir)?;
    let lang = ctx.config.lang(&ctx.resolve_lang(&dir, &meta, lang)?)?;
    let path = dir.join(&lang.file);
    let src = std::fs::read_to_string(&path)
        .map_err(|_| BojError::Usage(format!("소스 파일이 없어요: {}", path.display())))?;
    let code = strip_lines(&src, &lang.strip)?;
    if stdout_only {
        return Ok(Copied::Stdout(code));
    }
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(code.clone())) {
        Ok(()) => Ok(Copied::Clipboard),
        Err(_) => Ok(Copied::Stdout(code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn java_strip() -> Vec<String> {
        vec![r"^\s*package\s+[\w.]+;\s*$".to_string()]
    }

    #[test]
    fn strips_package_line() {
        let out = strip_lines(
            "package p1;\n\nimport java.io.*;\nclass Main {}\n",
            &java_strip(),
        )
        .unwrap();
        assert_eq!(out, "import java.io.*;\nclass Main {}\n");
    }

    #[test]
    fn keeps_other_lines() {
        let src = "import java.io.*;\n\n// package note;\nclass Main {}\n";
        assert_eq!(strip_lines(src, &java_strip()).unwrap(), src);
        assert_eq!(strip_lines(src, &[]).unwrap(), src);
    }

    #[test]
    fn bad_regex_errors() {
        assert!(matches!(
            strip_lines("x", &["(".into()]),
            Err(BojError::Config(_))
        ));
    }
}
