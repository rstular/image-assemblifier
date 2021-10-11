use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

use rand::seq::SliceRandom;
use rand::thread_rng;
use web_sys::console;

const COLOR_STEPS: &'static [u8] = &[0, 95, 135, 175, 215, 255];

#[wasm_bindgen(getter_with_clone)]
pub struct ConversionResult {
    pub status: i32,
    pub message: String,
}

struct ASMEntry {
    color_bg: u8,
    color_fg: u8,
    repeat: u8,
    character: u8,
}

#[wasm_bindgen]
pub fn generate_raw_data(
    image_data_js: &js_sys::Uint8Array,
    width: i32,
    height: i32,
) -> ConversionResult {
    let data_len: usize;
    match usize::try_from(image_data_js.length()) {
        Ok(v) => data_len = v,
        Err(_e) => {
            return ConversionResult {
                status: -1,
                message: String::from("Could not convert array length to usize!"),
            }
        }
    }

    let mut js: JsValue = width.into();
    console::log_2(&"Image width: ".into(), &js);
    js = height.into();
    console::log_2(&"Image height: ".into(), &js);
    js = data_len.into();
    console::log_2(&"Data size: ".into(), &js);

    // Convert UInt8Array to vector
    let img_data: Vec<u8> = image_data_js.to_vec();
    // Initialize array for Assembly entries
    let mut img_asm_entries: Vec<ASMEntry> = Vec::new();

    // Width datapoints
    let width_datapoints = (width as u32) * 4;

    // State-tracking variables
    let mut last_color: u8 = 0;
    let mut color_count: u8 = 0;
    let mut i: u32 = 0;

    loop {
        let mut terminal_color: u8 = 16;
        // Calculate terminal color
        terminal_color += get_closest_color(img_data[i as usize]) * 36;
        terminal_color += get_closest_color(img_data[i as usize + 1]) * 6;
        terminal_color += get_closest_color(img_data[i as usize + 2]);

        // Check if it is the same color as before
        if terminal_color == last_color {
            // If so, check if we reached the repetition limit
            if color_count == 255 {
                // If so, add an entry and begin a new one
                img_asm_entries.push(ASMEntry {
                    color_bg: last_color,
                    color_fg: if last_color == 0 { 1 } else { 0 },
                    repeat: color_count,
                    character: 0x20,
                });
                color_count = 0;
            }

            // Increase repetitions by 1
            color_count += 1;
        } else {
            // Check if we need to add an entry
            if color_count != 0 {
                img_asm_entries.push(ASMEntry {
                    color_bg: last_color,
                    color_fg: if last_color == 0 { 1 } else { 0 },
                    repeat: color_count,
                    character: 0x20,
                });
            }

            // Begin tracking the new color
            last_color = terminal_color;
            color_count = 1;
        }

        // Move to the next pixel
        i += 4;

        // Check if we reached the end of the line
        if i % width_datapoints == 0 {
            if color_count != 0 {
                img_asm_entries.push(ASMEntry {
                    color_bg: last_color,
                    color_fg: if last_color == 0 { 1 } else { 0 },
                    repeat: color_count,
                    character: 0x20,
                });
            }

            img_asm_entries.push(ASMEntry {
                color_bg: 0,
                color_fg: 0,
                repeat: 1,
                character: 0x0a,
            });

            last_color = 0;
            color_count = 0;
        }

        // Check for exit condition
        if i as usize == data_len {
            break;
        }
    }

    if (img_asm_entries.len() as i64) > i64::pow(2, 32) {
        return ConversionResult {
            status: -1,
            message: String::from("Resulting image is too big!"),
        };
    }

    // let mut OUT_ARRAY: js_sys::BigUint64Array =
    //     js_sys::BigUint64Array::new_with_length(img_asm_entries.len() as u32);
    let mut shuffle_mask: Vec<u32> = (1..(img_asm_entries.len() as u32)).collect();
    shuffle_mask.shuffle(&mut thread_rng());
    shuffle_mask.insert(0, 0);

    let mut out_values: Vec<u64> = vec![0; img_asm_entries.len()];

    let mut prev_idx: u32 = 0;
    for entry in img_asm_entries.iter() {
        let mut out_entry: u64 = entry.color_bg as u64;
        let next_idx: u32 = shuffle_mask.pop().unwrap();

        out_entry <<= 8;
        out_entry |= entry.color_fg as u64;
        out_entry <<= 32;
        out_entry |= next_idx as u64;
        out_entry <<= 8;
        out_entry |= entry.repeat as u64;
        out_entry <<= 8;
        out_entry |= entry.character as u64;

        out_values[prev_idx as usize] = out_entry;

        prev_idx = next_idx;
    }

    let mut output_string: String = String::from(".section .text\nMESSAGE:\n");
    for entry in out_values {
        output_string.push_str(format!("    .quad {:#018x}\n", entry).as_str());
    }

    ConversionResult {
        status: 0,
        message: output_string,
    }
}

fn get_closest_color(target: u8) -> u8 {
    let mut closest_diff: i16 = 256;
    let mut closest: u8 = 0;
    for (i, n) in COLOR_STEPS.iter().enumerate() {
        if ((n - target) as i16).abs() < closest_diff {
            closest_diff = ((n - target) as i16).abs();
            closest = i as u8;
        }
    }
    return closest;
}
