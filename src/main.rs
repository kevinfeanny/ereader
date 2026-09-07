#![allow(dead_code)]
#![allow(unused_variables)]

mod buttons;
mod config;
mod display;
mod epub;
mod library;
mod numpad;
mod battery;
mod reader;
mod settings;

use anyhow::Result;

fn main() -> Result<()> {
    println!("E-Reader starting...");
    let mut reader = reader::Reader::new()?;
    reader.run()?;
    Ok(())
}