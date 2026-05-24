use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use veritool_core::design::Design;

#[derive(Parser)]
#[command(name = "veritool", about = "SystemVerilog/Verilog static analysis tool")]
pub struct Cli {
    /// File list (-f filelist.f format)
    #[arg(short = 'f', long = "filelist", global = true)]
    pub filelist: Option<PathBuf>,

    /// Include directory (multiple allowed)
    #[arg(short = 'I', long = "incdir", global = true)]
    pub include_dirs: Vec<PathBuf>,

    /// Macro definition (e.g. -D NAME=VAL, multiple allowed)
    #[arg(short = 'D', global = true)]
    pub defines: Vec<String>,

    /// Parameter override (e.g. -P WIDTH=8, multiple allowed)
    #[arg(short = 'P', global = true)]
    pub param_overrides: Vec<String>,

    /// Output format
    #[arg(long, default_value = "text", global = true)]
    pub format: OutputFormat,

    /// Top module name
    #[arg(long, global = true)]
    pub top: Option<String>,

    /// Target module name (for ports/signals)
    #[arg(short = 'm', long = "module", global = true)]
    pub module_name: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show module port list
    Ports {
        /// SystemVerilog source files
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
    /// Show internal signal declarations
    Signals {
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
    /// Estimate flip-flop count
    Ff {
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
    /// Show module hierarchy
    Hier {
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
    /// Detect top-level modules
    Top {
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
    /// Generate full report
    Report {
        #[arg(name = "FILES")]
        files: Vec<PathBuf>,
    },
}

impl Commands {
    pub fn files(&self) -> &[PathBuf] {
        match self {
            Commands::Ports { files }
            | Commands::Signals { files }
            | Commands::Ff { files }
            | Commands::Hier { files }
            | Commands::Top { files }
            | Commands::Report { files } => files,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
    Csv,
}

/// Parse -D NAME or -D NAME=VAL strings into (name, Option<value>) pairs
pub fn parse_defines(defines: &[String]) -> Vec<(String, Option<String>)> {
    defines
        .iter()
        .map(|d| {
            if let Some(eq) = d.find('=') {
                (d[..eq].to_string(), Some(d[eq + 1..].to_string()))
            } else {
                (d.clone(), None)
            }
        })
        .collect()
}

/// Detect top-level modules (not instantiated by any other module in the design)
pub fn detect_top_module(design: &Design) -> Vec<String> {
    use std::collections::HashSet;
    let referenced: HashSet<&str> = design
        .modules
        .values()
        .flat_map(|m| m.instances.iter().map(|i| i.module_ref.as_str()))
        .collect();

    design
        .modules
        .keys()
        .filter(|name| !referenced.contains(name.as_str()))
        .cloned()
        .collect()
}
