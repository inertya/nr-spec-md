use crate::config::Config;
use crate::nav::{ForEachPage, NavFolder};
use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::Write;

pub fn mode_build(root: NavFolder, config: &Config) -> Result<()> {
    let _ = root;
    let _ = config;
    Ok(())
}

pub fn mode_check(root: NavFolder) -> Result<()> {
    let mut total = 0;
    let mut fails = 0;

    root.for_each_page(&mut |page| {
        total += 1;

        // TODO rich diff?
        if page.fixed_content != page.raw_content {
            println!("{} needs fixing", page.path);
            fails += 1;
        }

        Ok(())
    })
    .expect("always Ok");

    if fails == 0 {
        eprintln!("All {total} files look good!");
        Ok(())
    } else {
        Err(anyhow!("{fails}/{total} files need fixing :("))
    }
}

pub fn mode_fix(root: NavFolder) -> Result<()> {
    let mut fixed = 0;
    let mut total = 0;

    root.for_each_page(&mut |page| {
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
        eprintln!("All {total} files look good!");
    } else {
        eprintln!("Fixed {fixed}/{total} files")
    }

    Ok(())
}
