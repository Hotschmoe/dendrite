use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use dendrite::discovery::{discover_files, DiscoveryConfig, ParseError};
use dendrite::graph::{FileNode, GraphBuilder, Layer};
use dendrite::output::json::CodebaseMap;
use dendrite::parser::{parse_file, parse_rust_file};

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

    if cli.tui {
        eprintln!("TUI mode not yet implemented (Phase 5)");
        std::process::exit(1);
    }

    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    let start = Instant::now();

    // Task 1.6.1: Wire up --path
    if !cli.path.exists() {
        anyhow::bail!("Path does not exist: {}", cli.path.display());
    }

    let project_root = cli.path.canonicalize()?;

    if !cli.quiet {
        println!("Dendrite - Codebase Dependency Analysis");
        println!("========================================");
        println!();
        println!("Analyzing: {}", project_root.display());
        if cli.verbose {
            println!("Output directory: {}", cli.output.display());
            println!("JSON only: {}", cli.json);
            println!("Markdown only: {}", cli.markdown);
            println!("Generate all: {}", cli.all);
        }
        println!();
    }

    // 1. Discover files
    if cli.verbose {
        println!("Discovering source files...");
    }

    let config = DiscoveryConfig::default();
    let files = discover_files(&project_root, &config)?;

    if cli.verbose {
        println!("Found {} source files", files.len());
        for file in &files {
            println!("  - {}", file.display());
        }
        println!();
    }

    if files.is_empty() {
        if !cli.quiet {
            println!("No source files found in {}", project_root.display());
            println!("Looking for extensions: {:?}", config.extensions);
        }
        return Ok(());
    }

    // 2. Parse files based on extension
    if cli.verbose {
        println!("Parsing files...");
    }

    let mut zig_files = Vec::new();
    let mut rust_files = Vec::new();
    let mut parse_errors = Vec::<ParseError>::new();

    for file in &files {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "zig" => match parse_file(file) {
                Ok(parsed) => zig_files.push(parsed),
                Err(err) => {
                    if !cli.quiet {
                        eprintln!("Warning: failed to parse {}: {}", file.display(), err);
                    }
                    parse_errors.push(ParseError::new(file.clone(), err));
                }
            },
            "rs" => match parse_rust_file(file) {
                Ok(parsed) => rust_files.push(parsed),
                Err(err) => {
                    if !cli.quiet {
                        eprintln!("Warning: failed to parse {}: {}", file.display(), err);
                    }
                    parse_errors.push(ParseError::new(file.clone(), err));
                }
            },
            _ => {
                if cli.verbose {
                    println!("Skipping unknown extension: {}", file.display());
                }
            }
        }
    }

    if cli.verbose {
        println!("Parsed {} Zig files, {} Rust files", zig_files.len(), rust_files.len());
        if !parse_errors.is_empty() {
            println!("Encountered {} parse errors", parse_errors.len());
        }
        println!();
    }

    // 3. Build dependency graph
    if cli.verbose {
        println!("Building dependency graph...");
    }

    let mut builder = GraphBuilder::new(project_root.clone(), false);
    let mut import_map = HashMap::new();

    // Add Zig file nodes
    for zig_file in &zig_files {
        let relative_path = zig_file
            .path
            .strip_prefix(&project_root)
            .unwrap_or(&zig_file.path)
            .to_string_lossy()
            .to_string();

        let node = FileNode {
            path: zig_file.path.clone(),
            relative_path: relative_path.clone(),
            layer: Layer::from_path(&relative_path),
            depth: 0,
            summary: zig_file.doc_comment.clone(),
            exports: zig_file.exports.clone(),
            loc: zig_file.loc,
        };

        builder.add_file(node);

        // Collect imports for edge creation
        let imports: Vec<(String, usize)> = zig_file
            .imports
            .iter()
            .filter(|i| !i.is_std)
            .map(|i| (i.target.clone(), i.line))
            .collect();

        if !imports.is_empty() {
            import_map.insert(zig_file.path.clone(), imports);
        }
    }

    // Add Rust file nodes
    for rust_file in &rust_files {
        let relative_path = rust_file
            .path
            .strip_prefix(&project_root)
            .unwrap_or(&rust_file.path)
            .to_string_lossy()
            .to_string();

        let node = FileNode {
            path: rust_file.path.clone(),
            relative_path: relative_path.clone(),
            layer: Layer::from_path(&relative_path),
            depth: 0,
            summary: rust_file.doc_comment.clone(),
            exports: rust_file.exports.clone(),
            loc: rust_file.loc,
        };

        builder.add_file(node);

        // Collect mod file imports for edge creation
        let imports: Vec<(String, usize)> = rust_file
            .imports
            .iter()
            .filter(|i| i.kind == dendrite::parser::ImportKind::Mod)
            .map(|i| (format!("{}.rs", i.path), i.line))
            .collect();

        if !imports.is_empty() {
            import_map.insert(rust_file.path.clone(), imports);
        }
    }

    // Add edges
    builder.add_edges(&import_map);

    let graph = builder.build();

    if cli.verbose {
        println!(
            "Graph built: {} nodes, {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        println!();
    }

    // 4. Generate output
    // Task 1.6.3: Create output directory if needed
    if cli.json || cli.all {
        if !cli.output.exists() {
            if cli.verbose {
                println!("Creating output directory: {}", cli.output.display());
            }
            fs::create_dir_all(&cli.output)?;
        }
    }

    // Task 1.6.2: Output JSON
    if cli.json || cli.all {
        let json_path = cli.output.join("graph.json");
        let codemap = CodebaseMap::from_graph(&graph, &project_root.to_string_lossy());
        codemap.write_to_file(&json_path)?;

        if !cli.quiet {
            println!("JSON output written to: {}", json_path.display());
        }
    }

    // Markdown output (Phase 3 - not yet implemented)
    if cli.markdown || cli.all {
        if !cli.quiet {
            eprintln!("Markdown output not yet implemented (Phase 3)");
        }
    }

    // Task 1.6.6: Print summary stats
    let elapsed = start.elapsed();
    if !cli.quiet && !cli.json {
        println!();
        println!("Summary:");
        println!("  Files discovered: {}", files.len());
        println!("  Files parsed: {} Zig, {} Rust", zig_files.len(), rust_files.len());
        println!("  Parse errors: {}", parse_errors.len());
        println!("  Graph nodes: {}", graph.node_count());
        println!("  Graph edges: {}", graph.edge_count());
        println!("  Time elapsed: {:.2?}", elapsed);
    }

    Ok(())
}
