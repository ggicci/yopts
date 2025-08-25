use clap::{Arg, ArgMatches, Command};
use log::debug;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use std::fmt::Write;
use thiserror::Error;
use yaml_rust::{ScanError, Yaml, YamlLoader};

use crate::version::Version;

pub type Result<T> = std::result::Result<T, Error>;

pub const MAGIC_PROG_NAME: &str = "__YOPTS_PROG__";

static REG_SHORT_LONG_ARG_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^-(?P<short>[a-zA-Z])(/--(?P<long>[a-zA-Z][a-zA-Z0-9-]{1,}))*|--(?P<only_long>[a-zA-Z][a-zA-Z0-9-]{1,})$").unwrap()
});

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse yaml error: {0}")]
    ParseYaml(#[from] ScanError),

    #[error("no docs detected in the given yaml")]
    NoDocs,

    #[error("multi-docs detected in the given yaml, which is not supported")]
    MultiDocs,

    #[error(
        "invalid version, must be one of: {:?}, (key: version)",
        Version::supported_versions()
    )]
    InvalidVersion,

    #[error("missing program name (key: program)")]
    MissingProgram,

    #[error("missing argument name (key: args[].name)")]
    MissingArgumentName,

    #[error(transparent)]
    Format(#[from] std::fmt::Error),
}

pub struct ArgumentParser {
    doc: Yaml,
}

impl ArgumentParser {
    const LOG_TARGET: &'static str = "ArgumentParser";

    pub fn new(doc: Yaml) -> Result<Self> {
        debug!(target: Self::LOG_TARGET, "parse spec doc: {doc:?}");
        let parser = Self { doc };
        parser.validate()?;
        Ok(parser)
    }

    /// The version of the spec.
    pub fn version(&self) -> &str {
        self.doc["version"].as_str().unwrap_or_default()
    }

    /// Convert the version number from string to `Version`.
    pub fn parsed_version(&self) -> Option<Version> {
        Version::try_from(self.version()).ok()
    }

    /// The name of the program.
    pub fn program(&self) -> &str {
        self.doc["program"].as_str().unwrap_or_default()
    }

    /// Add a prefix to the name of each argument in the output script.
    /// For example, if a argument named "verbose", and prefix is "myapp_",
    /// the final output script will be `myapp_verbose=xxx`. By default,
    /// no prefix will be applied.
    pub fn output_prefix(&self) -> &str {
        self.doc["output_prefix"].as_str().unwrap_or_default()
    }

    /// A description of the program.
    pub fn about(&self) -> &str {
        self.doc["about"].as_str().unwrap_or_default()
    }

    /// Create a list of Argument instance by parsing the `args` definitions.
    pub fn args(&self) -> Vec<Argument> {
        self.doc["args"]
            .as_vec()
            .map(|vec| vec.iter().map(|item| Argument::new(item)).collect())
            .unwrap_or_default()
    }

    pub fn build_clap_command(&self) -> Result<Command> {
        let mut command = Command::new(self.program().to_owned()).about(self.about().to_owned());

        for arg in self.args().iter() {
            debug!(target: Self::LOG_TARGET, "build clap command, arg={arg:?}, id={}", arg.id()?);
            let mut is_positional = true;
            let mut clap_arg = Arg::new(arg.id()?);
            if let Some(short) = arg.short() {
                clap_arg = clap_arg.short(short);
                is_positional = false;
            }
            if let Some(long) = arg.long() {
                clap_arg = clap_arg.long(long);
                is_positional = false;
            }
            if arg.is_flag() {
                clap_arg = clap_arg.action(clap::ArgAction::SetTrue);
            }
            if let Some(help) = arg.help() {
                clap_arg = clap_arg.help(help.to_string());
            }

            clap_arg = clap_arg.required(is_positional);
            command = command.arg(clap_arg);
        }
        command.build();
        Ok(command)
    }

    fn validate(&self) -> Result<()> {
        if self.parsed_version().is_none() {
            return Err(Error::InvalidVersion);
        }
        if self.program().is_empty() {
            return Err(Error::MissingProgram);
        }
        Ok(())
    }
}

/// Represents a [`clap::Arg`], see tutorial:
/// https://docs.rs/clap/latest/clap/_tutorial/chapter_2/index.html
#[derive(Debug, Clone)]
pub struct Argument<'a> {
    doc: &'a Yaml,
}

impl<'a> Argument<'a> {
    pub fn new(doc: &'a Yaml) -> Self {
        Self { doc }
    }

    /// Only when an argument was defined using a single string, instead
    /// of a hash representation in the given YAML spec, it will return
    /// the whole string as the "bare name". ex.
    /// `args: [SRC, DST, -t/--threads, -s, --long]`, here every string
    /// in this array represents an argument. And the whole string of each
    /// is identified as a "bare name". "SRC" is a bare name, so as the rest.
    pub fn bare_name(&self) -> Option<&str> {
        self.doc.as_str()
    }

    /// The value of "name" in the provided arg definition.
    /// ex.
    ///
    /// ```yaml
    /// args:
    /// - name: threads
    ///   short: -t
    ///   long: --threads
    /// ```
    pub fn name(&self) -> Option<&str> {
        self.doc["name"].as_str()
    }

    /// Provide the short arg name, ex. -c, -d, -t, etc.
    pub fn short(&self) -> Option<char> {
        let haystack = self
            .bare_name()
            .or(self.doc["short"].as_str())
            .unwrap_or_default();
        extract_short_long_name(haystack)
            .0
            .map(|x| x.chars().next())
            .flatten()
    }

    /// Provide the long arg name, ex. --file, --num-threads, etc.
    pub fn long(&self) -> Option<String> {
        let haystack = self
            .bare_name()
            .or(self.doc["long"].as_str())
            .unwrap_or_default();
        extract_short_long_name(haystack).1
    }

    /// The id of the argument, it uses the value in the following order:
    /// name -> long -> short -> bare_name.
    pub fn id(&self) -> Result<String> {
        self.name()
            .map(|x| x.to_string())
            .or(self.long())
            .or(self.short().map(|x| x.to_string()))
            .or(self.bare_name().map(|x| x.to_string()))
            .ok_or(Error::MissingArgumentName)
    }

    /// The type of the argument, can be string, number, boolean.
    pub fn typ(&self) -> &str {
        self.doc["type"].as_str().unwrap_or("string")
    }

    pub fn is_flag(&self) -> bool {
        ["bool", "boolean"].contains(&self.typ())
    }

    /// The default value of the argument on absent.
    pub fn default(&self) -> &str {
        self.doc["default"].as_str().unwrap_or_default()
    }

    pub fn help(&self) -> Option<&str> {
        self.doc["help"].as_str()
    }

    pub fn select(&self) -> Option<Vec<&str>> {
        self.doc["select"]
            .as_vec()
            .map(|x| x.iter().map(|v| v.as_str().unwrap_or_default()).collect())
    }
}

pub fn parse(spec_yaml: &str, optstring: &[String]) -> Result<String> {
    let mut docs = YamlLoader::load_from_str(spec_yaml)?;
    validate_root_docs(&docs)?;

    // Build an ArgumentParser instance by parsing the given spec.
    let parser = ArgumentParser::new(docs.remove(0))?;
    let command = parser.build_clap_command()?;

    let optstring = normalize_optstring(optstring);
    // Let the command parse optstring. And use the matches to compose the eval script.
    debug!(target: "yopts::parse", "OPTSTRING: {optstring:?}");
    let matches = command.get_matches_from(optstring);
    compose_shell_script(&parser, &matches)
}

/// Add some salts to the given optstring.
/// Since we will be calling clap::Command::get_matches_from(VEC) API
/// to parse the optstring, and it treats the first element from the given
/// VEC as the name of the program, we insert a dummy value here to optstring.
fn normalize_optstring(optstring: &[String]) -> Vec<String> {
    let mut new_optstring = Vec::from(optstring);
    if new_optstring.len() == 0 || new_optstring[0] != MAGIC_PROG_NAME {
        new_optstring.insert(0, MAGIC_PROG_NAME.to_string());
    }
    new_optstring
}

fn compose_shell_script(parser: &ArgumentParser, matches: &ArgMatches) -> Result<String> {
    let mut script = String::with_capacity(256);

    for arg in parser.args().iter() {
        let key = arg.id()?;
        let prefix = parser.output_prefix();
        let output_key = format!("{prefix}{key}");

        debug!(
            target: "yopts::compose_shell_script",
            "key={key:?}, value={:?}",
            matches.get_raw(&key),
        );
        if arg.is_flag() {
            let flag = matches.get_flag(&key);
            writeln!(&mut script, "{}={}", output_key, flag)?;
        } else {
            let value = matches.get_one::<String>(&key);
            if let Some(given_value) = value {
                writeln!(&mut script, "{}={}", output_key, given_value)?;
            }
        }
    }

    Ok(script)
}

fn validate_root_docs(docs: &Vec<Yaml>) -> Result<()> {
    if docs.len() == 0 {
        return Err(Error::NoDocs);
    }
    if docs.len() > 1 {
        return Err(Error::MultiDocs);
    }
    Ok(())
}

/// Extract the short and long name from the given text when it complies to the pattern `-s/--long`.
fn extract_short_long_name(haystack: &str) -> (Option<String>, Option<String>) {
    let convert = |m: Option<Match<'_>>| m.map(|x| x.as_str().to_string());
    let mut short_name = None;
    let mut long_name = None;
    if let Some(captures) = REG_SHORT_LONG_ARG_NAME.captures(haystack) {
        short_name = convert(captures.name("short"));
        long_name = convert(captures.name("long")).or(convert(captures.name("only_long")));
    }
    (short_name, long_name)
}

#[cfg(test)]
mod test {
    use yaml_rust::{Yaml, YamlLoader};

    use crate::parser::Error;

    use super::{Argument, ArgumentParser};

    #[test]
    fn test_require_version_and_program_in_spec() -> anyhow::Result<()> {
        let parser = ArgumentParser::new(load_yaml(
            r#"
        version: "1.0.0"
        program: hello
        "#,
        )?)?;
        assert_eq!("hello", parser.program());
        Ok(())
    }

    #[test]
    fn test_err_missing_version() -> anyhow::Result<()> {
        let parser_rs = ArgumentParser::new(load_yaml(
            r#"
        program: hello
        "#,
        )?);

        assert!(matches!(parser_rs, Err(Error::InvalidVersion)));
        Ok(())
    }

    #[test]
    fn test_err_missing_program() -> anyhow::Result<()> {
        let parser_rs = ArgumentParser::new(load_yaml(
            r#"
        version: "1.0.0"
        "#,
        )?);

        assert!(matches!(parser_rs, Err(Error::MissingProgram)));
        Ok(())
    }

    #[test]
    fn test_arg_bare_name() -> anyhow::Result<()> {
        let doc = load_yaml("SRC")?;
        let parg = Argument::new(&doc);
        assert_eq!(Some("SRC"), parg.bare_name());
        assert_eq!("SRC", parg.id()?);
        Ok(())
    }

    #[test]
    fn test_arg_name() -> anyhow::Result<()> {
        let doc = load_yaml(
            r#"
        name: DEST
        "#,
        )?;
        let parg = Argument::new(&doc);
        assert!(parg.bare_name().is_none());
        assert_eq!("DEST", parg.id()?);
        Ok(())
    }

    /// Helper function to load a YAML and returns the first doc.
    fn load_yaml(yaml: &str) -> anyhow::Result<Yaml> {
        let mut docs = YamlLoader::load_from_str(yaml)?;
        Ok(docs.remove(0))
    }
}
