use crate::review_helper::{get_width_and_height, ExitCode};
use crate::spaced_repetition::SpacedRepetiton;
use crate::theme::THEME;
use crate::{query, review_helper::AnswerStatus};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use pulldown_cmark_mdcat_ratatui::bufferline::BufferLine;
use pulldown_cmark_mdcat_ratatui::markdown_widget::{
    FasterMarkdownWidget, MarkdownWidget, Offset, PathOrStr,
};
use ratatui::style::Stylize;
use ratatui::{prelude::*, widgets::*};
use std::cell::OnceCell;
use std::io;

/// App holds the state of the application
struct App {
    question: String,
    answer: Vec<PathOrStr>,
    answer_status: AnswerStatus,
    cell: OnceCell<Vec<BufferLine>>,
}

impl App {
    fn toggle(&mut self) {
        self.answer_status = self.answer_status.flip();
    }

    fn render_answer(&self, f: &mut Frame, area: Rect, offset: &mut Offset) {
        match self.answer_status {
            AnswerStatus::Show => {
                let t = self.cell.get_or_init(|| {
                    self.answer
                        .iter()
                        .map(|x| {
                            let mut y = x.get_bufferlines(area);
                            y.push(BufferLine::Line(Vec::new()));
                            y
                        })
                        .flatten()
                        .collect()
                });
                f.render_stateful_widget(FasterMarkdownWidget { t }, area, offset);
            }
            AnswerStatus::Hide => {
                let w = FasterMarkdownWidget { t: Vec::new() };
                f.render_stateful_widget(w, area, offset);
            }
        }
    }
}

pub fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut history: Vec<String> = Vec::new();

    let res = run_app(&mut terminal, crate::fsrs::Deck::default(), &mut history);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(ExitCode::ManualExit) => {
            println!("{:?}", history);
        }
        Ok(ExitCode::OutOfCard) => {
            println!("All cards reviewed");
        }
        Err(err) => {
            eprintln!("{err:?}");
        }
    }

    Ok(())
}

fn run_app<B: Backend, T: SpacedRepetiton>(
    terminal: &mut Terminal<B>,
    mut spaced_repetition: T,
    history: &mut Vec<String>,
) -> Result<ExitCode> {
    let mut offset = Offset { x: 0, y: 0 };
    let Some(mut app) = next(&mut spaced_repetition) else {
        return Ok(ExitCode::OutOfCard);
    };

    loop {
        if let Some(last_reviewed) = history.last()
            && last_reviewed == &app.question
        {
        } else {
            history.push(app.question.clone());
        }
        terminal.draw(|f| ui(f, &mut app, &mut offset))?;

        loop {
            if let Event::Key(key) = event::read()? {
                match &app.answer_status {
                    AnswerStatus::Show => match key.code {
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            spaced_repetition.update(app.question.to_owned(), 1)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            offset = Offset { x: 0, y: 0 };
                            break;
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            spaced_repetition.update(app.question.to_owned(), 2)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            break;
                        }
                        KeyCode::Char('g') | KeyCode::Char('G') => {
                            spaced_repetition.update(app.question.to_owned(), 3)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            break;
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            spaced_repetition.update(app.question.to_owned(), 4)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            break;
                        }
                        KeyCode::Char(' ') => {
                            app.toggle();
                            break;
                        }

                        KeyCode::Char('j') | KeyCode::Down => {
                            offset.y = offset.y.saturating_add(1);
                            break;
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            offset.y = offset.y.saturating_sub(1);
                            break;
                        }
                        KeyCode::Home => {
                            offset.y = 0;
                            break;
                        }
                        KeyCode::End => {
                            offset.y = u16::MAX;
                            break;
                        }
                        KeyCode::PageUp => {
                            offset.y = offset.y.saturating_sub(10);
                            break;
                        }
                        KeyCode::PageDown => {
                            offset.y = offset.y.saturating_add(10);
                            break;
                        }
                        KeyCode::Left => {
                            offset.x = offset.x.saturating_sub(1);
                            break;
                        }
                        KeyCode::Right => {
                            offset.x = offset.x.saturating_add(1);
                            break;
                        }

                        KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitCode::ManualExit),
                        _ => {}
                    },
                    AnswerStatus::Hide => match key.code {
                        KeyCode::Char(' ') => {
                            app.toggle();
                            break;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(ExitCode::ManualExit),
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, offset: &mut Offset) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // question
            Constraint::Min(1),    // answer
            Constraint::Length(1), // button
        ])
        .split(f.size());

    let question = Paragraph::new(app.question.as_str())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(question, chunks[0]);

    app.render_answer(f, chunks[1], offset);

    let escape_keys = [("Q/Esc", "Quit")];
    let hide_keys = [("<Space>", "Show answer")];
    let show_keys = [("a", "Again"), ("h", "Hard"), ("g", "Good"), ("e", "Easy")];
    let help_keys = [("j/k", "Up/Down"), ("Home/End", "Top/Bottom")];

    let keys: &[(&str, &str)] = match app.answer_status {
        AnswerStatus::Show => &show_keys,
        AnswerStatus::Hide => &hide_keys,
    };

    let spans = keys2span(&escape_keys);
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Right)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232));
    f.render_widget(buttons, chunks[2]);

    let spans = keys2span(keys);
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236));
    f.render_widget(buttons, chunks[2]);

    if app.answer_status == AnswerStatus::Show {
        let spans = keys2span(&help_keys);
        let buttons = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .fg(Color::Indexed(236));
        f.render_widget(buttons, chunks[2]);
    }
}

fn next<T>(spaced_repetition: &mut T) -> Option<App>
where
    T: SpacedRepetiton,
{
    let Ok(Some(question)) = spaced_repetition.next_to_review() else {
        return None;
    };
    if let Ok((_, answer)) = query(&question) {
        Some(App {
            question,
            answer,
            answer_status: AnswerStatus::Hide,
            cell: OnceCell::new(),
        })
    } else {
        spaced_repetition.remove(&question).unwrap();
        next(spaced_repetition)
    }
}

fn keys2span<'a>(keys: &'a [(&str, &str)]) -> Vec<Span<'a>> {
    keys.iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
            let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec()
}
