use anyhow::{anyhow, bail, ensure, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use serde_yaml::value::{Tag, TaggedValue};
use serde_yaml::{Mapping, Value};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FrontMatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    // converts absent to/from an empty vec
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nav: Vec<NavElem>,
}

// a folder is a nested structure with an index page and some child pages
// a category is a nested structure with NO index page (can't click on it directly) and some child pages

// --- defining files (optional name) ---
// file.md (file)
// just includes that file as a single, non nested page
//
//
// --- defining folders (optional name) ---
// folder/ (folder)
// semantically equivalent to `!index folder/index.md`
//
// !index page.md (tagged index)
// includes a folder with that index page (and nav defined there)
//
//
// --- defining categories ---
// Name: folder/* (include)
// equivalent to listing out each file alphabetically in a category
//
// Name: (category)
//   - file1.md
//   - Name: file2.md
// creates a category with all of those files

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Value", into = "Value")] // converting to/from yaml is much easier than to/from serde
pub enum NavElem {
    // - file.md
    // - Name: file.md
    File { name: Option<String>, path: String },
    // - folder/
    // - Name: folder/
    Folder { name: Option<String>, path: String },
    // - Name: !index abc.md
    TaggedIndex { name: Option<String>, path: String },
    // - Name: folder/*
    Include { name: String, path: String },
    // - 'My Category':
    //     - file.md
    //     - folder/
    Category { name: String, elems: Vec<NavElem> },
}

fn parse_filename(name: Option<String>, mut path: String) -> Result<NavElem> {
    trace!(target: "parse_filename", "name={name:?} path={path}");

    ensure!(path != "index.md", "cannot include index files directly");

    match path {
        // include/*
        _ if path.ends_with("/*") => {
            if let Some(name) = name {
                path.truncate(path.len() - 2);
                Ok(NavElem::Include { name, path })
            } else {
                Err(anyhow!("include {path} must specify a name"))
            }
        }

        // file.md
        _ if path.ends_with(".md") => Ok(NavElem::File { name, path }),

        // folder/
        _ if path.ends_with('/') => {
            path.truncate(path.len() - 1);
            Ok(NavElem::Folder { name, path })
        }

        _ => {
            if name.is_none() {
                bail!("{path} must be a .md file or a folder/");
            } else {
                bail!("{path} must be a .md file, a folder/, or an include/*");
            }
        }
    }
}

fn parse_tagged(name: Option<String>, TaggedValue { tag, value }: TaggedValue) -> Result<NavElem> {
    trace!(target: "parse_tagged", "name={name:?} tag={tag} value={value:?}");

    if tag != "index" {
        bail!("unknown tag: {tag}");
    }
    let Value::String(mut path) = value else {
        bail!("!index tag takes a string path (put name before the tag?)");
    };
    if !path.ends_with(".md") {
        bail!("!index tag {path:?} must be an md file");
    }

    // TODO: what happens if path is ./index.md?
    // TODO: also need to figure out handling for files including themselves

    if let Some(p) = path.strip_suffix("/index.md") {
        // path = p, but reusing the allocation
        path.truncate(p.len());

        // normalize `!index folder/index.md` to `folder/`
        // doesn't change semantics but fix will correct it
        Ok(NavElem::Folder { name, path })
    } else {
        Ok(NavElem::TaggedIndex { name, path })
    }
}

impl TryFrom<Value> for NavElem {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Tagged(tag) => parse_tagged(None, *tag),

            Value::String(s) => parse_filename(None, s),

            // singleton map
            Value::Mapping(map) if map.len() == 1 => {
                let (k, v) = map.into_iter().next().unwrap();

                let Value::String(name) = k else {
                    bail!("mapping key must be a string: {k:?}");
                };

                match v {
                    Value::Tagged(tag) => parse_tagged(Some(name), *tag),
                    Value::String(path) => parse_filename(Some(name), path),
                    Value::Sequence(seq) => seq
                        .into_iter()
                        .map(NavElem::try_from)
                        .collect::<Result<Vec<NavElem>>>()
                        .map(|elems| NavElem::Category { name, elems }),

                    _ => bail!("mapping value must be a sequence or a string"),
                }
            }

            _ => bail!("invalid nav item: {value:#?}"),
        }
    }
}

impl From<NavElem> for Value {
    fn from(value: NavElem) -> Self {
        match value {
            NavElem::File {
                name: Some(name),
                path,
            } => singleton_map(name, path),
            NavElem::File { name: None, path } => Value::String(path),

            NavElem::Folder {
                name: Some(name),
                path,
            } => singleton_map(name, path + "/"),
            NavElem::Folder { name: None, path } => Value::String(path + "/"),

            NavElem::TaggedIndex {
                name: Some(name),
                path,
            } => singleton_map(name, tagged_value("!index", path)),
            NavElem::TaggedIndex { name: None, path } => tagged_value("!index", path),

            NavElem::Include { name, path } => singleton_map(name, path + "/*"),

            NavElem::Category { name, elems } => singleton_map(name, elems),
        }
    }
}

fn singleton_map(k: impl Into<Value>, v: impl Into<Value>) -> Value {
    let mut map = Mapping::with_capacity(1);
    map.insert(k.into(), v.into());
    Value::Mapping(map)
}

fn tagged_value(tag: impl Into<String>, value: impl Into<Value>) -> Value {
    Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new(tag),
        value: value.into(),
    }))
}
