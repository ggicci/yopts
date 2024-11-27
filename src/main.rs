use anyhow::{anyhow, Result};
use clap::Parser;
use ramen;
use std::io::{self, Read};

const APP: &str = "ramen";
const DESC_ABOUT: &str = "An easier way to define and parse arguments in SHELL scripts.";

#[derive(Debug, Parser)]
#[command(name=&APP, version, about=DESC_ABOUT, long_about=None)]
struct Cli {
    #[arg(required = false, value_name = "SPEC")]
    spec: Option<String>,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    print!("cli: {:?}", cli);

    let spec_from_pipe = read_spec_from_stdin()?;
    let spec_from_arg = cli.spec.unwrap_or_default();
    println!("pipe: {:?}", spec_from_pipe);
    println!("arg: {:?}", spec_from_arg);

    // Stop working on data provided both through STDIN and CLI, avoid ambiguity.
    if spec_from_pipe.len() > 0 && spec_from_arg.len() > 0 {
        return Err(anyhow!("Error: both stdin and command-line argument were provided. Please use only one of them."));
    }

    let spec = if spec_from_pipe.is_empty() {
        spec_from_arg
    } else {
        spec_from_pipe
    };

    let output = ramen::parse(&spec)?;
    println!("{}", output);
    Ok(())
}
