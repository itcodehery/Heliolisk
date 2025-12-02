mod buffer;
mod editor;

use crossterm::{
    event::{KeyEvent, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::{
    buffer::Buffer,
    editor::{Editor, NavigateMode},
};

fn main() {
    enable_raw_mode().unwrap();
    let mut alpha_buffer = Buffer::new();
    let mut editor = Editor::<NavigateMode>::new(vec![alpha_buffer]);

    loop {
        // editor.render();

        match read() {
            Ok(Key(event)) => {
                let action = editor.handle_input(event);
                match action {
                    EditorAction::Quit => break,
                    EditorAction::SaveAndQuit => {
                        // save logic
                        break;
                    }
                    EditorAction::EnterCommandMode => {
                        editor = editor.enter_command_mode();
                    }
                    EditorAction::ExitCommandMode => {
                        editor = editor.exit_command_mode();
                    }
                    EditorAction::EnterEditMode => {
                        editor = editor.enter_edit_mode();
                    }
                    EditorAction::ExitEditMode => {
                        editor = editor.exit_edit_mode();
                    }
                    EditorAction::None => {}
                }
            }
            Err(err) => println!("Error: {}", err),
            _ => (),
        }
    }

    disable_raw_mode().unwrap();
}
