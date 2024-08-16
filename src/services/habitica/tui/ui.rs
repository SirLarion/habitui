use std::env;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Clear, Block, BorderType, Paragraph, Padding},
    Frame,
};

use crate::services::habitica::types::Task;

use super::{
  app::{Habitui, AppState}, 
  widgets::{grid::TaskGrid, editor::Editor}, 
  util::Palette
};

const TITLE_STR: &str = "╻ ╻┏━┓┏┓ ╻╺┳╸╻ ╻╻\n┣━┫┣━┫┣┻┓┃ ┃ ┃ ┃┃\n╹ ╹╹ ╹┗━┛╹ ╹ ┗━┛╹";

fn render_task_grid(f: &mut Frame, area: Rect, app: &mut Habitui) {
  let state = &mut app.grid_state;
  let widget = TaskGrid {};
  f.render_stateful_widget(widget, area, state);
}

fn render_footer(f: &mut Frame, area: Rect) {
  let style = Style::default().fg(Palette::BG2.into());
  let block = Block::bordered()
    .border_style(style)
    .border_type(BorderType::Rounded)
    .padding(Padding::horizontal(2));

  f.render_widget(Paragraph::new("q: quit | hjkl: navigate | b: bluupi")
    .block(block), area);
}

fn render_editor(f: &mut Frame, area: Rect, app: &mut Habitui) {
  let editor = Editor::new();
  let popup_area = Rect {
      x: area.width / 3 - 5,
      y: area.height / 3,
      width: area.width / 3 + 10,
      height: area.height / 2,
  };

  if let Some(state) = &mut app.editor_state {
    f.render_stateful_widget(editor, popup_area, state);
  }
}

fn render_debug(f: &mut Frame, area: Rect, msg: &String) {
  let popup_area = Rect {
      x: (area.width / 8) * 6,
      y: (area.height / 6) * 5,
      width: area.width / 6,
      height: area.height / 6,
  };
  f.render_widget(Clear::default(), popup_area);
  f.render_widget(Paragraph::new(msg.to_string())
    .block(Block::bordered()
      .padding(Padding::proportional(1))
    ), popup_area) 
}

pub fn render(frame: &mut Frame, app: &mut Habitui) {
  let [title_area, main_area, footer_area] = Layout::vertical([
    Constraint::Length(3), 
    Constraint::Fill(1), 
    Constraint::Length(3)
  ])
    .spacing(1)
    .areas(frame.area());


  frame.render_widget(Paragraph::new(TITLE_STR)
    .centered()
    .block(Block::default()), title_area);

  render_task_grid(frame, main_area, app);

  match app.state {
    AppState::Editor => {
      render_editor(frame, main_area, app);
    }
    _ => {}
  }

  render_footer(frame, footer_area);

  if let Ok(_) = env::var("HUTCTL_DEBUG") {
    if let Some((msg, _)) = &app.log_debug {
      render_debug(frame, main_area, msg);
    }
  }
}
