pub mod copy;
pub mod new;
pub mod open;
pub mod test;

use std::path::{Path, PathBuf};

use crate::config::{self, Config};
use crate::error::BojError;
use crate::meta;
use crate::template::{self, Vars};

/// 명령들이 공유하는 실행 환경: 현재 폴더, 프로젝트 루트, 병합된 설정.
pub struct Ctx {
    pub cwd: PathBuf,
    /// `.boj/`가 있는 폴더. 없으면 현재 폴더를 루트로 쓴다.
    pub root: PathBuf,
    pub project_root: Option<PathBuf>,
    pub global_dir: Option<PathBuf>,
    pub config: Config,
}

impl Ctx {
    pub fn new(cwd: &Path, global_dir: Option<PathBuf>) -> Result<Ctx, BojError> {
        let project_root = config::find_project_root(cwd);
        let config = config::load(project_root.as_deref(), global_dir.as_deref())?;
        Ok(Ctx {
            cwd: cwd.to_path_buf(),
            root: project_root.clone().unwrap_or_else(|| cwd.to_path_buf()),
            project_root,
            global_dir,
            config,
        })
    }

    /// 설정의 `path`로 문제 폴더 경로를 만든다.
    pub fn problem_dir(&self, id: u32) -> Result<PathBuf, BojError> {
        let vars: Vars = [("id", id.to_string())].into_iter().collect();
        let rel = template::render(&self.config.path, &vars)?;
        Ok(self.root.join(rel.trim()))
    }

    /// 설정의 `tests`로 예제 폴더 경로를 만든다.
    pub fn tests_dir(&self, dir: &Path, id: u32) -> Result<PathBuf, BojError> {
        let vars: Vars = [
            ("id", id.to_string()),
            ("dir", dir.to_string_lossy().into_owned()),
        ]
        .into_iter()
        .collect();
        let p = PathBuf::from(template::render(&self.config.tests, &vars)?.trim());
        Ok(if p.is_absolute() {
            p
        } else {
            self.root.join(p)
        })
    }

    /// 채점·복사할 언어. `-l` > 폴더에 소스가 하나뿐이면 그 언어 > 메타의 언어.
    /// 여러 언어 소스가 같이 있으면 `-l`을 요구한다.
    pub fn resolve_lang(
        &self,
        dir: &Path,
        meta: &meta::Meta,
        explicit: Option<&str>,
    ) -> Result<String, BojError> {
        if let Some(l) = explicit {
            self.config.lang(l)?;
            return Ok(l.to_string());
        }
        let present: Vec<(&String, &String)> = self
            .config
            .langs
            .iter()
            .filter(|(_, l)| dir.join(&l.file).is_file())
            .map(|(name, l)| (name, &l.file))
            .collect();
        match present.as_slice() {
            [] => Ok(meta.lang.clone()),
            [(name, _)] => Ok((*name).clone()),
            many => {
                let list: Vec<String> = many.iter().map(|(n, f)| format!("{f}({n})")).collect();
                Err(BojError::Usage(format!(
                    "여러 언어 소스가 있어요: {}. -l 로 골라 주세요 (예: boj test -l {})",
                    list.join(", "),
                    many[0].0
                )))
            }
        }
    }

    /// 번호가 있으면 설정 경로로, 없으면 현재 폴더에서 위로 찾는다.
    pub fn resolve_problem_dir(&self, id: Option<u32>) -> Result<PathBuf, BojError> {
        match id {
            Some(id) => {
                let dir = self.problem_dir(id)?;
                if dir.join(meta::META_FILE).is_file() {
                    Ok(dir)
                } else {
                    Err(BojError::NoProblemDir)
                }
            }
            None => meta::find_problem_dir(&self.cwd).ok_or(BojError::NoProblemDir),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::fs;

    fn meta(lang: &str) -> Meta {
        Meta {
            id: 1000,
            title: "A+B".into(),
            time_limit_ms: Some(2000),
            special_judge: false,
            samples: 1,
            lang: lang.into(),
        }
    }

    #[test]
    fn resolve_lang_rules() {
        let t = tempfile::tempdir().unwrap();
        let ctx = Ctx::new(t.path(), None).unwrap();
        let dir = t.path();

        // 소스가 없으면 메타의 언어
        assert_eq!(ctx.resolve_lang(dir, &meta("java"), None).unwrap(), "java");
        // 명시하면 그 언어, 모르는 언어는 설정 오류
        assert_eq!(
            ctx.resolve_lang(dir, &meta("java"), Some("py")).unwrap(),
            "py"
        );
        assert!(matches!(
            ctx.resolve_lang(dir, &meta("java"), Some("cobol")),
            Err(BojError::Config(_))
        ));
        // 소스가 하나뿐이면 그 언어 (메타와 달라도)
        fs::write(dir.join("main.cpp"), "").unwrap();
        assert_eq!(ctx.resolve_lang(dir, &meta("java"), None).unwrap(), "cpp");
        // 둘 이상이면 -l 요구
        fs::write(dir.join("Main.java"), "").unwrap();
        let err = ctx
            .resolve_lang(dir, &meta("java"), None)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("-l") && err.contains("Main.java") && err.contains("main.cpp"),
            "{err}"
        );
        assert_eq!(
            ctx.resolve_lang(dir, &meta("java"), Some("cpp")).unwrap(),
            "cpp"
        );
    }
}
