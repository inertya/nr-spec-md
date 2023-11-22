use crate::front_matter::FrontMatter;
use anyhow::{bail, Result};
use log::trace;
use pulldown_cmark::{Event, HeadingLevel, Options as CmOptions, Parser, Tag};
use pulldown_cmark_to_cmark::Options as C2cOptions;

const MD_OPTIONS: CmOptions = CmOptions::ENABLE_TABLES
    .union(CmOptions::ENABLE_FOOTNOTES)
    // dont enable strikethrough because it parses single tilde
    .union(CmOptions::ENABLE_HEADING_ATTRIBUTES);

const C2C_OPTIONS: C2cOptions = C2cOptions {
    newlines_after_headline: 2,
    newlines_after_paragraph: 2,
    newlines_after_codeblock: 2,
    newlines_after_table: 2,
    newlines_after_rule: 2,
    newlines_after_list: 2,
    newlines_after_blockquote: 2,
    newlines_after_rest: 1,
    code_block_token_count: 3, // default: 4
    code_block_token: '`',
    list_token: '-', // default: '*'
    ordered_list_token: '.',
    increment_ordered_list_bullets: false,
    emphasis_token: '*',
    strong_token: "**",
};

// transform nr-specific syntax to mkdocs syntax
// specifically for admonitions and stuff
pub fn build(content: &str) -> String {
    // adapt(content, |p| {
    //     yield;
    // })
    content.to_string()
}

pub fn fix(content: &str) -> String {
    // convert to events and then back to a string
    // easiest way to get a consistent style
    adapt(content, |p| p)
}

// pull-push-pull :|
// this is just a convince to avoid all the options and other boilerplate
fn adapt<'a, I>(content: &'a str, f: impl FnOnce(Parser<'a, 'a>) -> I) -> String
where
    I: IntoIterator<Item = Event<'a>>,
{
    let parser = Parser::new_ext(content, MD_OPTIONS);
    // decent guess for cap to try and avoid reallocating
    let mut buf = String::with_capacity(content.len().next_power_of_two());

    let iter = f(parser).into_iter();

    // error is a fmt::Error, which the string fmt::Write impl never returns, so unwrap should never panic
    // discarding the state cuz we dont need it
    let _ = pulldown_cmark_to_cmark::cmark_with_options(iter, &mut buf, C2C_OPTIONS).unwrap();

    // pulldown c2c doesn't keep the trailing newline for some reason
    buf.push('\n');

    buf
}

pub fn extract_title_h1(content: &str) -> Result<String> {
    let mut p = Parser::new_ext(content, MD_OPTIONS);

    match p.next() {
        Some(Event::Start(Tag::Heading(HeadingLevel::H1, _, _))) => {}
        Some(e) => bail!("expecting h1 heading, got: {e:?}"),
        None => bail!("file is empty?"),
    }

    let title = match p.next() {
        Some(Event::Text(s) | Event::Code(s)) => s.into_string(),
        e => unreachable!("bug? non text/code element in heading: {e:?}"),
    };

    assert!(
        matches!(
            p.next(),
            Some(Event::End(Tag::Heading(HeadingLevel::H1, _, _)))
        ),
        "bug? heading hasn't ended"
    );

    Ok(title)
}

pub fn take_front_matter(content: &str) -> Result<(FrontMatter, &str)> {
    let Some(s) = content.strip_prefix("---") else {
        return Ok((FrontMatter::default(), content));
    };

    let Some((fm, remaining)) = s.split_once("\n---") else {
        bail!("unclosed front matter block")
    };

    Ok((serde_yaml::from_str(fm)?, remaining))
}

pub fn prepend_front_matter(fm: &FrontMatter, content: &str) -> String {
    let Ok(serde_yaml::Value::Mapping(map)) = serde_yaml::to_value(fm) else {
        panic!("ser error/fm not a map???");
    };

    if map.is_empty() {
        trace!(
            target: "prepend_front_matter",
            "skipping empty fm, content={:?}",
            content.split_once('\n').map(|x| x.0).unwrap_or_default()
        );
        // skip empty fm blocks
        content.to_string()
    } else {
        format!(
            "---\n{}---\n{content}",
            serde_yaml::to_string(&map).unwrap()
        )
    }
}
