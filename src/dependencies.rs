use std::{fs, thread};

use anyhow::{anyhow, bail};
use colored::Colorize;
use crates_index::SparseIndex;
use http::Request;
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
    latest_version: Version,
}

impl Dependency {
    fn new(name: String, version: String) -> anyhow::Result<Dependency> {
        let version = Version::parse(&version)?;

        let latest_version = Dependency::get_latest_version(&name)?;

        Ok(Dependency {
            name,
            version,
            latest_version,
        })
    }

    fn get_latest_version(name: &str) -> anyhow::Result<Version> {
        let index = SparseIndex::new_cargo_default()?;

        let req = index.make_cache_request(name)?;

        let (parts, _) = req.into_parts();
        let req = Request::from_parts(parts, vec![]);

        let req: reqwest::blocking::Request = req.try_into().unwrap();

        let client = reqwest::blocking::ClientBuilder::new().gzip(true).build()?;

        let res = client.execute(req)?;

        let mut builder = http::Response::builder()
            .status(res.status())
            .version(res.version());

        builder
            .headers_mut()
            .unwrap()
            .extend(res.headers().iter().map(|(k, v)| (k.clone(), v.clone())));

        let body = res.bytes()?;
        let res = builder.body(body.to_vec())?;

        index.parse_cache_response(name, res, true)?;

        let krate = index.crate_from_cache(name)?;

        let latest_version = Version::parse(
            krate
                .highest_normal_version()
                .ok_or_else(|| anyhow!("todo"))?
                .version(),
        )?;

        Ok(latest_version)
    }
}

pub fn get_all_dependencies() -> anyhow::Result<Vec<Dependency>> {
    let string = fs::read_to_string("Cargo.toml")?;

    let document: Document = toml::from_str(string.as_str())?;

    let dependencies: anyhow::Result<Vec<Dependency>> = thread::scope(|scope| {
        let mut threads = vec![];

        for (name, value) in document.dependencies {
            let thread = scope.spawn(|| {
                let result: anyhow::Result<String> = match value {
                    Value::String(version) => Ok(version),
                    Value::Table(version) => {
                        let version = version.get("version").ok_or_else(|| anyhow!("todo"))?;

                        match version {
                            Value::String(version) => Ok(version.to_string()),
                            _ => bail!("invalid Cargo.toml"),
                        }
                    }
                    _ => bail!("invalid Cargo.toml"),
                };

                Dependency::new(name, result?)
            });

            threads.push(thread);
        }

        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect()
    });

    dependencies
}

pub fn print_dependencies(dependencies: &Vec<Dependency>) {
    const NAME: &str = "Name";
    const VERSION: &str = "Version";
    const LATEST_VERSION: &str = "Latest";

    let mut name_width = NAME.len();
    let mut version_width = VERSION.len();
    let mut latest_width = LATEST_VERSION.len();

    for dependency in dependencies {
        name_width = name_width.max(dependency.name.len());
        version_width = version_width.max(dependency.version.to_string().len());
        latest_width = latest_width.max(dependency.latest_version.to_string().len());
    }

    name_width += 3;
    version_width += 3;
    latest_width += 3;

    println!(
        "{:name_width$}{:version_width$}{:latest_width$}",
        NAME.bold(),
        VERSION.bold(),
        LATEST_VERSION.bold()
    );

    for dependency in dependencies {
        println!(
            "{:name_width$}{:version_width$}{:latest_width$}",
            dependency.name, dependency.version, dependency.latest_version
        );
    }
}
