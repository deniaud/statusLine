use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ccline")]
#[command(version, about = "High-performance Claude Code StatusLine")]
pub struct Cli {
    /// Enter TUI configuration mode
    #[arg(short = 'c', long = "config")]
    pub config: bool,

    /// Set theme
    #[arg(short = 't', long = "theme")]
    pub theme: Option<String>,

    /// Patch Claude Code cli.js to disable context warnings
    #[arg(long = "patch")]
    pub patch: Option<String>,

    /// Install ccline as Claude Code statusline (copies binary + patches settings.json)
    #[arg(long = "install")]
    pub install: bool,

    /// Verify ccline integration health (binary, settings, transcripts)
    #[arg(long = "doctor")]
    pub doctor: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
