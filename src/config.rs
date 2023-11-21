use semver::VersionReq;
use serde::Deserialize;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Build,
    Check,
    Fix,
}

#[derive(Debug)]
pub struct Config {
    pub file: ConfigFile,
    pub mode: Mode,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(rename = "version")]
    pub version_req: VersionReq,
    pub metadata: Metadata,
    pub build: Build,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub copyright: String,
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub source: String,
    pub output: String,
}
