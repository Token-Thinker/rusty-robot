use std::{
    fs,
    path::{Path, PathBuf},
};
use anyhow::{bail, Result};
use clap::ValueEnum;
use log::info;
use strum_macros::{Display, EnumIter};
use crate::cargo::{CargoArgsBuilder, run};

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
pub enum Package {
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
    mut features: Vec<String>,
    no_default_features: bool,
    toolchain: Option<String>,
    target: Option<String>,
    package: Package,
) -> Result<()> {
    // First, determine the path to the MCU package based on the selected package
    let package_paths = match package {
        Package::Esp32 => vec![workspace.join("hardware/mcu/esp32")],
        Package::Local => vec![workspace.join("hardware/mcu/local")],
        Package::Rp2040 => vec![workspace.join("hardware/mcu/rp2040")],
    };

    // Define the paths to the hardware and server packages
    let common_packages = vec![workspace.join("hardware"), workspace.join("server")];

    // Build the MCU package first
    for package_path in &package_paths {
        println!("Building MCU package: {}", package_path.display());

        if !package_path.exists() || !package_path.join("Cargo.toml").exists() {
            bail!(
                "The package path '{}' is not a valid directory or doesn't contain Cargo.toml",
                package_path.display()
            );
        }

        info!("Building package '{}'", package_path.display());

        let mut builder = CargoArgsBuilder::default()
            .subcommand("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(package_path.join("Cargo.toml").to_string_lossy().to_string());

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

        if !features.is_empty() {
            builder = builder.features(&features);
        }

        if no_default_features {
            builder = builder.arg("--no-default-features");
        }

        let cargo_args = builder.build();
        info!("Running cargo with args: {:?}", cargo_args);

        run(&cargo_args, package_path)?;
    }

    // Now, build the hardware and server packages
    for package_path in common_packages {
        println!("Building common package: {}", package_path.display());

        if !package_path.exists() || !package_path.join("Cargo.toml").exists() {
            bail!(
                "The package path '{}' is not a valid directory or doesn't contain Cargo.toml",
                package_path.display()
            );
        }

        info!("Building package '{}'", package_path.display());

        // Ensure the "mcu" feature is included when building the hardware package
        let mut common_builder = CargoArgsBuilder::default()
            .subcommand("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(package_path.join("Cargo.toml").to_string_lossy().to_string());

        if let Some(toolchain) = &toolchain {
            common_builder = common_builder.toolchain(toolchain);
        }

        if let Some(target) = &target {
            common_builder = common_builder.target(target);
            if target.contains("xtensa") {
                common_builder = common_builder.toolchain("esp");
                common_builder = common_builder.arg("-Zbuild-std=core,alloc");
            }
        }

        // Add the "mcu" feature if building the hardware package
        if package_path.ends_with("hardware") {
            if !features.contains(&"mcu".to_string()) {
                features.push("mcu".to_string());
            }
            common_builder = common_builder.features(&features);
        }

        if no_default_features {
            common_builder = common_builder.arg("--no-default-features");
        }

        let cargo_args = common_builder.build();
        info!("Running cargo with args: {:?}", cargo_args);

        run(&cargo_args, &package_path)?;
    }

    Ok(())
}

// ----------------------------------------------------------------------------
// Helper Functions

/// Return a (sorted) list of paths to each valid Cargo package in the
/// workspace.
pub fn package_paths(workspace: &Path, package: Package) -> Result<Vec<PathBuf>> {
    println!("package_paths invoked with workspace: {}", workspace.display());
    let mut paths = Vec::new();
    let target_subdir = match package {
        Package::Esp32 => "hardware/mcu/esp32",
        Package::Local => "hardware/mcu/local",
        Package::Rp2040 => "hardware/mcu/rp2040"
    };

    for entry in fs::read_dir(workspace)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let path = entry.path();
            println!("Checking path: {}", path.display());
            // Ensure it is a package (has Cargo.toml)
            if path.join("Cargo.toml").exists() {
                paths.push(path.clone());
            }

            // Check for nested packages like in `hardware/mcu`
            if let Ok(subdirs) = fs::read_dir(path.join("mcu")) {
                for subdir in subdirs {
                    let subdir = subdir?;
                    if subdir.file_type()?.is_dir() {
                        let subdir_path = subdir.path();
                        if subdir_path.ends_with(target_subdir) && subdir_path.join("Cargo.toml").exists() {
                            paths.push(subdir_path);
                        }
                    }
                }
            }
        }
    }

    paths.sort();
    Ok(paths)
}

/// Make the path "Windows"-safe
pub fn windows_safe_path(path: &Path) -> PathBuf {
    PathBuf::from(path.to_str().unwrap().to_string().replace("\\\\?\\", ""))
}
