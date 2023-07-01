use std::fs;

use anyhow::{anyhow, bail};
use serde::Deserialize;
use toml::{Table, Value};

#[derive(Deserialize)]
struct Document {
    dependencies: Table,
}

fn main() -> anyhow::Result<()> {
    let string = fs::read_to_string("Cargo.toml")?;

    let document: Document = toml::from_str(string.as_str())?;

    for (name, value) in document.dependencies {
        let result: anyhow::Result<String> = match value {
            Value::String(version) => Ok(version),
            Value::Table(version) => {
                let version = version.get("version").ok_or(anyhow!("no"))?;

                match version {
                    Value::String(version) => Ok(version.to_string()),
                    _ => bail!("invalid Cargo.toml"),
                }
            }
            _ => bail!("invalid Cargo.toml"),
        };

        let version = result?;

        println!("{} {}", name, version);
    }

    Ok(())
}
