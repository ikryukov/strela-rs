use crossbeam_channel::Sender;
use tracing::debug;

extern crate nalgebra_glm as glm;

use crate::Settings;
use glm::{Mat4, Vec3};
use log::info;
use std::sync::{Arc, Mutex};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    color: [f32; 4],
}
impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color: [r, g, b, a],
        }
    }
}

pub struct PathTracerRenderContext {
    // scene: Arc<EmbreeScene>,
    image_data: Mutex<Vec<Color>>,
    result_width: u32,
    result_height: u32,
    view: Mat4,
    tx: Sender<Vec<Color>>,
    input_rx: single_value_channel::Receiver<Mat4>,
    settings: Arc<Mutex<Settings>>,
}
impl PathTracerRenderContext {
    pub fn new(
        width: u32,
        height: u32,
        // scene: Arc<EmbreeScene>,
        tx: Sender<Vec<Color>>,
        input_rx: single_value_channel::Receiver<Mat4>,
        settings: Arc<Mutex<Settings>>,
    ) -> Self {
        Self {
            result_height: height,
            result_width: width,
            view: Mat4::new_translation(&Vec3::new(0.0f32, 0.0f32, -1.0f32)),
            // scene,
            image_data: Mutex::new(vec![Color::default(); (width * height) as usize]),
            tx,
            input_rx,
            settings,
        }
    }
}

pub fn run_iteration(pt_ctx: &mut PathTracerRenderContext) {
    let camera_matrix = pt_ctx.input_rx.latest();
    info!("camera matrix: {}", camera_matrix);

    let settings = pt_ctx.settings.lock().unwrap();

    let bg_color = Color::new(
        settings.color[0],
        settings.color[1],
        settings.color[2],
        1.0f32,
    );
    let mut image_data = pt_ctx.image_data.lock().unwrap().clone();
    for i in 0..pt_ctx.result_height {
        for j in 0..pt_ctx.result_width {
            let mut col = Color::new(1.0f32, 1.0f32, 1.0f32, 1.0f32);
            if i == j {
                col = bg_color
            }
            image_data[(i * pt_ctx.result_width + j) as usize] = col;
        }
    }
    match pt_ctx.tx.try_send(image_data) {
        Ok(_) => {
            debug!("frame has sent");
        }
        Err(_) => {
            debug!("swapchain is full");
        }
    }
}
