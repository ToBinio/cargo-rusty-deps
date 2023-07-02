use std::fmt::{Display, Formatter};
use std::process::Command;
use std::thread;

use anyhow::anyhow;
use colored::Colorize;
use semver::Version;

use crate::versions::{get_latest_version, get_version_diff, version_to_string};

#[derive(Debug)]
pub struct Dependency {
    name: String,
    version: Version,
    latest_version: Version,
}

pub struct Dependencies {
    dependencies: Vec<Dependency>,
}

impl Dependency {
    fn new(name: String, version: String) -> anyhow::Result<Dependency> {
        let version = Version::parse(&version)?;
        let latest_version = get_latest_version(&name)?;

        Ok(Dependency {
            name,
            version,
            latest_version,
        })
    }
}

pub fn get_all_dependencies() -> anyhow::Result<Dependencies> {
    let output = Command::new("cargo")
        .arg("tree")
        .args(["--depth", "1"])
        .args(["--prefix", "none"])
        .output()?;

    let output = String::from_utf8(output.stdout)?;

    let deps = output
        .lines()
        .skip(1)
        .map(|line| {
            let split: Vec<&str> = line.split(' ').collect();

            Ok((
                split
                    .get(0)
                    .ok_or_else(|| anyhow!("something went not good"))?
                    .to_string(),
                split
                    .get(1)
                    .ok_or_else(|| anyhow!("something went not good"))?
                    .to_string(),
            ))
        })
        .collect::<anyhow::Result<Vec<(String, String)>>>()?;

    let dependencies: anyhow::Result<Vec<Dependency>> = thread::scope(|scope| {
        let mut threads = vec![];

        for (name, version) in deps {
            let thread = scope.spawn(move || {
                let version: String = version.chars().skip(1).collect();

                Dependency::new(name, version)
            });

            threads.push(thread);
        }

        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect()
    });

    Ok(Dependencies {
        dependencies: dependencies?,
    })
}

impl Display for Dependencies {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const NAME: &str = "Name";
        const VERSION: &str = "Version";
        const LATEST_VERSION: &str = "Latest";

        let mut name_width = NAME.len();
        let mut version_width = VERSION.len();

        for dependency in &self.dependencies {
            name_width = name_width.max(dependency.name.len());
            version_width = version_width.max(dependency.version.to_string().len());
        }

        name_width += 3;
        version_width += 3;

        let mut lines = vec![];

        lines.push(format!(
            "{:name_width$}{:version_width$}{}",
            NAME.bold(),
            VERSION.bold(),
            LATEST_VERSION.bold()
        ));

        for dependency in &self.dependencies {
            let version_dif = get_version_diff(&dependency.version, &dependency.latest_version);

            let version_padding = version_width - dependency.version.to_string().len();

            lines.push(format!(
                "{:name_width$}{}{:version_padding$}{}",
                dependency.name,
                version_to_string(&dependency.version, &version_dif),
                "",
                version_to_string(&dependency.latest_version, &version_dif)
            ));
        }

        f.write_str(&lines.join("\n"))?;

        Ok(())
    }
}
