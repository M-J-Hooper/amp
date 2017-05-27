use models::application::modes::JumpMode;
use commands::{Command, application, jump_mode};
use input::Key;

pub fn handle(mode: &mut JumpMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Char(c) => Some(jump_mode::push_search_char),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
