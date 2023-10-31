use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

use super::{output, Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
}
#[derive(Default)]
pub struct Home {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  pub text: Vec<String>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub mode: Mode,
  pub connection_string: String,
}

impl Home {
  pub fn new(connection_string: String) -> Self {
    Self { connection_string, ..Default::default() }
  }
}

impl Component for Home {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {},

      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Percentage(5), Constraint::Percentage(45), Constraint::Percentage(50)])
      .split(area);

    let info_block = layout[0];
    let output_block = layout[1];
    let input_block = layout[2];

    f.render_widget(Paragraph::new("output block").block(Block::default().borders(Borders::ALL)), output_block);
    f.render_widget(Paragraph::new("input block").block(Block::default().borders(Borders::ALL)), input_block);
    f.render_widget(
      Paragraph::new(self.connection_string.clone()).block(Block::default().borders(Borders::ALL)),
      info_block,
    );

    Ok(())
  }
}
