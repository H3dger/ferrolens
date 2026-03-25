use crate::app::App;
use crate::error::Result;
use crate::input::Session;
use crate::ui;
use anyhow::anyhow;
use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn run(app: App, source_label: impl Into<String>) -> Result<()> {
    let mut lifecycle = TerminalLifecycle::default();

    enable_raw_mode()?;
    lifecycle.mark_raw_mode_enabled();

    let mut stdout = io::stdout();
    if let Err(error) = crossterm::execute!(stdout, EnterAlternateScreen) {
        let restore_result = lifecycle.restore(&mut || Ok(()), &mut || disable_raw_mode());
        return finalize_terminal_result(Err(error.into()), restore_result, Ok(()));
    }
    lifecycle.mark_alt_screen_enabled();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            let restore_result = restore_terminal(&lifecycle);
            let cursor_result = show_cursor();
            return finalize_terminal_result(Err(error.into()), restore_result, cursor_result);
        }
    };
    let mut session = Session::new(app, source_label);

    let run_result = run_event_loop(&mut terminal, &mut session);
    let restore_result = restore_terminal(&lifecycle);
    let cursor_result = show_cursor();

    finalize_terminal_result(run_result, restore_result, cursor_result)
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    session: &mut Session,
) -> Result<()> {
    while !session.should_quit() {
        terminal.draw(|frame| ui::render(frame, &session.render_state()))?;

        if let Event::Key(key) = event::read()? {
            if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                session.handle_key(key);
            }
        }
    }

    Ok(())
}

#[derive(Default)]
struct TerminalLifecycle {
    raw_mode_enabled: bool,
    alt_screen_enabled: bool,
}

impl TerminalLifecycle {
    fn mark_raw_mode_enabled(&mut self) {
        self.raw_mode_enabled = true;
    }

    fn mark_alt_screen_enabled(&mut self) {
        self.alt_screen_enabled = true;
    }

    fn restore<LeaveAlt, DisableRaw>(
        &self,
        leave_alt: &mut LeaveAlt,
        disable_raw: &mut DisableRaw,
    ) -> Result<()>
    where
        LeaveAlt: FnMut() -> io::Result<()>,
        DisableRaw: FnMut() -> io::Result<()>,
    {
        let mut first_error = None;

        if self.alt_screen_enabled {
            if let Err(error) = leave_alt() {
                first_error = Some(error);
            }
        }

        if self.raw_mode_enabled {
            if let Err(error) = disable_raw() {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }

        match first_error {
            Some(error) => Err(error.into()),
            None => Ok(()),
        }
    }
}

fn restore_terminal(lifecycle: &TerminalLifecycle) -> Result<()> {
    lifecycle.restore(
        &mut || {
            let mut stdout = io::stdout();
            crossterm::execute!(stdout, LeaveAlternateScreen)
        },
        &mut || disable_raw_mode(),
    )
}

fn show_cursor() -> Result<()> {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, Show)?;
    Ok(())
}

fn finalize_terminal_result(
    primary: Result<()>,
    restore_result: Result<()>,
    cursor_result: Result<()>,
) -> Result<()> {
    match primary {
        Err(error) => Err(append_followup_error(
            append_followup_error(error, "terminal restore also failed", restore_result.err()),
            "cursor restoration also failed",
            cursor_result.err(),
        )),
        Ok(()) => match restore_result {
            Err(error) => Err(append_followup_error(
                error,
                "cursor restoration also failed",
                cursor_result.err(),
            )),
            Ok(()) => cursor_result,
        },
    }
}

fn append_followup_error(
    error: anyhow::Error,
    label: &str,
    followup: Option<anyhow::Error>,
) -> anyhow::Error {
    match followup {
        Some(followup) => anyhow!("{error:#}\n{label}: {followup:#}"),
        None => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn terminal_restore_attempts_all_entered_cleanup_steps_even_after_failure() {
        let mut state = TerminalLifecycle::default();
        state.mark_raw_mode_enabled();
        state.mark_alt_screen_enabled();

        let calls = RefCell::new(Vec::new());
        let result = state.restore(
            &mut || {
                calls.borrow_mut().push("leave_alt");
                Err(io::Error::other("leave failed"))
            },
            &mut || {
                calls.borrow_mut().push("disable_raw");
                Ok(())
            },
        );

        assert!(result.is_err());
        assert_eq!(calls.into_inner(), vec!["leave_alt", "disable_raw"]);
    }

    #[test]
    fn terminal_restore_skips_steps_that_were_never_enabled() {
        let state = TerminalLifecycle::default();
        let calls = RefCell::new(Vec::new());

        let result = state.restore(
            &mut || {
                calls.borrow_mut().push("leave_alt");
                Ok(())
            },
            &mut || {
                calls.borrow_mut().push("disable_raw");
                Ok(())
            },
        );

        assert!(result.is_ok());
        assert!(calls.into_inner().is_empty());
    }
}
