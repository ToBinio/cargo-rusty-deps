use std::ops::Not;

use anyhow::anyhow;
use colored::Colorize;
use crates_index::SparseIndex;
use http::Request;
use semver::Version;

use crate::versions::VersionDiff::Same;

#[derive(PartialEq, Debug)]
pub enum VersionDiff {
    Major,
    Minor,
    Patch,
    Pre,
    Build,
    Same,
}

pub fn get_latest_version(name: &str) -> anyhow::Result<Version> {
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

pub fn get_version_diff(version_a: &Version, version_b: &Version) -> VersionDiff {
    if version_a.major != version_b.major {
        return VersionDiff::Major;
    }

    if version_a.minor != version_b.minor {
        return VersionDiff::Minor;
    }

    if version_a.patch != version_b.patch {
        return VersionDiff::Patch;
    }

    if version_a.pre != version_b.pre {
        return VersionDiff::Pre;
    }

    if version_a.build != version_b.build {
        return VersionDiff::Build;
    }

    Same
}

pub fn version_to_string(version: &Version, version_dif: &VersionDiff) -> String {
    let mut major = version.major.to_string().normal();
    let mut minor = version.minor.to_string().normal();
    let mut patch = version.patch.to_string().normal();
    let mut pre = version.pre.to_string().normal();
    let mut build = version.build.to_string().normal();

    match version_dif {
        VersionDiff::Major => major = major.red(),
        VersionDiff::Minor => minor = minor.red(),
        VersionDiff::Patch => patch = patch.red(),
        VersionDiff::Pre => pre = pre.red(),
        VersionDiff::Build => build = build.red(),
        Same => {}
    }

    let mut text = format!("{}.{}.{}", major, minor, patch);

    if version.pre.is_empty().not() {
        text += &format!("-{}", pre);
    }

    if version.build.is_empty().not() {
        text += &format!("+{}", build);
    }

    text
}
