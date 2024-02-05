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
use pulldown_cmark_mdcat_ratatui::markdown_widget::{MarkdownWidget, Offset, PathOrStr};
use ratatui::style::Stylize;
use ratatui::{prelude::*, widgets::*};
use std::io;
use std::time::{Duration, Instant};

/// App holds the state of the application
struct App {
    question: String,
    answer: Vec<PathOrStr>,
    answer_status: AnswerStatus,
    spent_time: Option<Duration>,
}

impl App {
    fn toggle(&mut self) {
        self.answer_status = self.answer_status.flip();
    }

    fn get_answer(&self) -> Option<&Vec<PathOrStr>> {
        match self.answer_status {
            AnswerStatus::Show => Some(&self.answer),
            AnswerStatus::Hide => None,
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

    let res = run_app(&mut terminal, crate::sm2::Deck::load());

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(ExitCode::ManualExit) => {}
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
) -> Result<ExitCode> {
    let mut offset = Offset { x: 0, y: 0 };
    let Some(mut app) = next(&mut spaced_repetition) else {
        return Ok(ExitCode::OutOfCard);
    };

    loop {
        terminal.draw(|f| ui(f, &mut app, &mut offset))?;
        let start = Instant::now();

        loop {
            if let Event::Key(key) = event::read()? {
                match &app.answer_status {
                    AnswerStatus::Show => match key.code {
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            let spent_time = app.spent_time.unwrap();
                            let q = if spent_time < Duration::from_secs(5) {
                                2
                            } else {
                                1
                            };
                            spaced_repetition.update_and_dump(app.question.to_owned(), q)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            offset = Offset { x: 0, y: 0 };
                            break;
                        }
                        KeyCode::Char('g') | KeyCode::Char('G') => {
                            let spent_time = app.spent_time.unwrap();
                            let q = if spent_time < Duration::from_secs(5) {
                                5
                            } else if spent_time < Duration::from_secs(15) {
                                4
                            } else {
                                3
                            };
                            spaced_repetition.update_and_dump(app.question.to_owned(), q)?;

                            let Some(new_app) = next(&mut spaced_repetition) else {
                                return Ok(ExitCode::OutOfCard);
                            };
                            app = new_app;
                            break;
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            spaced_repetition.update_and_dump(app.question.to_owned(), 0)?;

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

                        KeyCode::Down => {
                            offset.y = offset.y.saturating_add(1);
                            break;
                        }
                        KeyCode::Up => {
                            offset.y = offset.y.saturating_sub(1);
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
                            let end = Instant::now();
                            let duration = end - start;
                            if app.spent_time.is_none() {
                                app.spent_time = Some(duration);
                            }
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

    let v = Vec::new();
    let answer = MarkdownWidget {
        path_or_str: app.get_answer().unwrap_or(&v),
    };
    f.render_stateful_widget(answer, chunks[1], offset);

    let escape_keys = [("Q/Esc", "Quit")];
    let hide_keys = [("<Space>", "Show answer")];
    let show_keys = [("f", "Forget"), ("h", "Hard"), ("g", "Good")];

    let keys: &[(&str, &str)] = match app.answer_status {
        AnswerStatus::Show => &show_keys,
        AnswerStatus::Hide => &hide_keys,
    };

    let spans = escape_keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
            let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Right)
        .fg(Color::Indexed(236))
        .bg(Color::Indexed(232));
    f.render_widget(buttons, chunks[2]);

    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
            let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let buttons = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .fg(Color::Indexed(236));
    f.render_widget(buttons, chunks[2]);
}

fn next<T>(spaced_repetition: &mut T) -> Option<App>
where
    T: SpacedRepetiton,
{
    let Some(question) = spaced_repetition.next_to_review() else {
        return None;
    };
    if let Ok((_, answer)) = query(&question) {
        Some(App {
            question,
            answer,
            answer_status: AnswerStatus::Hide,
            spent_time: None,
        })
    } else {
        spaced_repetition.remove(&question);
        next(spaced_repetition)
    }
}
