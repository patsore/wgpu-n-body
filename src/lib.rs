#![feature(portable_simd)]

mod sim;
mod renderer;
mod camera;

pub use sim::*;
use wgpu::{Backends, Device, DeviceDescriptor, Features, InstanceDescriptor, InstanceFlags, PowerPreference, Queue, RequestAdapterOptions};
use crate::renderer::RenderState;


pub struct State {
    pub device: Device,
    pub queue: Queue,

    pub sim_state: SimState,
    pub render_state: RenderState,
}

const WIDTH: u32 = 1080;
const HEIGHT: u32 = 1920;

const PIXEL_SIZE: usize = std::mem::size_of::<[u8; 4]>();
const ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
const UNPADDED_BYTES_PER_ROW: u32 = PIXEL_SIZE as u32 * WIDTH;
const PADDING: u32 = (ALIGN - UNPADDED_BYTES_PER_ROW % ALIGN) % ALIGN;
const PADDED_BYTES_PER_ROW: u32 = UNPADDED_BYTES_PER_ROW + PADDING;


impl State {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::default(),
            ..Default::default()
        });


        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: None,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Default::default(),
            },
            None,
        ).await.unwrap();


        let render_state = RenderState::new(&device);

        let sim_state = SimState::new(&device);

        Self {
            device,
            queue,

            render_state,
            sim_state,
        }
    }

    pub async fn render(&mut self, filename: u32) {
        self.render_state.render(&self.device, &self.queue, &self.sim_state.output_positions, self.sim_state.bodies.len() as u32);
        self.render_state.save_buffer_to_image(filename, &self.device).await;
    }

    pub async fn tick(&mut self) {
        return self.sim_state.tick(&self.device, &self.queue).await;
    }
}
