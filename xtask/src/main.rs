use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Parser};
use xtask::{cargo::CargoArgsBuilder, package_paths, Platform};

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
    /// Lint all packages in workspace with cargo clippy
    LintPackages(LintPackagesArgs),
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

#[derive(Debug, Args)]
struct LintPackagesArgs
{
    /// Run in 'check' mode; exits with 0 if linting is correct, 1 otherwise
    #[arg(long)]
    check: bool,

    /// Target platform to lint for
    #[arg(value_enum)]
    platform: Platform,
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
        Cli::LintPackages(args) => lint_packages(&workspace, args),
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

fn lint_packages(
    workspace: &Path,
    args: LintPackagesArgs,
) -> Result<()>{
    let (target, _toolchain) = match args.platform {
        Platform::Esp32 => ("xtensa-esp32-none-elf", "esp"),
        Platform::Rp2040 => ("thumbv6m-none-eabi", "default"),
        Platform::Local => ("x86_64-unknown-linux-gnu", "default"),
    };

    let package_paths = package_paths(workspace)?
        .into_iter()
        .filter(|package_path| {
            package_path.ends_with("hardware")
                || package_path.ends_with("comms")
                || package_path.ends_with(format!("{:?}", args.platform).to_lowercase())
        })
        .collect::<Vec<_>>();

    for path in package_paths {
        match path.file_name().and_then(|name| name.to_str()) {
            Some("hardware") | Some("comms") => {
                lint_package(
                    &path,
                    &[
                        "-Zbuild-std=core,alloc",
                        &format!("--target={}", target),
                        &format!("--features={}", args.platform),
                    ],
                )?;
            }
            Some("app") => {
                lint_package(
                    &path,
                    &[
                        "-Zbuild-std=core,alloc",
                        &format!("--target={}", target),
                    ],
                )?;
            }
            _ => {
                lint_package(&path, &[])?;
            }
        }
    }

    Ok(())
}

fn lint_package(path: &Path, args: &[&str]) -> Result<()>{
    log::info!("Linting package: {}", path.display());

    let mut builder = CargoArgsBuilder::default().subcommand("clippy");

    for arg in args {
        builder = builder.arg(arg.to_string());
    }

    let cargo_args = builder
        .arg("--release")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .build();

    xtask::cargo::run(&cargo_args, path)
}
