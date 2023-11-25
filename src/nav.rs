use crate::front_matter::FrontMatter;
use crate::path::Path;
use std::iter::{once, Chain, Once};

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

type NavPageIter<'a> = Once<&'a NavPage>;

/// A folder with an index
///
/// name is index.page.name (ignored for root)
#[derive(Debug)]
pub struct NavFolder {
    // this may not be an index.md (if it was tagged in nav: `!index abc.md`)
    pub index: NavPage,
    pub children: Vec<NavItem>,
}

type NavFolderIter<'a> = Chain<NavPageIter<'a>, NavChildrenIter<'a>>;

/// a category
///
/// no index page
#[derive(Debug)]
pub struct NavCategory {
    pub name: String,

    pub children: Vec<NavItem>,
}

// without indirection rustc cant resolve the type layout
type NavChildrenIter<'a> = Box<dyn Iterator<Item = &'a NavPage> + 'a>;

fn nav_children_iter<'a>(data: &'a [NavItem]) -> NavChildrenIter<'a> {
    Box::new(data.iter().flat_map(<&'a NavItem>::into_iter))
}

#[derive(Debug)]
pub enum NavItem {
    Page(NavPage),
    Folder(NavFolder),
    Category(NavCategory),
}

pub enum NavItemIter<'a> {
    Page(NavPageIter<'a>),
    Folder(NavFolderIter<'a>),
    Category(NavChildrenIter<'a>),
}

//

impl<'a> IntoIterator for &'a NavPage {
    type Item = &'a NavPage;
    type IntoIter = NavPageIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        once(self)
    }
}

impl<'a> IntoIterator for &'a NavFolder {
    type Item = &'a NavPage;
    type IntoIter = NavFolderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.index
            .into_iter()
            .chain(nav_children_iter(&self.children))
    }
}

impl<'a> IntoIterator for &'a NavCategory {
    type Item = &'a NavPage;
    type IntoIter = NavChildrenIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        nav_children_iter(&self.children)
    }
}

impl<'a> IntoIterator for &'a NavItem {
    type Item = &'a NavPage;
    type IntoIter = NavItemIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            NavItem::Page(p) => NavItemIter::Page(p.into_iter()),
            NavItem::Folder(f) => NavItemIter::Folder(f.into_iter()),
            NavItem::Category(c) => NavItemIter::Category(c.into_iter()),
        }
    }
}

//

impl<'a> Iterator for NavItemIter<'a> {
    type Item = &'a NavPage;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NavItemIter::Page(i) => i.next(),
            NavItemIter::Folder(i) => i.next(),
            NavItemIter::Category(i) => i.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            NavItemIter::Page(i) => i.size_hint(),
            NavItemIter::Folder(i) => i.size_hint(),
            NavItemIter::Category(i) => i.size_hint(),
        }
    }
}
