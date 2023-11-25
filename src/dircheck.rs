use crate::nav::NavFolder;
use crate::path::Path;
use anyhow::Result;
use log::{debug, trace};
use std::collections::HashSet;
use std::fs;
use std::fs::Metadata;

pub struct DirCheck {
    pub unused: Vec<Path>,
    pub extra: Vec<Path>,
}

// walks dir (recursively) and finds:
// - .md files not in root (not in a nav)
// - all !.md files
pub fn dir_check(dir: &Path, root: &NavFolder) -> Result<DirCheck> {
    trace!(target: "dir_check", "dir={dir:?}");

    let mut nav_paths = HashSet::new();

    root.for_each_page(&mut |p| {
        nav_paths.insert(&p.path);
    });

    let mut unused = Vec::new();
    let mut extra = Vec::new();

    walk_dir_recursive(dir, &mut |path| {
        if path.extension() != Some("md".as_ref()) {
            // if it's not an .md file, it's extra
            extra.push(path);
        } else if !nav_paths.contains(&path) {
            // if it's an .md file, and it's not in the nav, it's unused
            unused.push(path);
        }
    })?;

    debug!(target: "dir_check", "extra: {extra:#?}");

    Ok(DirCheck { unused, extra })
}

fn walk_dir_recursive(dir: &Path, cb: &mut impl FnMut(Path)) -> Result<()> {
    let read_dir = unwrap!(fs::read_dir(dir), "couldn't read dir {dir}");

    for entry in read_dir {
        let entry = entry.unwrap();
        let path = Path::new_owned(entry.path());
        let metadata = entry.metadata().unwrap();
        trace!(
            target: "walk_dir_recursive",
            "{} {path}",
            file_type_str(&metadata),
        );

        if metadata.is_dir() {
            walk_dir_recursive(&path, cb)?;
        } else {
            cb(path);
        }
    }

    Ok(())
}

fn file_type_str(meta: &Metadata) -> &'static str {
    if meta.is_file() {
        "file"
    } else if meta.is_dir() {
        "dir "
    } else if meta.is_symlink() {
        "sym "
    } else {
        "??? "
    }
}
