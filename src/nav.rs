use crate::front_matter::FrontMatter;
use crate::path::Path;
use anyhow::Result;

/// a regular page, eg. `example.md`
#[derive(Debug)]
pub struct NavPage {
    pub path: Path,
    pub name: String,

    pub raw_content: String,
    pub built_content: String,
    pub fixed_content: String,
}

/// An index page (index.md)
///
/// name is page.name
#[derive(Debug)]
pub struct NavIndex {
    pub page: NavPage,
    pub fm: FrontMatter,
}

/// A folder with an index
///
/// name is index.page.name (ignored for root)
#[derive(Debug)]
pub struct NavFolder {
    pub index: NavIndex,
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

//

pub trait ForEachPage {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()>;
}

impl ForEachPage for NavPage {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        f(self)
    }
}

impl ForEachPage for NavIndex {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        self.page.for_each_page(f)
    }
}

impl ForEachPage for NavItem {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        match self {
            NavItem::Page(page) => page.for_each_page(f),
            NavItem::Folder(folder) => folder.for_each_page(f),
            NavItem::Category(category) => category.for_each_page(f),
        }
    }
}

impl ForEachPage for Vec<NavItem> {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        self.iter().try_for_each(|item| item.for_each_page(f))
    }
}

impl ForEachPage for NavFolder {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        self.index.for_each_page(f)?;
        self.children.for_each_page(f)
    }
}

impl ForEachPage for NavCategory {
    fn for_each_page(&self, f: &mut impl FnMut(&NavPage) -> Result<()>) -> Result<()> {
        self.children.for_each_page(f)
    }
}
