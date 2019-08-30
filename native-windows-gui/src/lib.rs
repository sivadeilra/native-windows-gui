#[macro_use]
extern crate bitflags;
extern crate winapi;

use std::rc::Rc;

#[cfg(test)]
mod tests;

mod errors;
pub use errors::SystemError;

mod events;
pub use events::Event;

pub(crate) mod win32;
pub use win32::{dispatch_thread_events, stop_thread_dispatch, enable_visual_styles, window::bind_event_handler,
 message_box::{MessageButtons, MessageIcons, MessageChoice, MessageParams, message, fatal_message, error_message, simple_message}};

mod controls;
pub use controls::*;


pub trait NativeUi<D, UI> {
    fn build_ui(d: D) -> Result<Rc<UI>, SystemError>;
}
