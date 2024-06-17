use anyhow::Context;

#[derive(Eq, Debug, PartialEq)]
enum TargetBoard {
    Esp32,
    Local,
    Rp2040,
}

impl EnvVarValue for &str {}

trait EnvVarValue: AsRef<str> {
    fn env_var_value(&self) -> bool {
        std::env::var(self.as_ref())
            .ok()
            .as_deref()
            .map(str::parse::<bool>)
            .transpose()
            .ok()
            .unwrap_or(Some(false))
            .unwrap_or(false)
    }
}

impl core::fmt::Display for TargetBoard {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Esp32 => {
                write!(formatter, "_esp32")
            }
            Self::Local => {
                write!(formatter, "_local")
            }
            Self::Rp2040 => {
                write!(formatter, "_rp2040")
            }
        }
    }
}

impl TargetBoard {
    fn from_env() -> anyhow::Result<Self> {
        match (
            std::env::var("CARGO_CFG_TARGET_OS")
                .context("Unable to determine target os")?
                .as_str(),
            std::env::var("CARGO_CFG_TARGET_ARCH")
                .context("Unable to determine target arch")?
                .as_str(),
            std::env::var("CARGO_CFG_TARGET_VENDOR")
                .context("Unable to determine target vendor")?
                .as_str(),
        ) {
            ("none", "arm", "unknown") => Ok(Self::Rp2040),
            ("none", "xtensa", "unknown") => Ok(Self::Esp32),
            (os, _, vendor) if (os != "none") && (vendor != "unknown") => Ok(Self::Local),
            (os, arch, vendor) => Err(anyhow::Error::msg(format!(
                "unknown target board triple: {}-{}-{}",
                arch, vendor, os
            ))),
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    if "CARGO_FEATURE_BOARD".env_var_value() {
        println!(r#"cargo:rustc-cfg=feature="{}""#, TargetBoard::from_env()?);
    }

    Ok(())
}
