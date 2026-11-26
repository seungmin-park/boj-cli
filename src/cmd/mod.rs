pub mod new;

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
