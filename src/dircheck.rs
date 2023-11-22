use crate::nav::NavFolder;
use crate::path::Path;
use anyhow::Result;
use std::collections::HashSet;
use std::fs;

pub struct DirCheck {
    pub unused: Vec<Path>,
    pub extra: Vec<Path>,
}

// walks dir (recursively) and finds:
// - .md files not in root (not in a nav)
// - all !.md files
pub fn dir_check(dir: &Path, root: &NavFolder) -> Result<DirCheck> {
    let nav_paths = root
        .into_iter() // recursive iterator over all pages in the folder
        .map(|p| &p.path)
        .collect::<HashSet<&Path>>();

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

    Ok(DirCheck { unused, extra })
}

fn walk_dir_recursive(dir: &Path, cb: &mut impl FnMut(Path)) -> Result<()> {
    let read_dir = unwrap!(fs::read_dir(dir), "couldn't read dir {dir}");

    for entry in read_dir {
        let entry = entry.unwrap();
        let path = Path::new(entry.path());
        let metadata = entry.metadata().unwrap();

        if metadata.is_dir() {
            walk_dir_recursive(&path, cb)?;
        } else {
            cb(path);
        }
    }

    Ok(())
}
