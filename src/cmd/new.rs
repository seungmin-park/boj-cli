use std::path::PathBuf;

use crate::cmd::Ctx;
use crate::error::BojError;
use crate::fetch::Fetcher;
use crate::meta::{Meta, META_FILE};
use crate::template::{self, Vars};

#[derive(Debug, Default)]
pub struct NewReport {
    pub dir: PathBuf,
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

pub struct NewArgs<'a> {
    pub id: u32,
    pub lang: Option<&'a str>,
    pub variant: Option<&'a str>,
    pub offline: bool,
}

/// `boj <번호>`: 소스, 예제, 메타를 만든다. 있는 파일은 절대 덮어쓰지 않는다.
pub fn run_new(ctx: &Ctx, args: &NewArgs, fetcher: &dyn Fetcher) -> Result<NewReport, BojError> {
    let lang_name = args.lang.unwrap_or(&ctx.config.default);
    let lang = ctx.config.lang(lang_name)?;
    let dir = ctx.problem_dir(args.id)?;
    let tests_dir = ctx.tests_dir(&dir, args.id)?;
    let existing = Meta::read(&dir)?;

    // 파일을 만들기 전에 받아 둔다. 없는 문제면 아무것도 남기지 않는다.
    let need_fetch = !args.offline
        && match &existing {
            None => true,
            Some(m) => (1..=m.samples).any(|n| {
                !tests_dir.join(format!("{n}.in")).exists()
                    || !tests_dir.join(format!("{n}.out")).exists()
            }),
        };
    let problem = if need_fetch {
        Some(fetcher.fetch(args.id)?)
    } else {
        None
    };

    let mut report = NewReport {
        dir: dir.clone(),
        ..Default::default()
    };
    std::fs::create_dir_all(&dir)?;

    let meta = match existing {
        Some(m) => m,
        None => {
            let m = match &problem {
                Some(p) => Meta {
                    id: p.id,
                    title: p.title.clone(),
                    time_limit_ms: p.time_limit_ms,
                    special_judge: p.special_judge,
                    samples: p.samples.len(),
                    lang: lang_name.to_string(),
                },
                None => Meta {
                    id: args.id,
                    title: String::new(),
                    time_limit_ms: None,
                    special_judge: false,
                    samples: 0,
                    lang: lang_name.to_string(),
                },
            };
            m.write(&dir)?;
            report.created.push(dir.join(META_FILE));
            m
        }
    };

    let rel = dir.strip_prefix(&ctx.root).unwrap_or(&dir);
    let package = template::java_package(rel);
    if let Some(w) = package.as_deref().and_then(template::package_warning) {
        report.warnings.push(w);
    }
    let src_path = dir.join(&lang.file);
    if src_path.exists() {
        report.skipped.push(src_path);
    } else {
        let body = template::load_template(
            &lang.template,
            args.variant,
            ctx.project_root.as_deref(),
            ctx.global_dir.as_deref(),
        )?;
        let vars: Vars = [
            ("id", args.id.to_string()),
            ("title", meta.title.clone()),
            ("dir", dir.to_string_lossy().into_owned()),
            ("file", lang.file.clone()),
            ("package", package.clone().unwrap_or_default()),
            ("main_class", template::main_class(package.as_deref())),
        ]
        .into_iter()
        .collect();
        std::fs::write(&src_path, template::render(&body, &vars)?)?;
        report.created.push(src_path);
    }

    if let Some(p) = &problem {
        if p.samples.is_empty() {
            report
                .warnings
                .push("예제가 없는 문제예요. tests 폴더에 직접 N.in / N.out을 넣어 주세요".into());
        }
        std::fs::create_dir_all(&tests_dir)?;
        for (i, sample) in p.samples.iter().enumerate() {
            let n = i + 1;
            for (ext, body) in [("in", &sample.input), ("out", &sample.output)] {
                let path = tests_dir.join(format!("{n}.{ext}"));
                if path.exists() {
                    report.skipped.push(path);
                } else {
                    std::fs::write(&path, body)?;
                    report.created.push(path);
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::FixtureFetcher;
    use std::fs;
    use std::path::Path;

    fn fixtures() -> FixtureFetcher {
        FixtureFetcher {
            dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        }
    }

    fn ctx(cwd: &Path) -> Ctx {
        Ctx::new(cwd, None).unwrap()
    }

    fn args(id: u32) -> NewArgs<'static> {
        NewArgs {
            id,
            lang: None,
            variant: None,
            offline: false,
        }
    }

    #[test]
    fn creates_source_tests_meta() {
        let t = tempfile::tempdir().unwrap();
        let r = run_new(&ctx(t.path()), &args(1260), &fixtures()).unwrap();
        let dir = t.path().join("1260");
        assert_eq!(r.dir, dir);
        let src = fs::read_to_string(dir.join("Main.java")).unwrap();
        assert!(src.contains("// 1260 DFS와 BFS"));
        assert!(!src.contains("package"));
        for n in 1..=3 {
            assert!(dir.join(format!("tests/{n}.in")).is_file());
            assert!(dir.join(format!("tests/{n}.out")).is_file());
        }
        assert_eq!(
            fs::read_to_string(dir.join("tests/1.in"))
                .unwrap()
                .lines()
                .next(),
            Some("4 5 1")
        );
        let meta = Meta::read(&dir).unwrap().unwrap();
        assert_eq!(
            (meta.samples, meta.time_limit_ms, meta.lang.as_str()),
            (3, Some(2000), "java")
        );
        assert_eq!(r.created.len(), 1 + 6 + 1);
    }

    #[test]
    fn never_overwrites() {
        let t = tempfile::tempdir().unwrap();
        run_new(&ctx(t.path()), &args(1260), &fixtures()).unwrap();
        let main = t.path().join("1260/Main.java");
        fs::write(&main, "MY CODE").unwrap();
        fs::write(t.path().join("1260/tests/1.out"), "MINE").unwrap();
        let r = run_new(&ctx(t.path()), &args(1260), &fixtures()).unwrap();
        assert_eq!(fs::read_to_string(&main).unwrap(), "MY CODE");
        assert_eq!(
            fs::read_to_string(t.path().join("1260/tests/1.out")).unwrap(),
            "MINE"
        );
        assert!(r.created.is_empty());
        assert!(r.skipped.contains(&main));
    }

    #[test]
    fn refills_missing_test_only() {
        let t = tempfile::tempdir().unwrap();
        run_new(&ctx(t.path()), &args(1260), &fixtures()).unwrap();
        let missing = t.path().join("1260/tests/2.in");
        fs::remove_file(&missing).unwrap();
        let r = run_new(&ctx(t.path()), &args(1260), &fixtures()).unwrap();
        assert_eq!(r.created, vec![missing.clone()]);
        assert!(missing.is_file());
    }

    #[test]
    fn unknown_problem_leaves_nothing() {
        let t = tempfile::tempdir().unwrap();
        let r = run_new(&ctx(t.path()), &args(424242), &fixtures());
        assert!(matches!(r, Err(BojError::ProblemNotFound(424242))));
        assert!(!t.path().join("424242").exists());
    }

    #[test]
    fn java_package_under_src_main_java() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join(".boj")).unwrap();
        fs::write(
            t.path().join(".boj/config.toml"),
            "path = \"src/main/java/p{{id}}\"\n",
        )
        .unwrap();
        let sub = t.path().join("src");
        fs::create_dir_all(&sub).unwrap();
        let r = run_new(&ctx(&sub), &args(1000), &fixtures()).unwrap();
        let src = fs::read_to_string(t.path().join("src/main/java/p1000/Main.java")).unwrap();
        assert!(src.starts_with("package p1000;\n"));
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn numeric_package_warns() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join(".boj")).unwrap();
        fs::write(
            t.path().join(".boj/config.toml"),
            "path = \"src/main/java/{{id}}\"\n",
        )
        .unwrap();
        let r = run_new(&ctx(t.path()), &args(1000), &fixtures()).unwrap();
        assert_eq!(r.warnings.len(), 1);
    }

    #[test]
    fn offline_creates_source_only() {
        let t = tempfile::tempdir().unwrap();
        let a = NewArgs {
            offline: true,
            ..args(1000)
        };
        // 오프라인이면 fetcher를 쓰지 않으므로 없는 fixture 폴더를 줘도 된다.
        let none = FixtureFetcher {
            dir: t.path().join("nowhere"),
        };
        run_new(&ctx(t.path()), &a, &none).unwrap();
        let dir = t.path().join("1000");
        assert!(dir.join("Main.java").is_file());
        assert!(!dir.join("tests/1.in").exists());
        let meta = Meta::read(&dir).unwrap().unwrap();
        assert_eq!((meta.time_limit_ms, meta.samples), (None, 0));
    }

    #[test]
    fn no_samples_warns() {
        let t = tempfile::tempdir().unwrap();
        let fx = tempfile::tempdir().unwrap();
        fs::copy(
            fixtures().dir.join("nosample.html"),
            fx.path().join("1000.html"),
        )
        .unwrap();
        let r = run_new(
            &ctx(t.path()),
            &args(1000),
            &FixtureFetcher {
                dir: fx.path().into(),
            },
        )
        .unwrap();
        assert_eq!(r.warnings.len(), 1);
        assert!(t.path().join("1000/Main.java").is_file());
    }

    #[test]
    fn empty_output_sample_is_empty_file() {
        let t = tempfile::tempdir().unwrap();
        let fx = tempfile::tempdir().unwrap();
        fs::copy(
            fixtures().dir.join("emptyout.html"),
            fx.path().join("1000.html"),
        )
        .unwrap();
        run_new(
            &ctx(t.path()),
            &args(1000),
            &FixtureFetcher {
                dir: fx.path().into(),
            },
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(t.path().join("1000/tests/1.out")).unwrap(),
            ""
        );
    }

    #[test]
    fn unknown_lang_errors() {
        let t = tempfile::tempdir().unwrap();
        let a = NewArgs {
            lang: Some("brainf"),
            ..args(1000)
        };
        assert!(matches!(
            run_new(&ctx(t.path()), &a, &fixtures()),
            Err(BojError::Config(_))
        ));
    }
}
