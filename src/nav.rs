use crate::front_matter::FrontMatter;
use crate::path::Path;
use anyhow::Result;

/// a regular page, eg. `example.md`
#[derive(Debug)]
pub struct NavPage {
    pub path: Path,
    pub name: String,

    pub fm: FrontMatter,

    pub raw_content: String,
    pub built_content: String,
    pub fixed_content: String,
}

/// A folder with an index
///
/// name is index.page.name
#[derive(Debug)]
pub struct NavFolder {
    // this may not be an index.md (if it was tagged in nav: `!index abc.md`)
    pub index: NavPage,
    pub children: Vec<NavItem>,
}

/// a category
///
/// no index page
#[derive(Debug)]
pub struct NavCategory {
    pub name: String,

    pub children: Vec<NavItem>,
}

#[derive(Debug)]
pub enum NavItem {
    Page(NavPage),
    Folder(NavFolder),
    Category(NavCategory),
}

impl NavItem {
    #[allow(clippy::needless_lifetimes)] // ???
    pub fn for_each_page<'s>(&'s self, f: &mut impl FnMut(&'s NavPage)) {
        match self {
            NavItem::Page(x) => f(x),
            NavItem::Folder(x) => x.for_each_page(f),
            NavItem::Category(x) => x.children.iter().for_each(|i| i.for_each_page(f)),
        }
    }

    pub fn try_for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        match self {
            NavItem::Page(x) => f(x),
            NavItem::Folder(x) => x.try_for_each_page(f),
            NavItem::Category(x) => x.children.iter().try_for_each(|i| i.try_for_each_page(f)),
        }
    }
}

impl NavFolder {
    #[allow(clippy::needless_lifetimes)] // ???
    pub fn for_each_page<'s>(&'s self, f: &mut impl FnMut(&'s NavPage)) {
        f(&self.index);
        self.children.iter().for_each(|i| i.for_each_page(f));
    }

    pub fn try_for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        f(&self.index)?;
        self.children
            .iter()
            .try_for_each(|i| i.try_for_each_page(f))
    }
}
