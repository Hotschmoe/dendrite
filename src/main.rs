use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dendrite")]
#[command(about = "Codebase mapping and dependency analysis tool", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to analyze (default: current directory)
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Output JSON only
    #[arg(short, long)]
    json: bool,

    /// Output markdown only
    #[arg(short, long)]
    markdown: bool,

    /// Output directory (default: .dendrite/)
    #[arg(short, long, default_value = ".dendrite")]
    output: PathBuf,

    /// Suppress stdout output
    #[arg(short, long)]
    quiet: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// CI mode - exit 1 on cycles (for Phase 2)
    #[arg(long)]
    check: bool,

    /// Also exit 1 on violations (for Phase 2)
    #[arg(long)]
    strict: bool,

    /// Launch interactive TUI (for Phase 5)
    #[arg(short, long)]
    tui: bool,

    /// Generate all outputs
    #[arg(short, long)]
    all: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.quiet {
        println!("Dendrite - Codebase Dependency Analysis");
        println!("========================================");
        println!();
        println!("Configuration:");
        println!("  Path to analyze: {}", cli.path.display());
        println!("  Output directory: {}", cli.output.display());
        println!();
        println!("Output options:");
        println!("  JSON only: {}", cli.json);
        println!("  Markdown only: {}", cli.markdown);
        println!("  Generate all: {}", cli.all);
        println!();
        println!("Mode options:");
        println!("  Verbose: {}", cli.verbose);
        println!("  Quiet: {}", cli.quiet);
        println!("  Check (CI): {}", cli.check);
        println!("  Strict: {}", cli.strict);
        println!("  TUI: {}", cli.tui);
        println!();
        println!("(Placeholder - parser and analysis not yet implemented)");
    }

    Ok(())
}
