#![forbid(unsafe_code)]

use crate::config::{Config, ConfigFile, Mode};
use crate::dircheck::{dir_check, DirCheck};
use crate::modes::{mode_build, mode_check, mode_fix};
use crate::path::Path;
use crate::process::process_folder;
use anyhow::{bail, ensure, Context, Result};
use log::{debug, error, info, warn, LevelFilter};
use semver::Version;
use std::process::ExitCode;
use std::time::Instant;
use std::{env, fs};

macro_rules! unwrap {
    ($x:expr, $($fmt:tt)+) => {
        // dont use ? because we don't need the implicit .into() and it breaks some inference
        match anyhow::Context::with_context($x, || format!($($fmt)+)) {
            Ok(x) => x,
            Err(e) => return Err(e),
        }
    };
}

mod config;
mod dircheck;
mod front_matter;
mod md;
mod mkdocs;
mod modes;
mod nav;
mod path;
mod process;

const CONFIG_PATH: &str = "nr-spec-md.toml";

const HELP_MESSAGE: &str = r"
This tool helps to build and validate the inertya specification
https://github.com/inertya/nr-spec-md

Usage: nr-spec-md [MODE]

Modes:
b, build - Builds a mkdocs site into mkdocs/
c, check - Checks that the markdown matches the style guide
f, fix   - Fixes any style mistakes (will modify src/)
";

fn main() -> ExitCode {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .parse_default_env()
        .init();

    let start = Instant::now();

    let res = run();

    debug!("took {:?}", start.elapsed());

    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e}");
            e.chain()
                .skip(1)
                .for_each(|cause| error!("Caused by: {cause}"));
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    info!("nr-spec-md v{version}");

    let config = get_config()?;

    debug!("{config:#?}");

    ensure!(
        config.file.version_req.matches(&version),
        "version mismatch: current version is v{}, but config requires {}",
        version,
        config.file.version_req,
    );

    let src = Path::new(&config.file.build.source);

    // this will read and process every file (specified in index navs)
    let root = process_folder(&src, None)?;

    // print unused files
    let DirCheck { unused, extra } = dir_check(&src, &root).context("dir check error")?;

    unused.iter().for_each(|path| warn!("Unused file: {path}"));

    //

    match config.mode {
        Mode::Build => mode_build(root, &config, &extra),
        Mode::Check => mode_check(&root),
        Mode::Fix => mode_fix(&root),
    }
}

fn get_config() -> Result<Config> {
    // read mode before config to allow --help with no config file
    let mode = get_mode()?;

    let config_str = fs::read_to_string(CONFIG_PATH).context("could not open config file")?;
    let config_file: ConfigFile = toml::from_str(&config_str).context("invalid config")?;

    Ok(Config {
        file: config_file,
        mode,
    })
}

fn get_mode() -> Result<Mode> {
    let mode = match env::args().nth(1).as_deref() {
        Some("b" | "build") => Mode::Build,
        Some("c" | "check") => Mode::Check,
        Some("f" | "fix") => Mode::Fix,
        None | Some("help" | "--help") => {
            eprintln!("{}", HELP_MESSAGE.trim());
            std::process::exit(2);
        }
        Some(s) => bail!("Unknown mode {s:?}, try `nr-spec-md help`"),
    };

    debug!("Mode: {mode:?}");

    Ok(mode)
}
