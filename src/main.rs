use crossterm::{
    cursor, 
    execute, 
    event,
    event::{read, KeyCode, KeyEvent, Event::*}, 
    terminal,
    terminal::ClearType,
    Result
};

use std::io::stdout;
use std::time::Duration; 

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) { 
        terminal::disable_raw_mode().expect("ERROR: Could Not Disable Raw Mode"); 
        Output::clear_screen().expect("ERROR: Failed To Clear Screen");
    }
}

struct Output
{
    window_size: (usize, usize),
}

impl Output
{
    fn new() -> Self 
    {
        let window_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self{ window_size }
    }

    fn clear_screen() -> crossterm::Result<()>
    {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&self)
    {
        for _ in 0..self.window_size.1 
        {
            println!("~\r");
        }
    }

    fn refresh_screen(&self) -> crossterm::Result<()>
    {
        Self::clear_screen()?;
        self.draw_rows();
        execute!(stdout(), cursor::MoveTo(0, 0))
    }
}

struct Reader;

impl Reader 
{
    fn read_key(&self) -> crossterm::Result<KeyEvent>
    {
        loop 
        {
            if event::poll(Duration::from_millis(500))?
            {
                if let Key(event) = read()?
                {
                    return Ok(event);
                }
            }
        }
    }
}

struct Editor 
{
    reader: Reader,
    output: Output,
}
impl Editor
{
    fn new() -> Self 
    {
        Self
        {
            reader: Reader,
            output: Output::new(),
        }
    }

    fn process_keypress(&self) -> crossterm::Result<bool>
    {
        match self.reader.read_key()?
        {
            KeyEvent
            {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),
            _ => {}
        }
        Ok(true)
    }

    fn run(&self) -> crossterm::Result<bool>
    {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

fn main() -> Result<()>
{
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let editor = Editor::new();
    while editor.run()?{}
    Ok(())
}
