use crossterm::{
    cursor, 
    execute, 
    event,
    event::{read, KeyCode, KeyEvent, Event::*}, 
    terminal,
    terminal::ClearType,
    queue,
    Result
};

use std::io::Write;
use std::io::stdout;
use std::time::Duration; 

const VERSION: &str = "1.0";

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) { 
        terminal::disable_raw_mode().expect("ERROR: Could Not Disable Raw Mode"); 
        Output::clear_screen().expect("ERROR: Failed To Clear Screen");
    }
}

struct CursorController
{
    cursor_x: usize,
    cursor_y: usize,
    screen_columns: usize,
    screen_rows: usize,
}

impl CursorController
{
    fn new(window_size: (usize, usize)) -> CursorController
    {
        Self 
        {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: window_size.0,
            screen_rows: window_size.1,
        }
    }

    fn move_cursor(&mut self, direction: KeyCode)
    {
        match direction 
        {
            KeyCode::Up => 
            {
                if self.cursor_y > 0 
                {
                    self.cursor_y -= 1; 
                }
            }
            KeyCode::Down => 
            { 
                if self.cursor_y < self.screen_columns
                {
                    self.cursor_y += 1; 
                }
            }
            KeyCode::Left => 
            { 
                if self.cursor_x > 0
                {
                    self.cursor_x -= 1; 
                }
            }
            KeyCode::Right => 
            { 
                if self.cursor_x < self.screen_columns
                {
                    self.cursor_y += 1; 
                }
            }
            _ => unimplemented!(), 
        }
    }
}

struct Output
{
    window_size: (usize, usize),
    editor_contents: EditorContents, 
    cursor_controller: CursorController,
}

impl Output
{
    fn new() -> Self 
    {
        let window_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self
        { 
            window_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(window_size),
        }
    }
    
    fn clear_screen() -> crossterm::Result<()>
    {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self)
    {
        let screen_rows = self.window_size.1;
        let screen_columns = self.window_size.0;
        for i in 0..screen_rows 
        {
            if i == screen_rows / 3 
            {
                let mut welcome : String = format!("Pound Editor --- Version {}", VERSION);
                if welcome.len() > screen_columns 
                {
                    welcome.truncate(screen_columns)
                }

                let mut padding = (screen_columns - welcome.len()) / 2;
                if padding != 0 
                {
                    self.editor_contents.push('~');
                    padding -= 1
                }

                (0..padding).for_each(|_| self.editor_contents.push(' '));
                self.editor_contents.push_str(&welcome);
            } 
            else 
            {
                self.editor_contents.push('~');
            }

            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();

            if i < screen_rows - 1 
            {
                self.editor_contents.push_str("\r\n");
            }
       }
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()>
    {
        queue!(
            self.editor_contents,
            cursor::Hide,
            terminal::Clear(ClearType::All), 
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y;
        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self, direction: KeyCode)
    {
        self.cursor_controller.move_cursor(direction);
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

    fn process_keypress(&mut self) -> crossterm::Result<bool>
    {
        match self.reader.read_key()?
        {
            // process quit
            KeyEvent
            {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),

            // process cursor movement
            KeyEvent
            {
                code: direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right ),
                modifiers: event::KeyModifiers::NONE,
            } => self.output.move_cursor(direction),
            
            // else do nothing
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool>
    {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

struct EditorContents
{
    content: String,
}

impl EditorContents
{
    fn new() -> Self
    {
        Self 
        {
            content: String::new(),
        }
    }

    fn push(&mut self, ch:char)
    {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str)
    {
        self.content.push_str(string)
    }
}

impl std::io::Write for EditorContents 
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
    {
        match std::str::from_utf8(buf)
        {
            Ok(s) => 
            {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(std::io::ErrorKind::WriteZero.into()),
        }
    }
    fn flush(&mut self) -> std::io::Result<()>
    {
        let out = write!(stdout(), "{}", self.content);
        std::io::stdout().flush()?;
        self.content.clear();
        out
    }
}

fn main() -> Result<()>
{
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    while editor.run()?{}
    Ok(())
}
