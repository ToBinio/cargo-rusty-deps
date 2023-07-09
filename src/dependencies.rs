use std::fmt::{Display, Formatter};
use std::process::Command;
use std::thread;

use anyhow::anyhow;
use colored::Colorize;
use semver::Version;

use crate::versions::{get_latest_version, get_version_diff, version_to_string, VersionDiff};

#[derive(Debug)]
pub struct Dependency {
    name: String,
    version: Version,
    latest_version: Version,
    version_diff: VersionDiff,
}

impl Dependency {
    fn new(name: String, version: String) -> anyhow::Result<Dependency> {
        let version = Version::parse(&version)?;
        let latest_version = get_latest_version(&name)?;

        let version_diff = get_version_diff(&version, &latest_version);

        Ok(Dependency {
            name,
            version,
            latest_version,
            version_diff,
        })
    }
}

pub struct Dependencies {
    dependencies: Vec<Dependency>,
}

impl Dependencies {
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
                let mut split = line.split(' ');

                Ok((
                    split
                        .next()
                        .ok_or_else(|| anyhow!("something went not good"))?
                        .to_string(),
                    split
                        .next()
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

    /// remove all non outdated deps
    /// in other words retain all outdated deps
    pub fn outdated(&mut self) {
        self.dependencies
            .retain(|dependency| dependency.version_diff != VersionDiff::Same);
    }

    pub fn update(&self) {
        let mut command = Command::new("cargo");

        command.arg("add");

        for dependency in &self.dependencies {
            command.arg(format!(
                "{}@{}",
                dependency.name,
                dependency.latest_version.to_string()
            ));
        }

        command.output().expect("oh no");
    }
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
            let version_padding = version_width - dependency.version.to_string().len();

            lines.push(format!(
                "{:name_width$}{}{:version_padding$}{}",
                dependency.name,
                version_to_string(&dependency.version, &dependency.version_diff),
                "",
                version_to_string(&dependency.latest_version, &dependency.version_diff)
            ));
        }

        f.write_str(&lines.join("\n"))?;

        Ok(())
    }
}
