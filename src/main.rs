use clap::Parser;

/// Deterministic structural checks over markdown documents.
#[derive(Parser, Debug)]
#[command(name = "mdcheck", version, about)]
struct Cli {
    /// Rule file (YAML)
    rules: String,

    /// Markdown files to check; `-` reads stdin
    paths: Vec<String>,

    /// Output format
    #[arg(long, default_value = "pretty")]
    format: String,

    /// Base directory for link resolution
    #[arg(long)]
    root: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    println!("rules: {}", cli.rules);
    println!("paths: {:?}", cli.paths);
    println!("format: {}", cli.format);
    println!("root: {:?}", cli.root);
    Ok(())
}
