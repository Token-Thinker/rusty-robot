use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Parser};
use xtask::{cargo::CargoArgsBuilder, Platform};

// ----------------------------------------------------------------------------
// Command-line Interface
#[derive(Debug, Parser)]
enum Cli
{
    /// Build the packages with the given options
    BuildPackage(BuildPackageArgs),
    /// Runs application with given options
    Run(RunArgs),
    /// Format all packages in the workspace with rustfmt
    FmtPackages(FmtPackagesArgs),
}

#[derive(Debug, Args)]
struct BuildPackageArgs
{
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

#[derive(Debug, Args)]
struct RunArgs
{
    /// Target platform to build for.
    #[arg(value_enum)]
    platform: Platform,
    /// Which part of the app to run (main, examples, etc.)
    #[arg(long)]
    bin: String,
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

#[derive(Debug, Args)]
struct FmtPackagesArgs
{
    /// Run in 'check' mode; exists with 0 if formatted correctly, 1 otherwise
    #[arg(long)]
    check: bool,
}
// ----------------------------------------------------------------------------
// Application

fn main() -> Result<()>
{
    println!("Starting xtask...");
    env_logger::Builder::new()
        .filter_module("xtask", log::LevelFilter::Info)
        .init();

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace.parent().unwrap().canonicalize()?;

    match Cli::parse() {
        Cli::BuildPackage(args) => build_package(&workspace, args),
        Cli::Run(args) => run(&workspace, args),
        Cli::FmtPackages(args) => fmt_packages(&workspace, args),
    }
}

// ----------------------------------------------------------------------------
// Subcommands

fn build_package(
    workspace: &Path,
    args: BuildPackageArgs,
) -> Result<()>
{
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

fn run(
    workspace: &Path,
    args: RunArgs,
) -> Result<()>
{
    let package_path = xtask::windows_safe_path(workspace);

    println!("Workspace path: {}", workspace.display());

    xtask::run_package(
        &package_path,
        args.features,
        args.no_default_features,
        args.toolchain,
        args.platform,
        &args.bin,
    )
}

fn fmt_packages(
    workspace: &Path,
    args: FmtPackagesArgs,
) -> Result<()>
{
    let package_paths = xtask::package_paths(workspace)?;

    for path in package_paths {
        log::info!("Formatting package: {}", path.display());

        let mut cargo_args = CargoArgsBuilder::default()
            .subcommand("fmt")
            .arg("--all")
            .build();

        if args.check {
            cargo_args.push("--".into());
            cargo_args.push("--check".into());
        }

        // Run cargo fmt in each package directory
        xtask::cargo::run(&cargo_args, &path)?;
    }

    Ok(())
}
