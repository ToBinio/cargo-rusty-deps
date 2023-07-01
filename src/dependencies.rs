use std::fs;

use anyhow::{anyhow, bail};
use colored::Colorize;
use semver::Version;
use serde::Deserialize;
use toml::{Table, Value};

#[derive(Deserialize)]
struct Document {
    dependencies: Table,
}

#[derive(Debug)]
pub struct Dependency {
    name: String,
    version: Version,
}

impl Dependency {
    fn new(name: String, version: String) -> anyhow::Result<Dependency> {
        let version = Version::parse(&version)?;

        Ok(Dependency { name, version })
    }
}

pub fn get_all_dependencies() -> anyhow::Result<Vec<Dependency>> {
    let string = fs::read_to_string("Cargo.toml")?;

    let document: Document = toml::from_str(string.as_str())?;

    let mut dependencies = vec![];

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

        dependencies.push(Dependency::new(name, result?)?)
    }

    Ok(dependencies)
}

pub fn print_dependencies(dependencies: &Vec<Dependency>) {
    const NAME: &str = "Name";
    const VERSION: &str = "Version";

    let mut name_width = NAME.len();
    let mut version_width = VERSION.len();

    for dependency in dependencies {
        name_width = name_width.max(dependency.name.len());
        version_width = version_width.max(dependency.version.to_string().len());
    }

    name_width += 3;
    version_width += 3;

    println!(
        "{:name_width$}{:version_width$}",
        NAME.bold(),
        VERSION.bold()
    );

    for dependency in dependencies {
        println!(
            "{:name_width$}{:version_width$}",
            dependency.name, dependency.version
        );
    }
}
