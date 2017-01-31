use tempfile;
use std::fs::File;
use std::error::Error;
use std::{thread, cmp};
use std::io::{self, Write};
use byteorder::{WriteBytesExt, NativeEndian};
use std::sync::mpsc::{Sender, Receiver, channel};
use image::{GenericImage, Pixel, DynamicImage, FilterType};

use modules::Block;
use parse_input::{self, Config};

pub fn start_bar_creator(bar_img_out: Sender<(File, i32)>,
                         resize_in: Receiver<u32>)
                         -> Result<(), Box<Error>> {
    let mut output_width = 0;
    let (combined_out, combined_in) = channel();

    let mut config = parse_input::read_config()?;

    // Start interval notification callback for every block
    // This will spawn threads inside the start_interval methods
    for element in config.left_blocks
        .iter()
        .chain(config.center_blocks.iter())
        .chain(config.right_blocks.iter()) {
        element.start_interval(combined_out.clone());
    }

    // Combine interval with output_width
    {
        let combined_out = combined_out.clone();
        thread::spawn(move || {
            while let Ok(output_width) = resize_in.recv() {
                if combined_out.send(Some(output_width)).is_err() {
                    break;
                }
            }
        });
    }

    loop {
        match combined_in.recv() {
            Ok(combined) => {
                if let Some(width) = combined {
                    output_width = width;
                }

                if output_width > 0 {
                    let bar_img = create_bar_from_config(&mut config, output_width)?;
                    bar_img_out.send((img_to_file(bar_img)?, config.bar_height as i32))?;
                }
            }
            Err(_) => Err("Config or Resize channel disconnected.".to_owned())?,
        };
    }
}

fn create_bar_from_config(config: &mut Config, bar_width: u32) -> Result<DynamicImage, Box<Error>> {
    let mut bar_img = config.bg.resize_exact(bar_width, config.bar_height, FilterType::Triangle);

    if let Some(left_image) = combine_elements(&mut config.left_blocks, config.bar_height)? {
        combine_images(&mut bar_img, &left_image, 0);
    }

    if let Some(center_image) = combine_elements(&mut config.center_blocks, config.bar_height)? {
        let offset = bar_width / 2 - center_image.width() / 2;
        combine_images(&mut bar_img, &center_image, offset);
    }

    if let Some(right_image) = combine_elements(&mut config.right_blocks, config.bar_height)? {
        let offset = bar_width - right_image.width();
        combine_images(&mut bar_img, &right_image, offset);
    }

    Ok(bar_img)
}

fn combine_elements(blocks: &mut [Box<Block>],
                    bar_height: u32)
                    -> Result<Option<DynamicImage>, Box<Error>> {
    if blocks.is_empty() {
        Ok(None)
    } else {
        let images = blocks.iter_mut()
            .map(|block| block.render())
            .collect::<Result<Vec<DynamicImage>, Box<Error>>>()?;
        let width = images.iter().map(|img| img.width()).sum();
        let mut result_img = DynamicImage::new_rgba8(width, bar_height);

        let mut offset = 0;
        for image in images {
            combine_images(&mut result_img, &image, offset);
            offset += image.width();
        }

        Ok(Some(result_img))
    }
}

// Draws the second image on top of the first one with an x-offset
fn combine_images(first: &mut DynamicImage, second: &DynamicImage, offset: u32) {
    for x in offset..cmp::min(offset + second.width(), first.width()) {
        for y in 0..second.height() {
            let mut pixel = first.get_pixel(x, y);
            pixel.blend(&second.get_pixel(x - offset, y));
            first.put_pixel(x, y, pixel);
        }
    }
}

fn img_to_file(img: DynamicImage) -> Result<File, io::Error> {
    let mut tmp = tempfile::tempfile()?;

    for pixel in img.pixels() {
        let channels = pixel.2.channels();
        if channels.len() == 4 {
            let _ = tmp.write_u32::<NativeEndian>((0xFF << 24) + ((channels[0] as u32) << 16) +
                                                  ((channels[1] as u32) << 8) +
                                                  channels[2] as u32);
        }
    }

    let _ = tmp.flush();
    Ok(tmp)
}

#[cfg(test)]
use parse_input;
#[test]
fn render_block_prevent_escape_sequences() {
    let mut col = DynamicImage::new_rgba8(1, 1);
    col.put_pixel(0, 0, Rgba { data: [255, 0, 255, 255] });
    let block = ImageElement {
        bg_col: col.clone(),
        fg_col: col,
    };
    let font = parse_input::get_settings().font;

    let result = block.render(30, &font).unwrap();
    assert_eq!(result.get_pixel(0, 0), Rgba { data: [255, 0, 255, 255] });
}
