#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod config;
pub mod mode;
pub mod tui;
pub mod utils;

use crate::{
  app::App,
  utils::{initialize_logging, initialize_panic_handler, version},
};
use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;
use std::env;

async fn tokio_main() -> Result<()> {
  initialize_logging()?;

  initialize_panic_handler()?;
  let gpt_api_key = env::var("GPT_API_KEY").expect("API key not found");
  let args = Cli::parse();
  let mut app = App::new(args.tick_rate, args.frame_rate, args.init_query, gpt_api_key)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
