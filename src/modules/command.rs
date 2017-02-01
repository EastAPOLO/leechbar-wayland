use std::cmp;
use std::thread;
use toml::Value;
use rusttype::Font;
use std::error::Error;
use std::time::Duration;
use std::process::Command;
use std::sync::mpsc::Sender;
use image::{DynamicImage, Rgba};

use modules::Block;
use parse_input::Config;
use modules::text::TextBlock;

pub struct CommandBlock {
    pub bar_height: u32,
    pub font_height: u32,
    pub font: Font<'static>,
    pub bg_col: DynamicImage,
    pub fg_col: Rgba<u8>,
    pub width: u32,
    pub spacing: u32,
    pub command: String,
    pub interval: u32,
}

// Unwraps cannot fail
impl CommandBlock {
    pub fn create(config: Config, value: &Value) -> Result<Box<Block>, Box<Error>> {
        let command = value.lookup("command").ok_or("Could not find command in a command module.")?;
        let command = command.as_str().ok_or("Command in command module is not a String.")?;
        let font_height = cmp::min(config.bar_height, config.font_height.unwrap());
        Ok(Box::new(CommandBlock {
            bar_height: config.bar_height,
            font_height: font_height,
            font: config.font.unwrap(),
            bg_col: config.bg,
            fg_col: config.fg,
            width: config.width,
            spacing: config.spacing,
            command: command.to_owned(),
            interval: config.interval,
        }))
    }
}

impl Block for CommandBlock {
    fn start_interval(&self, interval_out: Sender<Option<u32>>) {
        if self.interval > 0 {
            let interval = self.interval as u64;
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(interval));
                    interval_out.send(None).unwrap(); // TODO: Not unwrap?
                }
            });
        }
    }

    // TODO: Implement caching for Command
    fn render(&mut self) -> Result<DynamicImage, Box<Error>> {
        let output = Command::new("sh").arg("-c").arg(&self.command).output()?;
        let text = String::from_utf8_lossy(&output.stdout);

        let mut text_block = TextBlock {
            bar_height: self.bar_height,
            font_height: self.font_height,
            font: self.font.clone(),
            bg_col: self.bg_col.clone(),
            fg_col: self.fg_col,
            text: text.to_string(),
            width: self.width,
            spacing: self.spacing,
            cache: None,
        };

        text_block.render()
    }
}
