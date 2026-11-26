use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::BojError;

const BUILTIN: &str = include_str!("builtin.toml");

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub default: String,
    pub path: String,
    pub tests: String,
    pub langs: BTreeMap<String, LangConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LangConfig {
    pub file: String,
    pub template: String,
    pub compile: Option<String>,
    pub run: String,
    pub strip: Vec<String>,
    pub time: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    default: Option<String>,
    path: Option<String>,
    tests: Option<String>,
    #[serde(default)]
    lang: BTreeMap<String, RawLang>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLang {
    file: Option<String>,
    template: Option<String>,
    compile: Option<String>,
    run: Option<String>,
    strip: Option<Vec<String>>,
    time: Option<String>,
}

impl Config {
    pub fn lang(&self, name: &str) -> Result<&LangConfig, BojError> {
        self.langs
            .get(name)
            .ok_or_else(|| BojError::Config(format!("지원하지 않는 언어: {name}")))
    }
}

/// 현재 위치에서 위로 올라가며 `.boj/` 디렉터리가 있는 폴더를 찾는다.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(".boj").is_dir())
        .map(Path::to_path_buf)
}

/// 전역 설정 폴더. `BOJ_CONFIG_DIR` > `$XDG_CONFIG_HOME/boj` > `~/.config/boj` (Windows는 %APPDATA%\\boj)
pub fn global_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("BOJ_CONFIG_DIR") {
        return Some(d.into());
    }
    if cfg!(windows) {
        return dirs::config_dir().map(|d| d.join("boj"));
    }
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME").filter(|x| !x.is_empty()) {
        return Some(PathBuf::from(x).join("boj"));
    }
    dirs::home_dir().map(|h| h.join(".config").join("boj"))
}

/// 내장 → 전역 → 프로젝트 순으로 병합한다.
pub fn load(project_root: Option<&Path>, global_dir: Option<&Path>) -> Result<Config, BojError> {
    let mut raw = parse(BUILTIN, "내장 설정")?;
    if let Some(g) = global_dir {
        overlay_file(&mut raw, &g.join("config.toml"))?;
    }
    if let Some(p) = project_root {
        overlay_file(&mut raw, &p.join(".boj").join("config.toml"))?;
    }
    finish(raw)
}

fn parse(text: &str, origin: &str) -> Result<RawConfig, BojError> {
    toml::from_str(text).map_err(|e| BojError::Config(format!("{origin}: {e}")))
}

fn overlay_file(base: &mut RawConfig, path: &Path) -> Result<(), BojError> {
    if !path.is_file() {
        return Ok(());
    }
    let text = std::fs::read_to_string(path)?;
    let top = parse(&text, &path.display().to_string())?;
    overlay(base, top);
    Ok(())
}

fn overlay(base: &mut RawConfig, top: RawConfig) {
    if top.default.is_some() {
        base.default = top.default;
    }
    if top.path.is_some() {
        base.path = top.path;
    }
    if top.tests.is_some() {
        base.tests = top.tests;
    }
    for (name, t) in top.lang {
        let b = base.lang.entry(name).or_default();
        macro_rules! take {
            ($($f:ident),*) => { $( if t.$f.is_some() { b.$f = t.$f; } )* };
        }
        take!(file, template, compile, run, strip, time);
    }
}

fn finish(raw: RawConfig) -> Result<Config, BojError> {
    let need = |v: Option<String>, what: &str| {
        v.ok_or_else(|| BojError::Config(format!("{what} 값이 비어 있어요")))
    };
    let mut langs = BTreeMap::new();
    for (name, l) in raw.lang {
        let what = |f: &str| format!("[lang.{name}] {f}");
        langs.insert(
            name.clone(),
            LangConfig {
                file: need(l.file, &what("file"))?,
                template: need(l.template, &what("template"))?,
                compile: l.compile,
                run: need(l.run, &what("run"))?,
                strip: l.strip.unwrap_or_default(),
                time: l.time.unwrap_or_default(),
            },
        );
    }
    let config = Config {
        default: need(raw.default, "default")?,
        path: need(raw.path, "path")?,
        tests: need(raw.tests, "tests")?,
        langs,
    };
    config.lang(&config.default)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    #[test]
    fn builtin_only_has_java() {
        let c = load(None, None).unwrap();
        assert_eq!(c.default, "java");
        assert_eq!(c.path, "{{id}}");
        assert!(c
            .lang("java")
            .unwrap()
            .compile
            .as_ref()
            .unwrap()
            .contains("--release 11"));
    }

    #[test]
    fn global_overrides_one_field() {
        let g = tempfile::tempdir().unwrap();
        write(
            g.path(),
            "config.toml",
            "[lang.java]\ncompile = \"javac --release 8 -d {{build}} {{file}}\"\n",
        );
        let c = load(None, Some(g.path())).unwrap();
        let java = c.lang("java").unwrap();
        assert!(java.compile.as_ref().unwrap().contains("--release 8"));
        assert_eq!(java.run, "java -Xss256m -cp {{build}} {{main_class}}");
    }

    #[test]
    fn project_beats_global() {
        let g = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        write(g.path(), "config.toml", "path = \"global/{{id}}\"\n");
        write(
            p.path(),
            ".boj/config.toml",
            "path = \"src/main/java/p{{id}}\"\n",
        );
        let c = load(Some(p.path()), Some(g.path())).unwrap();
        assert_eq!(c.path, "src/main/java/p{{id}}");
    }

    #[test]
    fn user_adds_language() {
        let g = tempfile::tempdir().unwrap();
        write(g.path(), "config.toml", "[lang.rust]\nfile = \"main.rs\"\ntemplate = \"rust\"\ncompile = \"rustc -O -o {{build}}/a {{file}}\"\nrun = \"{{build}}/a\"\n");
        let c = load(None, Some(g.path())).unwrap();
        let r = c.lang("rust").unwrap();
        assert_eq!(r.time, "");
        assert!(r.strip.is_empty());
    }

    #[test]
    fn missing_required_field_errors() {
        let g = tempfile::tempdir().unwrap();
        write(g.path(), "config.toml", "[lang.go]\nfile = \"main.go\"\n");
        assert!(matches!(
            load(None, Some(g.path())),
            Err(BojError::Config(_))
        ));
    }

    #[test]
    fn unknown_key_errors() {
        let g = tempfile::tempdir().unwrap();
        write(g.path(), "config.toml", "pth = \"x\"\n");
        assert!(matches!(
            load(None, Some(g.path())),
            Err(BojError::Config(_))
        ));
    }

    #[test]
    fn unknown_default_lang_errors() {
        let g = tempfile::tempdir().unwrap();
        write(g.path(), "config.toml", "default = \"cobol\"\n");
        assert!(matches!(
            load(None, Some(g.path())),
            Err(BojError::Config(_))
        ));
    }

    #[test]
    fn find_root_walks_up() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join("a/.boj")).unwrap();
        fs::create_dir_all(t.path().join("a/b/c")).unwrap();
        let root = find_project_root(&t.path().join("a/b/c")).unwrap();
        assert_eq!(
            root.canonicalize().unwrap(),
            t.path().join("a").canonicalize().unwrap()
        );
        assert!(find_project_root(t.path()).is_none());
    }
}
