#[macro_use]
extern crate wayland_client;
#[macro_use]
extern crate wayland_sys;
#[macro_use]
extern crate lazy_static;
extern crate byteorder;
extern crate tempfile;
extern crate rusttype;
extern crate image;
extern crate regex;
extern crate toml;

use std::thread;
use std::sync::mpsc::channel;

mod modules;
mod wayland;
mod create_bar;
mod parse_input;

// TODO: Logging instead of unwrapping
// TODO: Immortality -> Auto-Revive
fn main() {
    let (bar_img_out, bar_img_in) = channel();
    let (resize_out, resize_in) = channel();
    let (config_out, config_in) = channel();

    {
        thread::spawn(move || {
            create_bar::start_bar_creator(bar_img_out, resize_in, config_in).unwrap();
        });
    }

    {
        thread::spawn(move || {
            parse_input::read_config(&config_out).unwrap();
        });
    }

    wayland::start_wayland_panel(bar_img_in, resize_out).unwrap();
}
