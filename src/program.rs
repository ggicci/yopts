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

pub fn parse(yaml: &str) -> Result<String, Error> {
    let res = String::new();

    let docs = YamlLoader::load_from_str(yaml)?;
    validate_root_docs(&docs)?;

    let doc = &docs[0];

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
