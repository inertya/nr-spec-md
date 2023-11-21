use crate::front_matter::{FrontMatter, NavElem};
use crate::md;
use crate::nav::{NavCategory, NavFolder, NavIndex, NavItem, NavPage};
use crate::path::Path;
use anyhow::{bail, ensure, Result};
use std::fs;

pub fn process_item(elem: &NavElem, dir: &Path) -> Result<NavItem> {
    match elem {
        NavElem::File { name, path } => {
            process_page(&dir.join(path), name.clone()).map(NavItem::Page)
        }
        NavElem::Folder { name, path } => {
            process_folder(&dir.join(path), name.clone()).map(NavItem::Folder)
        }
        NavElem::Include { name, path } => {
            process_include(&dir.join(path), name.clone()).map(NavItem::Category)
        }
        NavElem::Category { name, elems } => {
            process_category(dir, name.clone(), elems).map(NavItem::Category)
        }
    }
}

fn process_content(path: &Path, name: Option<String>) -> Result<(NavPage, Option<FrontMatter>)> {
    let raw = unwrap!(fs::read_to_string(path), "could not read file {path}",);

    let (fm, content) = md::take_front_matter(&raw)?;

    let built_content = md::build(content);
    let fixed_content = md::fix(content);

    let title_h1 = unwrap!(
        md::extract_title_h1(content),
        "couldn't extract title from {path}"
    );

    Ok((
        NavPage {
            path: path.clone(),
            name: name.unwrap_or(title_h1),
            raw_content: raw,
            built_content,
            fixed_content,
        },
        fm,
    ))
}

pub fn process_page(path: &Path, name: Option<String>) -> Result<NavPage> {
    let (page, fm) = process_content(path, name)?;

    ensure!(
        fm.is_none(),
        "regular (non-index) page {path} cannot have front matter"
    );

    Ok(page)
}

pub fn process_index(path: &Path, name: Option<String>) -> Result<NavIndex> {
    let (mut page, fm) = process_content(path, name)?;

    let fm = unwrap!(fm, "index page {path} is missing front matter");

    page.fixed_content = md::prepend_front_matter(&fm, &page.fixed_content);

    Ok(NavIndex { page, fm })
}

pub fn process_folder(path: &Path, name: Option<String>) -> Result<NavFolder> {
    let index = process_index(&path.join("index.md"), name)?;

    let children = index
        .fm
        .nav
        .iter()
        .map(|elem| process_item(elem, path))
        .collect::<Result<Vec<NavItem>>>()?;

    Ok(NavFolder { index, children })
}

// includes are sorted alphabetical (by name or path?)
//
// will ignore directories and non .md files
pub fn process_include(dir: &Path, name: String) -> Result<NavCategory> {
    let read_dir = unwrap!(
        fs::read_dir(dir),
        "couldn't read include/* directory {name} at {dir}"
    );
    let mut children = Vec::new();

    for entry in read_dir {
        let entry = entry.unwrap();
        let path = Path::new(entry.path());
        let metadata = entry.metadata().unwrap();

        if metadata.is_dir() {
            println!("skipping included/* directory {path}");
            continue;
        }
        if path.as_ref().extension() != Some("md".as_ref()) {
            println!("skipping included/* non .md file {path}");
            continue;
        }
        if path.as_ref().file_name() == Some("index.md".as_ref()) {
            bail!("cannot include/* an index file at {path}");
        }

        children.push(process_page(&path, None)?);
    }

    children.sort_by(|x, y| x.name.cmp(&y.name));

    Ok(NavCategory {
        name,
        children: children.into_iter().map(NavItem::Page).collect(),
    })
}

pub fn process_category(dir: &Path, name: String, elems: &[NavElem]) -> Result<NavCategory> {
    let children = elems
        .iter()
        .map(|elem| process_item(elem, dir))
        .collect::<Result<Vec<NavItem>>>()?;

    Ok(NavCategory { name, children })
}
