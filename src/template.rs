use std::collections::BTreeMap;
use std::path::{Component, Path};

use crate::error::BojError;

pub type Vars = BTreeMap<&'static str, String>;

/// 템플릿과 명령에서 쓸 수 있는 자리 표시자 전체.
pub const PLACEHOLDERS: [&str; 7] = [
    "id",
    "title",
    "dir",
    "file",
    "build",
    "package",
    "main_class",
];

const BUILTIN_TEMPLATES: &[(&str, &str)] = &[
    ("java", include_str!("templates/java.java")),
    ("cpp", include_str!("templates/cpp.cpp")),
    ("py", include_str!("templates/py.py")),
    ("kt", include_str!("templates/kt.kt")),
];

/// `{{name}}`을 치환하고, 패키지가 비어 생긴 `package ;` 줄을 지운다.
pub fn render(src: &str, vars: &Vars) -> Result<String, BojError> {
    let out = substitute(src, vars)?;
    let empty_pkg = regex::Regex::new(r"^\s*package\s*;\s*$").expect("고정된 정규식");
    let kept: Vec<&str> = out
        .split_inclusive('\n')
        .filter(|l| !empty_pkg.is_match(l.trim_end_matches(['\n', '\r'])))
        .collect();
    Ok(kept.concat())
}

fn substitute(src: &str, vars: &Vars) -> Result<String, BojError> {
    let re = regex::Regex::new(r"\{\{\s*([A-Za-z_]+)\s*\}\}").expect("고정된 정규식");
    let mut err = None;
    let out = re.replace_all(src, |c: &regex::Captures| {
        let name = &c[1];
        if !PLACEHOLDERS.contains(&name) {
            err.get_or_insert(format!("알 수 없는 자리 표시자: {{{{{name}}}}}"));
            return String::new();
        }
        match vars.get(name) {
            Some(v) => v.clone(),
            None => {
                err.get_or_insert(format!("여기서는 {{{{{name}}}}}을 쓸 수 없어요"));
                String::new()
            }
        }
    });
    match err {
        Some(e) => Err(BojError::Template(e)),
        None => Ok(out.into_owned()),
    }
}

/// 명령 문자열을 먼저 토큰으로 나눈 뒤 토큰마다 치환한다 (경로에 공백이 있어도 안전).
pub fn render_command(cmd: &str, vars: &Vars) -> Result<Vec<String>, BojError> {
    let tokens = shell_words::split(cmd)
        .map_err(|e| BojError::Config(format!("명령을 읽을 수 없어요: {cmd:?} ({e})")))?;
    if tokens.is_empty() {
        return Err(BojError::Config("명령이 비어 있어요".into()));
    }
    tokens.iter().map(|t| substitute(t, vars)).collect()
}

/// 경로 안 `src/main/java/` 뒤의 구성 요소를 `.`으로 이은 Java 패키지 이름.
pub fn java_package(rel_dir: &Path) -> Option<String> {
    let parts: Vec<String> = rel_dir
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let at = parts
        .windows(3)
        .position(|w| w[0] == "src" && w[1] == "main" && w[2] == "java")?;
    let rest = &parts[at + 3..];
    if rest.is_empty() {
        None
    } else {
        Some(rest.join("."))
    }
}

pub fn main_class(package: Option<&str>) -> String {
    match package {
        Some(p) if !p.is_empty() => format!("{p}.Main"),
        _ => "Main".to_string(),
    }
}

/// 숫자로 시작하는 구성 요소가 있으면 Java에서 쓸 수 없는 패키지라 경고한다.
pub fn package_warning(pkg: &str) -> Option<String> {
    pkg.split('.')
        .find(|seg| seg.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(|seg| {
            format!(
                "Java 패키지 이름 `{pkg}`의 `{seg}`가 숫자로 시작해서 컴파일되지 않아요. path를 \"src/main/java/p{{{{id}}}}\"처럼 바꿔 주세요"
            )
        })
}

/// 프로젝트 `.boj/templates/` → 전역 `templates/` → 내장 순으로 찾는다.
pub fn load_template(
    name: &str,
    variant: Option<&str>,
    project_root: Option<&Path>,
    global_dir: Option<&Path>,
) -> Result<String, BojError> {
    let stem = match variant {
        Some(v) => format!("{name}-{v}"),
        None => name.to_string(),
    };
    let dirs = [
        project_root.map(|p| p.join(".boj").join("templates")),
        global_dir.map(|g| g.join("templates")),
    ];
    for dir in dirs.into_iter().flatten() {
        if let Some(path) = find_by_stem(&dir, &stem) {
            return Ok(std::fs::read_to_string(path)?);
        }
    }
    BUILTIN_TEMPLATES
        .iter()
        .find(|(n, _)| *n == stem)
        .map(|(_, body)| body.to_string())
        .ok_or_else(|| BojError::Template(format!("템플릿 `{stem}`을 찾을 수 없어요")))
}

fn find_by_stem(dir: &Path, stem: &str) -> Option<std::path::PathBuf> {
    let mut hits: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.file_stem().is_some_and(|s| s == stem))
        .collect();
    hits.sort();
    hits.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn vars(pairs: &[(&'static str, &str)]) -> Vars {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn renders_vars() {
        let out = render(
            "// {{id}} {{ title }}\n",
            &vars(&[("id", "1000"), ("title", "A+B")]),
        )
        .unwrap();
        assert_eq!(out, "// 1000 A+B\n");
    }

    #[test]
    fn unknown_placeholder_errors() {
        let r = render("{{nope}}", &vars(&[]));
        assert!(matches!(r, Err(BojError::Template(m)) if m.contains("nope")));
    }

    #[test]
    fn known_but_missing_value_errors() {
        assert!(matches!(
            render("{{build}}", &vars(&[])),
            Err(BojError::Template(_))
        ));
    }

    #[test]
    fn empty_package_line_removed() {
        let out = render(
            "package {{package}};\n\nimport x;\n",
            &vars(&[("package", "")]),
        )
        .unwrap();
        assert_eq!(out, "\nimport x;\n");
        let kept = render("package {{package}};\n", &vars(&[("package", "p1000")])).unwrap();
        assert_eq!(kept, "package p1000;\n");
    }

    #[test]
    fn command_tokens_keep_spaces_in_paths() {
        let v = vars(&[("build", "/tmp/알고 리즘/b"), ("file", "Main.java")]);
        let toks = render_command("javac -d {{build}} {{file}}", &v).unwrap();
        assert_eq!(toks, vec!["javac", "-d", "/tmp/알고 리즘/b", "Main.java"]);
    }

    #[test]
    fn packages() {
        assert_eq!(
            java_package(Path::new("src/main/java/p1000")),
            Some("p1000".into())
        );
        assert_eq!(
            java_package(Path::new("src/main/java/boj/p1000")),
            Some("boj.p1000".into())
        );
        assert_eq!(java_package(Path::new("1000")), None);
        assert_eq!(java_package(Path::new("src/main/java")), None);
        assert_eq!(main_class(Some("p1000")), "p1000.Main");
        assert_eq!(main_class(None), "Main");
        assert!(package_warning("1000").is_some());
        assert!(package_warning("boj.1000").is_some());
        assert!(package_warning("p1000").is_none());
    }

    #[test]
    fn every_builtin_template_renders() {
        let v = vars(&[
            ("id", "1000"),
            ("title", "A+B"),
            ("dir", "/x"),
            ("file", "f"),
            ("package", ""),
            ("main_class", "Main"),
        ]);
        for name in ["java", "cpp", "py", "kt"] {
            let src = load_template(name, None, None, None).unwrap();
            let out = render(&src, &v).unwrap();
            assert!(out.contains("1000 A+B"), "{name}: {out}");
            assert!(!out.contains("{{"), "{name}");
        }
    }

    #[test]
    fn builtin_java_template() {
        let t = load_template("java", None, None, None).unwrap();
        assert!(t.contains("static BufferedReader br"));
        assert!(t.contains("package {{package}};"));
    }

    #[test]
    fn project_template_beats_global_and_builtin() {
        let p = tempfile::tempdir().unwrap();
        let g = tempfile::tempdir().unwrap();
        fs::create_dir_all(p.path().join(".boj/templates")).unwrap();
        fs::create_dir_all(g.path().join("templates")).unwrap();
        fs::write(p.path().join(".boj/templates/java.java"), "PROJECT").unwrap();
        fs::write(g.path().join("templates/java.java"), "GLOBAL").unwrap();
        assert_eq!(
            load_template("java", None, Some(p.path()), Some(g.path())).unwrap(),
            "PROJECT"
        );
        assert_eq!(
            load_template("java", None, None, Some(g.path())).unwrap(),
            "GLOBAL"
        );
    }

    #[test]
    fn variant_lookup() {
        let g = tempfile::tempdir().unwrap();
        fs::create_dir_all(g.path().join("templates")).unwrap();
        fs::write(g.path().join("templates/java-scanner.java"), "SCANNER").unwrap();
        assert_eq!(
            load_template("java", Some("scanner"), None, Some(g.path())).unwrap(),
            "SCANNER"
        );
        let missing = load_template("java", Some("nope"), None, Some(g.path()));
        assert!(matches!(missing, Err(BojError::Template(_))));
    }
}
