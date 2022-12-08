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

use std::io::{Write, stdout};
use std::{cmp, env, fs, io};
use std::cmp::Ordering;
use std::time::Duration; 
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    row_offset: usize,
    column_offset: usize,
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
            row_offset: 0,
            column_offset: 0, 
        }
    }

    fn scroll(&mut self)
    {
        self.column_offset = cmp::min(self.column_offset, self.cursor_x);
        if self.cursor_x >= self.column_offset + self.screen_columns
        {
            self.column_offset = self.cursor_x - self.screen_columns + 1;
        }

        self.row_offset = cmp::min(self.row_offset, self.cursor_y);
        if self.cursor_y >= self.row_offset + self.screen_rows
        {
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }
    }

    fn move_cursor(&mut self, direction: KeyCode, editor_rows: &EditorRows)
    {
        let number_of_rows = editor_rows.number_of_rows();
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
                if self.cursor_y < number_of_rows
                {
                    self.cursor_y += 1; 
                }
            }
            KeyCode::Left => 
            {
                if self.cursor_x != 0
                {
                    self.cursor_x -= 1;
                }
                else if self.cursor_y > 0
                {
                    self.cursor_y -= 1;
                    self.cursor_x = editor_rows.get_row(self.cursor_y).len();
                }
            }
            KeyCode::Right => 
            {
                if self.cursor_y < number_of_rows
                {
                    match self.cursor_x.cmp(&editor_rows.get_row(self.cursor_y).len())
                    {
                        Ordering::Less => self.cursor_x += 1,
                        Ordering::Equal => {
                            self.cursor_y += 1;
                            self.cursor_x = 0
                        },
                        _ => {}
                    }
                }
            },
            KeyCode::End => self.cursor_y = number_of_rows - 1,
            KeyCode::Home => self.cursor_y = 0,

            _ => unimplemented!(), 
        }
        let row_len = if self.cursor_y < number_of_rows
        {
            editor_rows.get_row(self.cursor_y).len()
        }
        else 
        {
            0
        };
        self.cursor_x = cmp::min(self.cursor_x, row_len);
    }
}

struct EditorRows
{
    row_contents: Vec<Box<str>>,
}

impl EditorRows
{
    fn new() -> Self 
    {
        let mut arg = env::args();

        match arg.nth(1) 
        {
            None => Self {
                row_contents: Vec::new(),
            },
            
            Some(file) => Self::from_file(file.as_ref()),
        }
    }
    
    fn from_file(file: &Path) -> Self 
    {
        let file_contents = fs::read_to_string(file).expect("Unable to read file");
        Self 
        {
            row_contents: file_contents.lines().map(|it| it.into()).collect(),
        }
    }

    fn number_of_rows(&self) -> usize 
    {
        self.row_contents.len()
    }

    fn get_row(&self, at:usize) -> &str 
    {
        &self.row_contents[at] 
    }
}

struct Output
{
    window_size: (usize, usize),
    editor_contents: EditorContents, 
    cursor_controller: CursorController,
    editor_rows : EditorRows,
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
            editor_rows: EditorRows::new(),
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
            let file_row = i + self.cursor_controller.row_offset;
            if file_row >= self.editor_rows.number_of_rows() 
            {
                if self.editor_rows.number_of_rows() == 0 && i == screen_rows / 3 
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
            }
            else 
            {
                let row = self.editor_rows.get_row(file_row);
                let column_offset = self.cursor_controller.column_offset;
                let len = cmp::min(row.len().saturating_sub(column_offset), screen_columns);
                let start = if len == 0 { 0 } else { column_offset };
                self.editor_contents.push_str(&row[start..start + len])
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
        self.cursor_controller.scroll();
        
        queue!(
            self.editor_contents,
            cursor::Hide,
            terminal::Clear(ClearType::All), 
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();

        let cursor_x = self.cursor_controller.cursor_x - self.cursor_controller.column_offset;
        let cursor_y = self.cursor_controller.cursor_y - self.cursor_controller.row_offset;
        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x as u16, cursor_y as u16),
            cursor::Show
        )?;
        
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self, direction: KeyCode)
    {
        self.cursor_controller.move_cursor(direction, &self.editor_rows);
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
                code: direction @ (KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End),
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
