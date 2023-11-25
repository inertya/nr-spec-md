use crate::config::Config;
use crate::nav::NavFolder;
use crate::path::Path;
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::fs::File;
use std::io::Write;

pub fn mode_build(root: NavFolder, config: &Config, extra: &[Path]) -> Result<()> {
    debug!(target: "mode", "build");

    let _ = root;
    let _ = config;
    let _ = extra;
    Ok(())
}

pub fn mode_check(root: &NavFolder) -> Result<()> {
    debug!(target: "mode", "check");

    let mut total = 0;
    let mut fails = 0;

    root.for_each_page(&mut |page| {
        total += 1;

        // TODO rich diff?
        if page.fixed_content != page.raw_content {
            error!(target: "", "Fix: {}", page.path);
            fails += 1;
        } else {
            debug!(target: "mode_check", "pass {}", page.path);
        }
    });

    if fails == 0 {
        info!(target: "", "All {total} files look good!");
        Ok(())
    } else {
        Err(anyhow!("{fails}/{total} files need fixing :("))
    }
}

pub fn mode_fix(root: &NavFolder) -> Result<()> {
    debug!(target: "mode", "fix");

    let mut fixed = 0;
    let mut total = 0;

    root.try_for_each_page(&mut |page| {
        total += 1;

        if page.fixed_content == page.raw_content {
            debug!(target: "mode_fix", "pass  {}", page.path);
            return Ok(());
        }

        fixed += 1;

        let path = &page.path;
        let mut file = unwrap!(
            File::options().write(true).truncate(true).open(path),
            "couldn't open {path} for fixing",
        );

        unwrap!(
            file.write_all(page.fixed_content.as_bytes()),
            "write error while fixing file {path}"
        );

        debug!(target: "mode_fix", "fixed {path}");
        Ok(())
    })?;

    if fixed == 0 {
        info!(target: "", "All {total} files look good!");
    } else {
        info!(target: "", "Fixed {fixed}/{total} files")
    }

    Ok(())
}
