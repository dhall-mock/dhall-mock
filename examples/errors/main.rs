use std::fmt::Display;
use std::fs;

use serde::export::Formatter;
use thiserror;

use anyhow::{anyhow, Context, Error};

fn define_error() -> Result<(), Error> {
    Err(anyhow!("Custom error from anyhow!"))
}

fn context_error() -> Result<(), Error> {
    define_error().context("Added some context using context function")
}

#[derive(thiserror::Error, Debug)]
struct StructuredContext {
    message: String,
    value: u8,
}

unsafe impl Send for StructuredContext {}
unsafe impl Sync for StructuredContext {}

impl Display for StructuredContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Structured context using context : {:?}", self)
    }
}

fn define_structured_context(message: String, value: u8) -> Result<(), Error> {
    context_error().context(StructuredContext { message, value })
}

fn augment_external_errors() -> Result<(), Error> {
    fs::read_to_string("file_that_does_not_exists")
        .map(|_| ())
        .context("Error reading file")
}

fn main() {
    match define_structured_context("param string".to_string(), 42) {
        Ok(_) => eprintln!("Can't be ok"),
        Err(err) => {
            println!("Current error message : {}", err);
            println!();
            println!("Print chain of errors");
            for error in err.chain() {
                println!("{}", error)
            }
            println!();
        }
    }
    match augment_external_errors() {
        Ok(_) => eprintln!("Can't be ok"),
        Err(err) => {
            println!("Current error message : {}", err);
            println!();
            println!("Search for specific std::io::Error");
            match err.downcast::<std::io::Error>() {
                Ok(io_error) => println!("Downcasted to std::io::Error {}", io_error),
                Err(cast_error) => eprintln!("Error downcasting : {}", cast_error),
            }
            println!();
        }
    }
}
