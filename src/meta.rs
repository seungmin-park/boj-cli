use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::BojError;

pub const META_FILE: &str = ".boj.toml";

/// 문제 폴더의 `.boj.toml`. 한 번 받은 문제 정보를 저장해 다시 요청하지 않게 한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub id: u32,
    pub title: String,
    pub time_limit_ms: Option<u64>,
    pub special_judge: bool,
    pub samples: usize,
    pub lang: String,
}

impl Meta {
    pub fn read(dir: &Path) -> Result<Option<Meta>, BojError> {
        let path = dir.join(META_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text)
            .map(Some)
            .map_err(|e| BojError::Config(format!("{}: {e}", path.display())))
    }

    pub fn write(&self, dir: &Path) -> Result<(), BojError> {
        let text = toml::to_string(self).map_err(|e| BojError::Config(e.to_string()))?;
        std::fs::write(dir.join(META_FILE), text)?;
        Ok(())
    }
}

/// 현재 위치에서 위로 올라가며 `.boj.toml`이 있는 문제 폴더를 찾는다.
pub fn find_problem_dir(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(META_FILE).is_file())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_find() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("1000");
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        let m = Meta {
            id: 1000,
            title: "A+B".into(),
            time_limit_ms: Some(2000),
            special_judge: false,
            samples: 1,
            lang: "java".into(),
        };
        m.write(&dir).unwrap();
        assert_eq!(Meta::read(&dir).unwrap(), Some(m));
        assert_eq!(find_problem_dir(&dir.join("tests")).unwrap(), dir);
        assert!(find_problem_dir(t.path()).is_none());
    }
}
