use color_eyre::{eyre::WrapErr, Result};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    terminal::Terminal,
};
use std::{io::stdout, time::Duration};

pub fn init() -> Result<Terminal<impl Backend>> {
    // this size is to match the size of the terminal when running the demo
    // using vhs in a 1280x640 sized window (github social preview size)
    // let options = TerminalOptions {
    //     viewport: Viewport::Fixed(Rect::new(0, 0, 81, 18)),
    // };
    // let terminal = Terminal::with_options(CrosstermBackend::new(io::stdout()), options)?;
    enable_raw_mode().context("enable raw mode")?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;

    stdout()
        .execute(EnterAlternateScreen)
        .wrap_err("enter alternate screen")?;
    Ok(terminal)
}

pub fn restore() -> Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    stdout()
        .execute(LeaveAlternateScreen)
        .wrap_err("leave alternate screen")?;
    Ok(())
}

pub fn next_event(timeout: Duration) -> Result<Option<Event>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    let event = event::read()?;
    Ok(Some(event))
}
