use anyhow::{Error, Result};
use lexopt::Arg::{Long, Short, Value};
use lexopt::{Parser, ValueExt};

pub struct Args {
    pub path: String,
}

pub fn parse_args() -> Result<Args> {
    let mut path = None;
    let mut parser = Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) if path.is_none() => {
                path = Some(val.string()?);
            }
            Long("help") => {
                println!("Usage: chip-8 PATH");
                std::process::exit(0);
            }
            Short('h') => {
                println!("Usage: chip-8 PATH");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(Args {
        path: path
            .ok_or("missing argument PATH".to_string())
            .map_err(Error::msg)?,
    })
}
