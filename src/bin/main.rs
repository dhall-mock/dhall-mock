use anyhow::Error;

extern crate dhall_mock;

fn main() -> Result<(), Error> {
    dhall_mock::run()
}
