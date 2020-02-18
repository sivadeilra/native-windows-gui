use winapi::um::winuser::{WS_VISIBLE, WS_DISABLED, BS_ICON, BS_BITMAP};
use crate::win32::window_helper as wh;
use crate::win32::resources_helper as rh;
use crate::{NwgError, Font, Bitmap, Icon};
use super::{ControlBase, ControlHandle};

const NOT_BOUND: &'static str = "Button is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: Button handle is not HWND!";


bitflags! {
    /**
        The button flags

        * NONE:     No flags. Equivalent to a invisible blank button.
        * VISIBLE:  The button is immediatly visible after creation
        * DISABLED: The button cannot be interacted with by the user. It also has a grayed out look.
        * BITMAP: The button will display a bitmap image with no text. Must have a bitmap or else it will only show text.
        * ICON: The button will display a icon image with no text. Must have a icon or else it will only show text.
    */
    pub struct ButtonFlags: u32 {
        const NONE = 0;
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const ICON = BS_ICON;
        const BITMAP = BS_BITMAP;
    }
}

/**
A push button is a rectangle containing an application-defined text label.
Use `ImageButton` if you need to have a button that ONLY contains an icon or a bitmap.

Button is not behind any features.

**Builder parameters:**
  * `parent`:   **Required.** The button parent container.
  * `text`:     The button text.
  * `size`:     The button size.
  * `position`: The button position.
  * `enabled`:  If the button can be used by the user. It also has a grayed out look if disabled.
  * `flags`:    A combination of the ButtonFlags values.
  * `font`:     The font used for the button text
  * `bitmap`:   A bitmap to display next to the button text. If this value is set, icon is ignored.
  * `icon`:     An icon to display next to the button text

**Control events:**
  * `OnButtonClick`: When the button is clicked once by the user
  * `OnButtonDoubleClick`: When the button is clicked twice rapidly by the user
  * `MousePress(_)`: Generic mouse press events on the button
  * `OnMouseMove`: Generic mouse mouse event

```rust
use native_windows_gui as nwg;
fn build_button(button: &mut nwg::Button, window: &nwg::Window, font: &nwg::Font) {
    nwg::Button::builder()
        .text("Hello")
        .flags(nwg::ButtonFlags::VISIBLE)
        .font(Some(font))
        .parent(window)
        .build(button);
}
```

*/
#[derive(Default, Eq, PartialEq)]
pub struct Button {
    pub handle: ControlHandle
}

impl Button {

    pub fn builder<'a>() -> ButtonBuilder<'a> {
        ButtonBuilder {
            text: "Button",
            size: (100, 25),
            position: (0, 0),
            enabled: true,
            flags: None,
            font: None,
            parent: None,
            bitmap: None,
            icon: None
        }
    }

    /// Simulate a user click
    pub fn click(&self) {
        use winapi::um::winuser::{BM_CLICK};

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        wh::send_message(handle, BM_CLICK, 0, 0);
    }

    /// Sets the bitmap image of the button. Replace the current bitmap or icon.
    /// Set `image` to `None` to remove the image
    pub fn set_bitmap<'a>(&self, image: Option<&'a Bitmap>) {
        use winapi::um::winuser::{BM_SETIMAGE, IMAGE_BITMAP};
        use winapi::shared::minwindef::{WPARAM, LPARAM};

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let image_handle = image.map(|i| i.handle as LPARAM).unwrap_or(0);
        wh::send_message(handle, BM_SETIMAGE, IMAGE_BITMAP as WPARAM, image_handle);
    }

    /// Sets the bitmap image of the button. Replace the current bitmap or icon.
    /// Set `image` to `None` to remove the image
    pub fn set_icon<'a>(&self, image: Option<&'a Icon>) {
        use winapi::um::winuser::{BM_SETIMAGE, IMAGE_ICON};
        use winapi::shared::minwindef::{WPARAM, LPARAM};

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let image_handle = image.map(|i| i.handle as LPARAM).unwrap_or(0);
        wh::send_message(handle, BM_SETIMAGE, IMAGE_ICON as WPARAM, image_handle);
    }

    /// Returns the current image in the button.
    /// If the button has a bitmap, the value will be returned in `bitmap`
    /// If the button has a icon, the value will be returned in `icon`
    pub fn image<'a>(&self, bitmap: &mut Option<Bitmap>, icon: &mut Option<Icon>) {
        use winapi::um::winuser::{BM_GETIMAGE, IMAGE_BITMAP, IMAGE_ICON};
        use winapi::shared::minwindef::WPARAM;
        use winapi::shared::windef::HBITMAP;
        use winapi::um::winnt::HANDLE;

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let bitmap_handle = wh::send_message(handle, BM_GETIMAGE, IMAGE_BITMAP as WPARAM, 0);
        let icon_handle = wh::send_message(handle, BM_GETIMAGE, IMAGE_ICON as WPARAM, 0);

        *bitmap = None;
        *icon = None;

        if bitmap_handle != 0 && rh::is_bitmap(bitmap_handle as HBITMAP) {
            *bitmap = Some(Bitmap { handle: bitmap_handle as HANDLE, owned: false });
        } else if icon_handle != 0 {
            *icon = Some(Icon { handle: icon_handle as HANDLE, owned: false });
        }
    }

    /// Returns the font of the control
    pub fn font(&self) -> Option<Font> {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let font_handle = wh::get_window_font(handle);
        if font_handle.is_null() {
            None
        } else {
            Some(Font { handle: font_handle })
        }
    }

    /// Sets the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_font(handle, font.map(|f| f.handle), true); }
    }

    /// Returns true if the control currently has the keyboard focus
    pub fn focus(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_focus(handle) }
    }

    /// Sets the keyboard focus on the button.
    pub fn set_focus(&self) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_focus(handle); }
    }

    /// Returns true if the control user can interact with the control, return false otherwise
    pub fn enabled(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_enabled(handle) }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_enabled(handle, v) }
    }

    /// Returns true if the control is visible to the user. Will return true even if the 
    /// control is outside of the parent client view (ex: at the position (10000, 10000))
    pub fn visible(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_visibility(handle) }
    }

    /// Show or hide the control to the user
    pub fn set_visible(&self, v: bool) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_visibility(handle, v) }
    }

    /// Returns the size of the button in the parent window
    pub fn size(&self) -> (u32, u32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Sets the size of the button in the parent window
    pub fn set_size(&self, x: u32, y: u32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_size(handle, x, y, false) }
    }

    /// Returns the position of the button in the parent window
    pub fn position(&self) -> (i32, i32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Sets the position of the button in the parent window
    pub fn set_position(&self, x: i32, y: i32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y) }
    }

    /// Returns the button label
    pub fn text(&self) -> String { 
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_text(handle) }
    }

    /// Sets the button label
    pub fn set_text<'a>(&self, v: &'a str) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_text(handle, v) }
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        "BUTTON"
    }

    /// Winapi base flags used during window creation
    pub fn flags(&self) -> u32 {
        ::winapi::um::winuser::WS_VISIBLE
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        use winapi::um::winuser::{BS_NOTIFY, WS_CHILD};

        BS_NOTIFY | WS_CHILD
    }

}

pub struct ButtonBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    flags: Option<ButtonFlags>,
    font: Option<&'a Font>,
    bitmap: Option<&'a Bitmap>,
    icon: Option<&'a Icon>,
    parent: Option<ControlHandle>
}

impl<'a> ButtonBuilder<'a> {

    pub fn flags(mut self, flags: ButtonFlags) -> ButtonBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn text(mut self, text: &'a str) -> ButtonBuilder<'a> {
        self.text = text;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> ButtonBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> ButtonBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn enabled(mut self, e: bool) -> ButtonBuilder<'a> {
        self.enabled = e;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> ButtonBuilder<'a> {
        self.font = font;
        self
    }

    pub fn bitmap(mut self, bit: Option<&'a Bitmap>) -> ButtonBuilder<'a> {
        self.bitmap = bit;
        self
    }

    pub fn icon(mut self, ico: Option<&'a Icon>) -> ButtonBuilder<'a> {
        self.icon = ico;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> ButtonBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut Button) -> Result<(), NwgError> {
        let flags = self.flags.map(|f| f.bits()).unwrap_or(out.flags());

        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("Button"))
        }?;

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            .size(self.size)
            .position(self.position)
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        if self.font.is_some() {
            out.set_font(self.font);
        }

        out.set_enabled(self.enabled);

        if self.bitmap.is_some() {
            out.set_bitmap(self.bitmap);
        } else if self.icon.is_some() {
            out.set_icon(self.icon);
        }

        Ok(())
    }

}
