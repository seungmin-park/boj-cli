use clap::{Parser, Subcommand};

/// 문제 번호 하나로 백준 예제를 받고 바로 채점해요.
#[derive(Debug, Parser)]
#[command(name = "boj", version, args_conflicts_with_subcommands = true)]
pub struct Cli {
    /// 만들 문제 번호 (예: boj 2309)
    pub id: Option<u32>,

    /// 언어 (기본값은 설정의 default)
    #[arg(short, long)]
    pub lang: Option<String>,

    /// 같은 언어의 다른 템플릿 이름 (templates/<언어>-<이름>.*)
    #[arg(short, long)]
    pub template: Option<String>,

    /// 네트워크 없이 소스와 메타만 만들어요
    #[arg(long)]
    pub offline: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 컴파일하고 예제를 전부 채점해요 (번호를 생략하면 현재 폴더)
    Test {
        id: Option<u32>,
        /// 한 폴더에 여러 언어 소스가 있을 때 고를 언어
        #[arg(short, long)]
        lang: Option<String>,
    },
    /// 제출용 코드를 클립보드에 복사해요 (패키지 줄 등 제거)
    Copy {
        id: Option<u32>,
        /// 한 폴더에 여러 언어 소스가 있을 때 고를 언어
        #[arg(short, long)]
        lang: Option<String>,
        /// 클립보드 대신 표준 출력으로 내보내요
        #[arg(long)]
        stdout: bool,
    },
    /// 백준 문제 페이지를 열어요
    Open {
        id: Option<u32>,
        /// 브라우저를 열지 않고 URL만 출력해요
        #[arg(long)]
        print: bool,
    },
}
