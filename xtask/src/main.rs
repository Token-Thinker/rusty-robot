use std::path::{Path, PathBuf};
use anyhow::Result;
use clap::{Args, Parser};
use xtask::{Platform};


// ----------------------------------------------------------------------------
// Command-line Interface
#[derive(Debug, Parser)]
enum Cli {
    BuildPackage(BuildPackageArgs),
}

#[derive(Debug, Args)]
struct BuildPackageArgs {
    /// Target platform to build for.
    #[arg(value_enum)]
    platform: Platform,
    /// Target to build for.
    #[arg(long)]
    target: Option<String>,
    /// Features to build with.
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    /// Toolchain to build with.
    #[arg(long)]
    toolchain: Option<String>,
    /// Don't enable the default features.
    #[arg(long)]
    no_default_features: bool,
}

// ----------------------------------------------------------------------------
// Application

fn main() -> Result<()> {
    println!("Starting xtask...");
    env_logger::Builder::new()
        .filter_module("xtask", log::LevelFilter::Info)
        .init();

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    println!("Invoking build_package...");
    match Cli::parse() {
        Cli::BuildPackage(args) => build_package(&workspace, args),
    }
}

// ----------------------------------------------------------------------------
// Subcommands

fn build_package(workspace: &Path, args: BuildPackageArgs) -> Result<()> {
    // Absolute path of the package's root:
    let package_path = xtask::windows_safe_path(workspace);

    println!("Workspace path: {}", workspace.display());

    // Build the package using the provided features and/or target, if any:
    xtask::build_package(
        &package_path,
        args.features,
        args.no_default_features,
        args.toolchain,
        args.target,
        args.platform,
    )
}