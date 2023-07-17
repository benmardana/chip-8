use anyhow::{Error, Result};
use lexopt::Arg::{Long, Short, Value};
use lexopt::{Parser, ValueExt};

pub struct Args {
    pub path: String,
    pub hertz: Option<f64>,
}

pub fn parse_args() -> Result<Args> {
    let mut path = None;
    let mut parser = Parser::from_env();
    let mut hertz: Option<f64> = None;
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) => {
                path = Some(val.string()?);
            }
            Long("help") | Short('h') => {
                println!("Usage: chip-8 PATH [--hertz=NUM]");
                std::process::exit(0);
            }
            Long("hertz") => {
                hertz = parser.value()?.parse().ok();
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(Args {
        path: path
            .ok_or("missing argument PATH".to_string())
            .map_err(Error::msg)?,
        hertz,
    })
}
