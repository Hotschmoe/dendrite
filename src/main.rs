use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use dendrite::discovery::{discover_files, DiscoveryConfig, ParseError};
use dendrite::graph::analysis::{analyze, AnalysisResult};
use dendrite::graph::{FileNode, GraphBuilder, Layer};
use dendrite::output::json::CodebaseMap;
use dendrite::output::markdown::{generate_codebase_md, generate_file_summaries, generate_metrics_json};
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

    /// Generate per-file summaries in summaries/ directory
    #[arg(short, long)]
    summaries: bool,

    /// Output directory (default: .dendrite/)
    #[arg(short, long, default_value = ".dendrite")]
    output: PathBuf,

    /// Clean output directory before generating
    #[arg(long)]
    clean: bool,

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

/// Compute relative path from project root.
fn relative_path(path: &std::path::Path, root: &std::path::Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
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
        let rel_path = relative_path(&zig_file.path, &project_root);
        let node = FileNode {
            path: zig_file.path.clone(),
            relative_path: rel_path.clone(),
            layer: Layer::from_path(&rel_path),
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
        let rel_path = relative_path(&rust_file.path, &project_root);
        let node = FileNode {
            path: rust_file.path.clone(),
            relative_path: rel_path.clone(),
            layer: Layer::from_path(&rel_path),
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

    // 4. Run analysis (needed for markdown and CI mode)
    let analysis = analyze(&graph, 5); // threshold for high fan-in/out

    // 5. Generate outputs
    let wants_json = cli.json || cli.all;
    let wants_markdown = cli.markdown || cli.all;
    let wants_summaries = cli.summaries || cli.all;
    let wants_output = wants_json || wants_markdown || wants_summaries;

    if wants_output {
        // Clean output directory if requested
        if cli.clean && cli.output.exists() {
            if cli.verbose {
                println!("Cleaning output directory: {}", cli.output.display());
            }
            fs::remove_dir_all(&cli.output)?;
        }

        if !cli.output.exists() {
            if cli.verbose {
                println!("Creating output directory: {}", cli.output.display());
            }
            fs::create_dir_all(&cli.output)?;
        }

        let codemap = CodebaseMap::from_graph(&graph, &project_root.to_string_lossy());
        let generated_at = &codemap.generated_at;

        if wants_json {
            let json_path = cli.output.join("graph.json");
            codemap.write_to_file(&json_path)?;
            if !cli.quiet {
                println!("JSON output written to: {}", json_path.display());
            }
        }

        if wants_markdown {
            generate_codebase_md(&graph, &analysis, &codemap, &cli.output)?;
            if !cli.quiet {
                println!("Markdown output written to: {}", cli.output.join("CODEBASE.md").display());
            }
        }

        if wants_summaries {
            let count = generate_file_summaries(&graph, &analysis, &cli.output, generated_at)?;
            if !cli.quiet {
                println!("File summaries written to: {} ({} files)", cli.output.join("summaries").display(), count);
            }
        }

        // Always generate metrics.json with any output
        generate_metrics_json(&graph, &analysis, &cli.output, generated_at)?;
        if !cli.quiet && cli.verbose {
            println!("Metrics written to: {}", cli.output.join("metrics.json").display());
        }
    }

    // CI mode: --check and --strict
    if cli.check || cli.strict {
        let has_cycle_errors = analysis.has_errors();
        let has_violations = analysis.has_warnings();

        // Print actionable error messages in file:line format
        if !cli.quiet {
            for cycle in &analysis.cycles {
                for (from, to, line) in &cycle.edges {
                    eprintln!("error: cycle: {}:{}: imports {} (forms cycle)", from, line, to);
                }
            }

            if cli.strict {
                for violation in &analysis.violations {
                    eprintln!(
                        "warning: violation: {}:{}: {} layer imports from {} layer ({})",
                        violation.file,
                        violation.line,
                        violation.from_layer,
                        violation.to_layer,
                        violation.imports
                    );
                }
            }

            // Print fix suggestions
            let suggestions = analysis.fix_suggestions();
            if !suggestions.is_empty() {
                eprintln!();
                eprintln!("Fix suggestions:");
                for suggestion in &suggestions {
                    eprintln!("  - {}", suggestion);
                }
            }
        }

        // Output analysis as JSON if requested
        if cli.check && cli.json {
            let analysis_json = analysis_to_json(&analysis);
            println!("{}", analysis_json);
        }

        // Exit with error if issues found
        if has_cycle_errors {
            if !cli.quiet {
                eprintln!();
                eprintln!("Found {} cycle(s). Fix cycles to ensure clean architecture.", analysis.cycles.len());
            }
            std::process::exit(1);
        }

        if cli.strict && has_violations {
            if !cli.quiet {
                eprintln!();
                eprintln!("Found {} layer violation(s). Fix violations or use --check (not --strict) to allow.", analysis.violations.len());
            }
            std::process::exit(1);
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
        if cli.check || cli.strict {
            println!("  Cycles detected: {}", analysis.cycles.len());
            println!("  Layer violations: {}", analysis.violations.len());
        }
        println!("  Time elapsed: {:.2?}", elapsed);
    }

    Ok(())
}

/// Convert analysis result to JSON for --check --json combo.
fn analysis_to_json(analysis: &AnalysisResult) -> String {
    use serde_json::json;

    let cycles_json: Vec<_> = analysis
        .cycles
        .iter()
        .map(|c| {
            json!({
                "nodes": c.nodes,
                "edges": c.edges.iter().map(|(from, to, line)| {
                    json!({
                        "from": from,
                        "to": to,
                        "line": line
                    })
                }).collect::<Vec<_>>()
            })
        })
        .collect();

    let violations_json: Vec<_> = analysis
        .violations
        .iter()
        .map(|v| {
            json!({
                "file": v.file,
                "imports": v.imports,
                "from_layer": v.from_layer.to_string(),
                "to_layer": v.to_layer.to_string(),
                "line": v.line,
                "reason": v.reason
            })
        })
        .collect();

    let result = json!({
        "has_errors": analysis.has_errors(),
        "has_warnings": analysis.has_warnings(),
        "cycles": cycles_json,
        "violations": violations_json,
        "metrics": {
            "max_depth": analysis.max_depth,
            "entry_points": analysis.entry_points,
            "high_fan_out": analysis.high_fan_out,
            "high_fan_in": analysis.high_fan_in,
            "orphans": analysis.orphans
        }
    });

    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
}
