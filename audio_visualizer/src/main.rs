use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::InputCallbackInfo;
use macroquad::prelude::*;

fn do_audio_stuff(
    left_sender: Sender<f32>,
    right_sender: Sender<f32>) {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("no default output device");

    let supported_config = device.default_input_config().unwrap();
    let config = supported_config.config();
    
    let channels = config.channels as usize;
    
    let handle_left = move |value: f32| {
        println!("Left channel value: {}", value);
        left_sender.send(value).unwrap();
    };

    let handle_right = move |value: f32| {
        println!("Right channel value: {}", value);
        right_sender.send(value).unwrap();
    };
    
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &InputCallbackInfo| {
            for frame in data.chunks(channels) {
                
                let mut channel = 0;

                for sample in frame.iter() {

                    let normalized_sample = (*sample * 10000.0).abs();

                    match channel {
                        0 => handle_left(normalized_sample),
                        1 => handle_right(normalized_sample),
                        _ => println!("What?")
                    }

                    channel = channel + 1;
                }
            }
        },
        move |err| {
            panic!("Something bad happened: {}", err);
        },
        None)
        .expect("Error building stream");

    stream.play().expect("Error playing stream");

    thread::sleep(std::time::Duration::from_millis(30000));
}

async fn do_graphics_stuff(
    left_val_rec: Receiver<f32>,
    right_val_rec: Receiver<f32>) {
    
    let mut left_bar_height = 0.0;
    let mut right_bar_height = 0.0;

    loop {
        match left_val_rec.try_recv() {
            Ok(value) => left_bar_height = value,
            Err(_) => println!("No left channel data.")
        }

        match right_val_rec.try_recv() {
            Ok(value) => right_bar_height = value,
            Err(_) => println!("No right channel data.")
        }

        clear_background(BLACK);

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

    let (left_val_send, left_val_rec) = mpsc::channel::<f32>();
    let (right_val_send, right_val_rec) = mpsc::channel::<f32>();

    thread::spawn(move || {
        do_audio_stuff(left_val_send, right_val_send);
    });
    
    do_graphics_stuff(left_val_rec, right_val_rec).await;
}
