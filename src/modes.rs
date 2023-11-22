use crate::config::Config;
use crate::nav::NavFolder;
use crate::path::Path;
use anyhow::{anyhow, Context, Result};
use log::{error, info};
use std::convert::Infallible;
use std::fs::File;
use std::io::Write;

pub fn mode_build(root: NavFolder, config: &Config, extra: &[Path]) -> Result<()> {
    let _ = root;
    let _ = config;
    let _ = extra;
    Ok(())
}

pub fn mode_check(root: &NavFolder) -> Result<()> {
    let mut total = 0;
    let mut fails = 0;

    let res = root.into_iter().try_for_each(|page| {
        total += 1;

        // TODO rich diff?
        if page.fixed_content != page.raw_content {
            error!("Fix: {}", page.path);
            fails += 1;
        }

        Ok::<(), Infallible>(())
    });

    match res {
        Ok(()) => {}
        Err(infallible) => match infallible {},
    }

    if fails == 0 {
        info!("All {total} files look good!");
        Ok(())
    } else {
        Err(anyhow!("{fails}/{total} files need fixing :("))
    }
}

pub fn mode_fix(root: &NavFolder) -> Result<()> {
    let mut fixed = 0;
    let mut total = 0;

    root.into_iter().try_for_each(|page| {
        total += 1;

        if page.fixed_content == page.raw_content {
            return Ok(());
        }

        fixed += 1;

        let path = &page.path;
        let mut file = unwrap!(
            File::options().write(true).truncate(true).open(path),
            "couldn't open {path} for fixing",
        );

        file.write_all(page.fixed_content.as_bytes())
            .with_context(|| format!("write error while fixing file {path}"))
    })?;

    if fixed == 0 {
        info!("All {total} files look good!");
    } else {
        info!("Fixed {fixed}/{total} files")
    }

    Ok(())
}
