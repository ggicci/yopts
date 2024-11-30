use clap::{Arg, Command};
use thiserror::Error;
use yaml_rust::{ScanError, Yaml, YamlLoader};

use once_cell::sync::Lazy;
use regex::Regex;

static REG_SHORT_LONG_ARG_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^-(?P<short>[a-zA-Z])/--(?P<long>[a-zA-Z][a-zA-Z0-9-]*)$").unwrap());

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse yaml error: {0}")]
    ParseYaml(#[from] ScanError),

    #[error("no docs detected in the given yaml")]
    NoDocs,

    #[error("multi-docs detected in the given yaml, which is not supported")]
    MultiDocs,
}

pub struct ArgumentParser {
    doc: Yaml,
}

impl ArgumentParser {
    pub fn new(doc: Yaml) -> Self {
        Self { doc }
    }

    pub fn version(&self) -> String {
        String::from(self.doc["version"].as_str().unwrap_or_default())
    }

    pub fn program(&self) -> String {
        String::from(self.doc["program"].as_str().unwrap_or_default())
    }

    pub fn iter_args(&self) -> impl Iterator<Item = Argument> {
        ArgumentIterator::new(self.doc["args"].as_vec())
    }
}

/// Represents a [`clap::Arg`], see tutorial:
/// https://docs.rs/clap/latest/clap/_tutorial/chapter_2/index.html
pub struct Argument<'a> {
    doc: &'a Yaml,
}

impl<'a> Argument<'a> {
    pub fn new(doc: &'a Yaml) -> Self {
        Self { doc }
    }

    pub fn bare_name(&self) -> Option<&str> {
        self.doc.as_str()
    }

    pub fn name(&self) -> &str {
        self.bare_name()
            .or(self.doc["name"].as_str())
            .unwrap_or_default()
    }

    /// Provide the short arg name, ex. -c, -d, -t, etc.
    pub fn short(&self) -> Option<String> {
        match self.bare_name() {
            Some(name) => extract_short_long_name(name).map(|(short, _)| short),
            None => self.doc["short"].as_str().map(|x| x.to_string()),
        }
    }

    /// Provide the long arg name, ex. --file, --num-threads, etc.
    pub fn long(&self) -> Option<String> {
        match self.bare_name() {
            Some(name) => extract_short_long_name(name).map(|(_, long)| long),
            None => self.doc["long"].as_str().map(|x| x.to_string()),
        }
    }

    /// The type of the argument, can be string, number, boolean.
    pub fn r#type(&self) -> &str {
        self.doc["type"].as_str().unwrap_or("string")
    }

    /// The default value of the argument on absent.
    pub fn default(&self) -> &str {
        self.doc["default"].as_str().unwrap_or_default()
    }

    pub fn select(&self) -> Option<Vec<&str>> {
        self.doc["select"]
            .as_vec()
            .map(|x| x.iter().map(|v| v.as_str().unwrap_or_default()).collect())
    }
}

/// Extract the short and long name from the given text when it complies to the pattern `-s/--long`.
fn extract_short_long_name(haystack: &str) -> Option<(String, String)> {
    if let Some(captures) = REG_SHORT_LONG_ARG_NAME.captures(haystack) {
        let short_name = captures.name("short").unwrap().as_str();
        let long_name = captures.name("long").unwrap().as_str();
        Some((short_name.to_string(), long_name.to_string()))
    } else {
        None
    }
}

struct ArgumentIterator<'a> {
    args: Option<&'a Vec<Yaml>>,
    index: usize,
}

impl<'a> ArgumentIterator<'a> {
    fn new(args: Option<&'a Vec<Yaml>>) -> Self {
        Self { args, index: 0 }
    }
}

impl<'a> Iterator for ArgumentIterator<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.args {
            None => None,
            Some(args) => {
                if self.index < args.len() {
                    let item = &args[self.index];
                    self.index += 1;
                    Some(Argument::new(item))
                } else {
                    None
                }
            }
        }
    }
}

pub fn parse(yaml: &str) -> Result<String, Error> {
    let res = String::new();

    let mut docs = YamlLoader::load_from_str(yaml)?;
    validate_root_docs(&docs)?;

    let parser = ArgumentParser::new(docs.remove(0));

    Ok(res)
}

fn validate_root_docs(docs: &Vec<Yaml>) -> Result<(), Error> {
    if docs.len() == 0 {
        return Err(Error::NoDocs);
    }
    if docs.len() > 1 {
        return Err(Error::MultiDocs);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use yaml_rust::{Yaml, YamlLoader};

    use super::{Argument, ArgumentParser};

    #[test]
    fn get_program() -> anyhow::Result<()> {
        let parser = ArgumentParser::new(load_yaml("program: hello")?);
        assert_eq!("hello", parser.program());
        Ok(())
    }

    #[test]
    fn arg_bare_name() -> anyhow::Result<()> {
        let doc = load_yaml("SRC")?;
        let parg = Argument::new(&doc);
        assert_eq!(Some("SRC"), parg.bare_name());
        assert_eq!("SRC", parg.name());
        Ok(())
    }

    #[test]
    fn arg_name() -> anyhow::Result<()> {
        let doc = load_yaml(
            r#"
        name: DEST
        "#,
        )?;
        let parg = Argument::new(&doc);
        assert!(parg.bare_name().is_none());
        assert_eq!("DEST", parg.name());
        Ok(())
    }

    fn load_yaml(yaml: &str) -> anyhow::Result<Yaml> {
        let mut docs = YamlLoader::load_from_str(yaml)?;
        Ok(docs.remove(0))
    }
}
