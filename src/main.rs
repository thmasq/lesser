use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, stdout, Read, Write};
use std::time::Duration;
use textwrap::{wrap, Options};

struct TerminalState;

impl TerminalState {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = disable_raw_mode();
    }
}

fn print_contents(contents: &[u8]) -> io::Result<()> {
    let mut stdout = stdout();
    let mut offset = 0;

    let text = String::from_utf8_lossy(contents);
    let lines: Vec<&str> = text.lines().collect();
    let (mut width, mut height) = crossterm::terminal::size()?;

    let _terminal_state = TerminalState::enter()?;
    let mut last_offset: Option<usize> = None;

    loop {
        if last_offset != Some(offset) {
            execute!(stdout, Clear(ClearType::All))?;

            let options = Options::new(width as usize).break_words(false);
            let mut display_lines: Vec<String> = Vec::new();
            for line in lines.iter().skip(offset) {
                let wrapped_lines = wrap(line, &options);
                display_lines.extend(wrapped_lines.iter().enumerate().map(|(i, wrapped_line)| {
                    if i == 0 {
                        wrapped_line.to_string()
                    } else {
                        format!("↩ {wrapped_line}")
                    }
                }));
            }

            for (i, line) in display_lines.iter().take(height as usize).enumerate() {
                execute!(
                    stdout,
                    cursor::MoveTo(0, u16::try_from(i).unwrap_or_default())
                )?;
                stdout.write_all(line.as_bytes())?;
            }
            stdout.flush()?;
            last_offset = Some(offset);
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down if offset + (height as usize) < lines.len() => offset += 1,
                    KeyCode::Up if offset > 0 => offset -= 1,
                    KeyCode::Home => offset = 0,
                    KeyCode::End => {
                        if lines.len() > height as usize {
                            offset = lines.len() - height as usize;
                        } else {
                            offset = 0;
                        }
                    }
                    KeyCode::PageUp => offset = offset.saturating_sub(height as usize),
                    KeyCode::PageDown => {
                        offset = (offset + height as usize).min(lines.len() - height as usize);
                    }
                    _ => {}
                },
                Event::Mouse(MouseEvent { kind, .. }) => match kind {
                    MouseEventKind::ScrollDown if offset + (height as usize) < lines.len() => {
                        offset += 3;
                    }
                    MouseEventKind::ScrollUp if offset > 0 => offset -= 3,
                    _ => {}
                },
                Event::Resize(new_width, new_height) => {
                    width = new_width;
                    height = new_height;
                    last_offset = None;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut contents = Vec::new();
    io::stdin().read_to_end(&mut contents)?;

    print_contents(&contents)?;

    Ok(())
}
