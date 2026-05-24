mod args;
mod format;

use anyhow::Result;
use clap::Parser;

use args::{Cli, Commands, OutputFormat, detect_top_module, parse_defines};
use veritool_core::design::Design;
use veritool_core::loader;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Loading files...");
    }

    let design = load_design(&cli)?;

    match &cli.command {
        Commands::Ports { .. } => {
            if cli.verbose {
                eprintln!("Extracting ports...");
            }
            run_ports(&design, &cli)
        }
        Commands::Signals { .. } => {
            if cli.verbose {
                eprintln!("Extracting signals...");
            }
            run_signals(&design, &cli)
        }
        Commands::Ff { .. } => {
            if cli.verbose {
                eprintln!("Counting FFs...");
            }
            run_ff(&design, &cli)
        }
        Commands::Hier { .. } => {
            if cli.verbose {
                eprintln!("Building hierarchy...");
            }
            run_hier(&design, &cli)
        }
        Commands::Top { .. } => {
            if cli.verbose {
                eprintln!("Detecting top modules...");
            }
            run_top(&design, &cli)
        }
        Commands::Report { .. } => {
            if cli.verbose {
                eprintln!("Generating report...");
            }
            run_report(&design, &cli)
        }
    }
}

fn load_design(cli: &Cli) -> Result<Design> {
    let defines = parse_defines(&cli.defines);

    if let Some(filelist) = &cli.filelist {
        let list = loader::parse_filelist(filelist)?;
        let mut all_defines = defines;
        all_defines.extend(list.defines);
        let mut all_dirs = cli.include_dirs.clone();
        all_dirs.extend(list.include_dirs);
        loader::parse_sv_files(&list.files, &all_dirs, &all_defines)
    } else {
        let files = cli.command.files().to_vec();
        if files.is_empty() {
            anyhow::bail!("No files specified. Use FILES... or -f <filelist>");
        }
        loader::parse_sv_files(&files, &cli.include_dirs, &defines)
    }
}

fn get_top_module(design: &Design, cli: &Cli) -> Result<String> {
    if let Some(top) = &cli.top {
        Ok(top.clone())
    } else {
        let tops = detect_top_module(design);
        tops.into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No top module detected. Use --top to specify one."))
    }
}

fn run_ports(design: &Design, cli: &Cli) -> Result<()> {
    let fmt = cli.format;
    if let Some(module_name) = &cli.module_name {
        let m = design
            .modules
            .get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", module_name))?;
        format::print_ports(m, fmt)
    } else {
        for m in design.modules.values() {
            format::print_ports(m, fmt)?;
            if fmt == OutputFormat::Text {
                println!();
            }
        }
        Ok(())
    }
}

fn run_signals(design: &Design, cli: &Cli) -> Result<()> {
    let fmt = cli.format;
    if let Some(module_name) = &cli.module_name {
        let m = design
            .modules
            .get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", module_name))?;
        format::print_signals(m, fmt)
    } else {
        for m in design.modules.values() {
            format::print_signals(m, fmt)?;
            if fmt == OutputFormat::Text {
                println!();
            }
        }
        Ok(())
    }
}

fn run_ff(design: &Design, cli: &Cli) -> Result<()> {
    let fmt = cli.format;
    let overrides = parse_param_overrides(&cli.param_overrides);
    if let Some(module_name) = &cli.module_name {
        let m = design
            .modules
            .get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module '{}' not found", module_name))?;
        format::print_ff_module(design, m, fmt, &overrides)
    } else {
        let top = get_top_module(design, cli)?;
        format::print_ff_hierarchy(design, &top, fmt, &overrides)
    }
}

/// Parse `-P NAME=VAL` strings into `(name, value)` pairs.
/// Ignores entries that are not valid integers (emits a warning).
fn parse_param_overrides(args: &[String]) -> Vec<(String, i64)> {
    args.iter()
        .filter_map(|s| {
            let s = s.trim();
            if let Some(eq) = s.find('=') {
                let name = s[..eq].trim().to_string();
                let val_str = s[eq + 1..].trim();
                match val_str.parse::<i64>() {
                    Ok(v) => Some((name, v)),
                    Err(_) => {
                        eprintln!("Warning: -P {}: '{}' is not an integer, ignoring", name, val_str);
                        None
                    }
                }
            } else {
                eprintln!("Warning: -P '{}': expected NAME=VAL format, ignoring", s);
                None
            }
        })
        .collect()
}

fn run_hier(design: &Design, cli: &Cli) -> Result<()> {
    let top = get_top_module(design, cli)?;
    format::print_hier(design, &top, cli.format)
}

fn run_top(design: &Design, cli: &Cli) -> Result<()> {
    let tops = detect_top_module(design);
    format::print_top(&tops, cli.format)
}

fn run_report(design: &Design, cli: &Cli) -> Result<()> {
    let top = get_top_module(design, cli)?;
    format::print_report(design, &top, cli.format)
}
