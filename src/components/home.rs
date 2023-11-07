use super::{output, Component, Frame};
use crate::config::key_event_to_string;
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};
use chatgpt::prelude::*;
use chatgpt::types::Role::User;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use libc::exit;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::result;
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::UnboundedSender;
use tracing_subscriber::fmt::format;
use tui_input::{backend::crossterm::EventHandler, Input};
extern crate exitcode;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
}

pub struct Home {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  pub keymap: HashMap<KeyEvent, Action>,
  pub mode: Mode,
  pub input: Input,
  pub conversation: Conversation,
  pub last_events: Vec<KeyEvent>,
  pub gpt_client: ChatGPT,
}

impl Home {
  pub async fn new(init_query: String, gpt_api_key: String) -> Self {
    let gpt_client = ChatGPT::new(gpt_api_key).unwrap();
    let conversation: Conversation = gpt_client.new_conversation();
    Self {
      gpt_client,
      conversation,
      input: "".into(),
      mode: Mode::Normal,
      last_events: Vec::new(),
      command_tx: Default::default(),
      config: Config::new().unwrap(),
      keymap: HashMap::new(),
    }
  }
  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }
  pub fn tick(&mut self) {
    self.last_events.drain(..);
  }

  pub fn add(&mut self, s: String) {
    let msg = ChatMessage { role: (User), content: (s) };
    self.conversation.history.push(msg)
  }
  fn send_query(&mut self, s: String) {}
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
      Action::Tick => self.tick(),
      Action::EnterNormal => {
        self.mode = Mode::Normal;
      },
      Action::SendQuery(s) => {
        self.add(s); //append query to conversation
      },
      Action::EnterInsert => {
        self.mode = Mode::Insert;
      },
      Action::EnterProcessing => {
        self.mode = Mode::Processing;
      },
      Action::ExitProcessing => {
        // TODO: Make this go to previous mode instead
        self.mode = Mode::Normal;
      },
      _ => (),
    }
    Ok(None)
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Processing => return Ok(None),
      Mode::Normal => match key.code {
        _ => return Ok(None), //todo: Implement selecting diff messages
      },
      Mode::Insert => match key.code {
        KeyCode::Esc => Action::EnterNormal,
        KeyCode::Enter => {
          if let Some(sender) = &self.command_tx {
            if let Err(e) = sender.send(Action::SendQuery(self.input.value().to_string())) {
              eprintln!("Failed to send action: {:?}", e);
            }
          }
          self.input = "".into();
          Action::EnterNormal
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          Action::Update
        },
      },
    };
    Ok(Some(action))
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default().constraints([Constraint::Percentage(80), Constraint::Min(12)].as_ref()).split(rect);
    let mut text: Vec<Line> = self
      .conversation
      .history
      .clone()
      .iter()
      .map(|l| Line::from(format!("{:?} : {}", l.role.clone(), l.content.clone())))
      .collect();
    text.insert(0, "".into());
    text.insert(0, "Type into input and hit enter to display here".dim().into());
    text.insert(0, "".into());
    text.insert(0, "".into());
    text.insert(0, "".into());
    f.render_widget(
      Paragraph::new(text)
        .block(
          Block::default()
            .title("shellwyze")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(match self.mode {
              Mode::Processing => Style::default().fg(Color::Red),
              _ => Style::default(),
            })
            .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center),
      rects[0],
    );

    let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = self.input.visual_scroll(width as usize);

    let input = Paragraph::new(self.input.value())
      .style(match self.mode {
        Mode::Insert => Style::default().fg(Color::Yellow),
        _ => Style::default(),
      })
      .scroll((0, scroll as u16))
      .block(Block::default().borders(Borders::ALL).title(Line::from(vec![
        Span::raw("Enter Input Mode "),
        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("i", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
      ])));
    f.render_widget(input, rects[1]);

    if self.mode == Mode::Insert {
      f.set_cursor((rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2), rects[1].y + 1)
    }

    f.render_widget(
      Block::default()
        .title(
          ratatui::widgets::block::Title::from(format!(
            "{:?}",
            &self.last_events.iter().map(|k| key_event_to_string(k)).collect::<Vec<_>>()
          ))
          .alignment(Alignment::Right),
        )
        .title_style(Style::default().add_modifier(Modifier::BOLD)),
      Rect { x: rect.x + 1, y: rect.height.saturating_sub(1), width: rect.width.saturating_sub(2), height: 1 },
    );

    Ok(())
  }
}
