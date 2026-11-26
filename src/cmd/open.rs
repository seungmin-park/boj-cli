use crate::cmd::Ctx;
use crate::error::BojError;
use crate::fetch::problem_url;
use crate::meta::Meta;

pub enum Opened {
    Browser(String),
    Printed(String),
}

/// `boj open`: 번호가 없으면 현재 문제 폴더의 메타에서 읽는다.
pub fn run_open(ctx: &Ctx, id: Option<u32>, print_only: bool) -> Result<Opened, BojError> {
    let id = match id {
        Some(id) => id,
        None => {
            let dir = ctx.resolve_problem_dir(None)?;
            Meta::read(&dir)?.ok_or(BojError::NoProblemDir)?.id
        }
    };
    let url = problem_url(id);
    if print_only {
        return Ok(Opened::Printed(url));
    }
    match open::that(&url) {
        Ok(()) => Ok(Opened::Browser(url)),
        Err(_) => Ok(Opened::Printed(url)),
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch::problem_url;

    #[test]
    fn url() {
        assert_eq!(problem_url(2309), "https://www.acmicpc.net/problem/2309");
    }
}
