use crate::front_matter::NavElem;
use crate::md;
use crate::nav::{NavCategory, NavFolder, NavItem, NavPage};
use crate::path::Path;
use anyhow::{anyhow, bail, ensure, Result};
use log::{debug, info};
use std::fs;

pub fn process_item(elem: NavElem, dir: &Path) -> Result<NavItem> {
    match elem {
        NavElem::File { name, path } => {
            // regular pages can't have nav
            process_page(&dir.join(path), name)
                .and_then(ensure_page_has_no_nav)
                .map(NavItem::Page)
        }
        NavElem::Folder { name, path } => {
            // folder/ is implicitly folder/index.md
            process_folder(&dir.join(path).join("index.md"), name).map(NavItem::Folder)
        }
        NavElem::TaggedIndex { name, path } => {
            process_folder(&dir.join(path), name).map(NavItem::Folder)
        }
        NavElem::Include { name, path } => {
            process_include(&dir.join(path), name).map(NavItem::Category)
        }
        NavElem::Category { name, elems } => {
            process_category(dir, name, elems).map(NavItem::Category)
        }
    }
}

fn process_page(path: &Path, name: Option<String>) -> Result<NavPage> {
    let raw = unwrap!(fs::read_to_string(path), "could not read file {path}",);

    let (fm, content) = unwrap!(md::take_front_matter(&raw), "invalid fm in {path}");

    // don't debug print empty fm
    if let Some(fm) = &fm {
        debug!(target: "take_front_matter", "{path}: {fm:#?}");
    }
    let fm = fm.unwrap_or_default();

    let built_content = md::build(content);
    let fixed_content = md::fix(content);
    let fixed_content = md::prepend_front_matter(&fm, &fixed_content);

    // enforce all files having a title
    let title_h1 = unwrap!(
        md::extract_title_h1(content),
        "all files must have an h1 title, but couldn't extract one from {path}"
    );

    // if fm specifies a name, use it over an assigned name
    // this is mainly only useful for the root index.md
    let name = if let Some(fm_name) = fm.name.clone() {
        ensure!(
            name.is_none(),
            "cannot specify both a fm name and a nav name for {path}"
        );

        fm_name
    } else {
        name.unwrap_or(title_h1)
    };

    Ok(NavPage {
        path: path.clone(),
        name,
        fm,
        raw_content: raw,
        built_content,
        fixed_content,
    })
}

pub fn process_folder(path: &Path, name: Option<String>) -> Result<NavFolder> {
    let index = process_page(path, name)?;

    let dir = path.parent().unwrap();

    ensure!(
        !index.fm.nav.is_empty(),
        "index page {path} is missing fm nav"
    );

    let children = index
        .fm
        .nav
        .iter()
        .cloned()
        .map(|elem| process_item(elem, &dir))
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
            info!(target: "process_include", "skipping directory {path}");
            continue;
        }
        if path.extension() != Some("md".as_ref()) {
            info!(target: "process_include", "skipping non .md file {path}");
            continue;
        }
        if path.file_name() == Some("index.md".as_ref()) {
            bail!("process_include: cannot include/* an index file at {path}");
        }

        // regular pages can't have nav
        children.push(process_page(&path, None).and_then(ensure_page_has_no_nav)?);
    }

    children.sort_by(|x, y| x.name.cmp(&y.name));

    Ok(NavCategory {
        name,
        children: children.into_iter().map(NavItem::Page).collect(),
    })
}

pub fn process_category(dir: &Path, name: String, elems: Vec<NavElem>) -> Result<NavCategory> {
    let children = elems
        .into_iter()
        .map(|elem| process_item(elem, dir))
        .collect::<Result<Vec<NavItem>>>()?;

    Ok(NavCategory { name, children })
}

fn ensure_page_has_no_nav(page: NavPage) -> Result<NavPage> {
    if page.fm.nav.is_empty() {
        Ok(page)
    } else {
        Err(anyhow!("non index page {} cannot have fm nav", page.path))
    }
}
