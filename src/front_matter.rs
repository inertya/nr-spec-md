use anyhow::{anyhow, bail, ensure, Result};
use log::trace;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Value", into = "Value")]
pub enum NavElem {
    // - file.md
    // - Name: file.md
    File { name: Option<String>, path: String },
    // - folder/
    // - Name: folder/
    Folder { name: Option<String>, path: String },
    // - Name: folder/*
    Include { name: String, path: String },
    // - 'My Category':
    //     - file.md
    //     - folder/
    Category { name: String, elems: Vec<NavElem> },
}

fn parse_filename(name: Option<String>, mut path: String) -> Result<NavElem> {
    trace!("parse_filename name={name:?} path={path:?}");
    
    ensure!(path != "index.md", "cannot include index files directly");

    match path {
        // include/*
        _ if path.ends_with("/*") => {
            if let Some(name) = name {
                path.truncate(path.len() - 2);
                Ok(NavElem::Include { name, path })
            } else {
                Err(anyhow!("include {path} must specify a filename"))
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
            if name.is_some() {
                bail!("{path} must be a .md file or a folder/");
            } else {
                bail!("{path} must be a .md file, a folder/, or an include/*");
            }
        }
    }
}

impl TryFrom<Value> for NavElem {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => parse_filename(None, s),

            Value::Mapping(map) if map.len() == 1 => {
                let (k, v) = map.into_iter().next().unwrap();

                let Value::String(name) = k else {
                    bail!("mapping key must be a string: {k:?}");
                };

                match v {
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
