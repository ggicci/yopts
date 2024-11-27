use clap::Command;
use thiserror::Error;
use yaml_rust::{ScanError, Yaml, YamlLoader};

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
