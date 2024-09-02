use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use clap::ValueEnum;
use log::info;
use strum_macros::{Display, EnumIter};

use crate::cargo::{run, CargoArgsBuilder};

pub mod cargo;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumIter,
    ValueEnum,
    serde::Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Platform
{
    #[strum(serialize = "esp32")]
    Esp32,
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "rp2040")]
    Rp2040,
}

/// Build the specified package, using the given toolchain/target/features if
/// provided.
pub fn build_package(
    workspace: &Path,
    features: Vec<String>,
    no_default_features: bool,
    toolchain: Option<String>,
    target: Option<String>,
    platform: Platform,
) -> Result<()>
{
    let package_paths = package_paths(workspace)?
        .into_iter()
        .filter(|package_path| {
            package_path.ends_with("hardware")
                || package_path.ends_with("comms")
                || package_path.ends_with(format!("{:?}", platform).to_lowercase())
        })
        .collect::<Vec<_>>();

    for package_path in package_paths {
        let builder = setup_package_build(
            &package_path,
            features.clone(),
            no_default_features,
            toolchain.clone(),
            target.clone(),
            platform,
            true, // Set to true for quiet build
        )?;

        let cargo_args = builder.build();
        info!("Running cargo with args: {:?}", cargo_args);

        run(&cargo_args, &package_path)?;
    }

    Ok(())
}

pub fn run_package(
    workspace: &Path,
    features: Vec<String>,
    no_default_features: bool,
    toolchain: Option<String>,
    platform: Platform,
    bin: &str,
) -> Result<()>
{
    let target = match platform {
        Platform::Esp32 => "xtensa-esp32-none-elf".to_string(),
        Platform::Rp2040 => "thumbv6m-none-eabi".to_string(),
        Platform::Local => "x86_64-unknown-linux-gnu".to_string(),
    };

    build_package(
        workspace,
        features.clone(),
        no_default_features,
        toolchain.clone(),
        Some(target.clone()),
        platform,
    )?;

    let app_path = workspace.join("app");

    let mut app_features = features.clone();
    if bin != "rr-app" {
        app_features.push(bin.to_string());
    }

    let builder = setup_package_build(
        &app_path,
        app_features,
        no_default_features,
        toolchain,
        Some(target),
        platform,
        false,
    )?;

    let mut builder = builder.subcommand("run");

    if bin == "rr-app" {
        builder = builder.arg("--bin").arg(bin);
    }
    else {
        builder = builder.arg("--example").arg(bin);
    }

    let cargo_args = builder.build();
    info!("Running cargo with args: {:?}", cargo_args);

    run(&cargo_args, &app_path)?;

    Ok(())
}

// ----------------------------------------------------------------------------
// Helper Functions
fn setup_package_build(
    package_path: &Path,
    features: Vec<String>,
    no_default_features: bool,
    toolchain: Option<String>,
    target: Option<String>,
    platform: Platform,
    quiet: bool,
) -> Result<CargoArgsBuilder>
{
    println!("Setting up build for package: {}", package_path.display());

    if !package_path.exists() || !package_path.join("Cargo.toml").exists() {
        bail!(
            "The package path '{}' is not a valid directory or doesn't contain Cargo.toml",
            package_path.display()
        );
    }

    info!("Setting up package '{}'", package_path.display());

    let mut builder = CargoArgsBuilder::default()
        .subcommand("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(
            package_path
                .join("Cargo.toml")
                .to_string_lossy()
                .to_string(),
        );

    if quiet {
        builder = builder.arg("--quiet");
    }

    if package_path.ends_with("hardware") || package_path.ends_with("comms") {
        let mut specific_features = features.clone();
        specific_features.push(platform.to_string());
        builder = builder.features(&specific_features);
    }
    else {
        builder = builder.features(&features);
    }

    if let Some(toolchain) = &toolchain {
        builder = builder.toolchain(toolchain);
    }

    if let Some(target) = &target {
        builder = builder.target(target);
        if target.contains("xtensa") {
            builder = builder.toolchain("esp");
            builder = builder.arg("-Zbuild-std=core,alloc");
        }
    }

    if no_default_features {
        builder = builder.arg("--no-default-features");
    }

    Ok(builder)
}

/// Return a list paths to each valid CArgo package in the workspace
pub fn package_paths(workspace: &Path) -> Result<Vec<PathBuf>>
{
    let mut paths = vec![];
    for entry in fs::read_dir(workspace)? {
        let entry = entry?.path();

        if entry.is_dir() {
            paths.extend(package_paths(&entry)?);
        }
        else if entry.is_file() && entry.file_name() == Some("Cargo.toml".as_ref()) {
            paths.push(entry.parent().unwrap().to_path_buf());
        }
    }

    Ok(paths)
}

/// Make the path "Windows"-safe
pub fn windows_safe_path(path: &Path) -> PathBuf
{
    PathBuf::from(path.to_str().unwrap().to_string().replace("\\\\?\\", ""))
}
