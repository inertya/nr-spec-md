use crate::config::Config;
use crate::nav::NavFolder;
use crate::path::Path;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, trace};
use std::convert::Infallible;
use std::fs::File;
use std::io::Write;

pub fn mode_build(root: NavFolder, config: &Config, extra: &[Path]) -> Result<()> {
    debug!("mode build");

    let _ = root;
    let _ = config;
    let _ = extra;
    Ok(())
}

pub fn mode_check(root: &NavFolder) -> Result<()> {
    debug!("mode check");

    let mut total = 0;
    let mut fails = 0;

    let res = root.into_iter().try_for_each(|page| {
        total += 1;

        // TODO rich diff?
        if page.fixed_content != page.raw_content {
            error!("Fix: {}", page.path);
            fails += 1;
        } else {
            debug!("pass: {:?}", page.path);
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
    debug!("mode fix");

    let mut fixed = 0;
    let mut total = 0;

    root.into_iter().try_for_each(|page| {
        total += 1;

        if page.fixed_content == page.raw_content {
            debug!("pass: {:?}", page.path);
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
