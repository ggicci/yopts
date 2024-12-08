use anyhow::{anyhow, Result};
use clap::Parser;
use log::LevelFilter;
use ramen;
use std::io::{self, Read};
const APP: &str = "ramen";
const DESC_ABOUT: &str = "An easier way to define and parse arguments in SHELL scripts.";

#[derive(Debug, Parser)]
#[command(name=&APP, version, about=DESC_ABOUT, long_about=None)]
struct Cli {
    /// A definition of the argument parser in YAML format.
    #[arg()]
    spec: Option<String>,

    /// Enable debug mode.
    #[arg(short, long, value_name = "DEBUG")]
    debug: bool,

    /// The arguments to parse. Passed as the last argument, after a "--".
    /// Usually it's "$@" in the bash script. e.g.
    ///
    ///     eval "$( ramen "$program" -- "$@" )"
    #[arg(last = true)]
    optstring: Vec<String>,
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();
    init_logging(if cli.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    });

    let spec_from_pipe = read_spec_from_stdin()?;
    let spec_from_arg = cli.spec.unwrap_or_default();

    // Stop working on data provided both through STDIN and CLI, avoid ambiguity.
    if spec_from_pipe.len() > 0 && spec_from_arg.len() > 0 {
        return Err(anyhow!("Error: both stdin and command-line argument were provided. Please use only one of them."));
    }

    let spec = if spec_from_pipe.is_empty() {
        spec_from_arg
    } else {
        spec_from_pipe
    };

    // Since we will be calling clap::Command::get_matches_from(VEC) API
    // to parse the optstring, and it treats the first element from the given
    // VEC as the name of the program, we insert a dummy value here to optstring.
    cli.optstring.insert(0, "PROG".to_string());
    let output = ramen::parse(&spec, &cli.optstring)?;
    println!("{}", output);
    Ok(())
}

/// Read data from STDIN if provided.
fn read_spec_from_stdin() -> Result<String> {
    // Avoids reading from stdin when it is connected to a terminal.
    if atty::is(atty::Stream::Stdin) {
        return Ok("".to_string());
    }
    let mut pipe_input = String::new();
    io::stdin()
        .read_to_string(&mut pipe_input)
        .map_err(|e| anyhow!("read STDIN error: {}", e))?;
    Ok(pipe_input)
}

fn init_logging(level_filter: LevelFilter) {
    env_logger::Builder::new().filter_level(level_filter).init();
}
