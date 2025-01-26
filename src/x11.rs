// x11 = "0.24"
extern crate x11;

use x11::xlib::*;
use std::ffi::CString;
use std::os::raw::c_char;

fn main() {
    // Open display connection
    let dpy = unsafe { XOpenDisplay(std::ptr::null()) };
    if dpy.is_null() {
        eprintln!("Cannot open X display");
        return;
    }

    // Get the default screen and root window
    let screen = unsafe { DefaultScreenOfDisplay(dpy) };
    let root = unsafe { RootWindow(dpy, *screen) };

    // Create a font (you'll need to have some fonts available on your system)
    let font_name = CString::new("fixed").expect("CString creation failed");
    let font_set = unsafe { XLoadQueryFont(dpy, font_name.as_ptr()) };
    if font_set.is_null() {
        eprintln!("Cannot load font");
        unsafe { XCloseDisplay(dpy) };
        return;
    }

    // Set the foreground and background colors
    let fg_color = 0xFFFFFF; // White
    let bg_color = 0x000000; // Black
    unsafe {
        XSetForeground(dpy, DefaultGCOfScreen(screen), fg_color);
        XSetBackground(dpy, DefaultGCOfScreen(screen), bg_color);
    }

    // Create a graphics context with the font settings
    let gc = unsafe { XCreateGC(dpy, root, 0, std::ptr::null_mut()) };
    if gc.is_null() {
        eprintln!("Cannot create GC");
        unsafe {
            XUnloadFont(dpy, font_set);
            XCloseDisplay(dpy);
        }
        return;
    }

    // Draw the text on the root window
    let text = CString::new("Hello, World!").expect("CString creation failed");
    let x = 10;
    let y = 50;
    unsafe {
        XDrawString(dpy, root, gc, x as c_int, y as c_int,
                     text.as_ptr() as *const c_char, text.to_bytes().len() as i32);
    }

    // Flush the display to ensure the changes are applied
    unsafe { XFlush(dpy) };

    // Clean up resources
    unsafe {
        XFreeGC(dpy, gc);
        XUnloadFont(dpy, font_set);
        XCloseDisplay(dpy);
    }
}
