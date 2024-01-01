use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::{InputCallbackInfo, OutputCallbackInfo};
use macroquad::prelude::*;

fn do_audio_stuff(
    left_val_container: Arc<Mutex<f32>>,
    right_val_container: Arc<Mutex<f32>>) {
    
    let host = cpal::host_from_id(cpal::HostId::Wasapi).expect("Failed to load host.");

    let device = host
        .default_output_device()
        .expect("No default output device.");

    let supported_config = device.default_output_config().unwrap();
    let config = supported_config.config();
    
    println!("config: {:?}", config);

    let channels = config.channels;
    
    let update_values = move |new_left: f32, new_right: f32| {
        println!("Left channel value: {}", new_left);
        println!("Right channel value: {}", new_right);
        
        let mut left_val = left_val_container.lock().unwrap();
        *left_val = new_left;
        
        let mut right_val = right_val_container.lock().unwrap();
        *right_val = new_right;
    };

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &InputCallbackInfo| {
            let frame_size = 800.0;
            let chunk_size = (channels as f32 * frame_size) as usize;

            for frame in data.chunks(chunk_size) {

                let left_avg = (frame[0] * 1000.0).abs();
                let right_avg = (frame[1] * 1000.0).abs();

                update_values(left_avg, right_avg);
            }
        },
        move |err| {
            panic!("Something bad happened: {}", err);
        },
        None)
        .expect("Error building stream.");

    stream.play().expect("Error playing stream.");

    thread::sleep(std::time::Duration::MAX);
}

async fn do_graphics_stuff(
    left_val_container: Arc<Mutex<f32>>,
    right_val_container: Arc<Mutex<f32>>) {
    
    clear_background(BLACK);
    
    loop {
        let left_bar_height = *left_val_container.lock().unwrap();
        let right_bar_height = *right_val_container.lock().unwrap();

        let right_bar_start_x = (screen_width() / 2.0) + 20.0;
        
        let bar_top_y = 50.0;
        let bar_width = 200.0;
        
        let left_bar_start_x = right_bar_start_x - 40.0 - bar_width;
        
        let max_bar_height = (screen_height() / 4.0) * 3.0;

        let left_bar_y = bar_top_y + (max_bar_height - left_bar_height);
        let right_bar_y = bar_top_y + (max_bar_height - right_bar_height);

        draw_rectangle(left_bar_start_x, left_bar_y, bar_width, left_bar_height, RED);
        draw_rectangle(right_bar_start_x, right_bar_y, bar_width, right_bar_height, RED);

        next_frame().await
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let left_val_container: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
    let right_val_container: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));

    let left_val_audio_container = Arc::clone(&left_val_container);
    let right_val_audio_container = Arc::clone(&right_val_container);

    let left_val_graphics_container = Arc::clone(&left_val_container);
    let right_val_graphics_container = Arc::clone(&right_val_container);

    thread::spawn(move || {
        do_audio_stuff(
            left_val_audio_container,
            right_val_audio_container);
    });
    
    do_graphics_stuff(
        left_val_graphics_container,
        right_val_graphics_container).await;
}
