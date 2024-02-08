use std::fmt::Debug;
use std::num::NonZeroU64;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, Device, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        let projection_matrix = OPENGL_TO_WGPU_MATRIX * proj * view;

        return projection_matrix;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        Self {
            view_proj: camera.build_view_projection_matrix().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

impl CameraState {
    pub fn new(device: &Device, (width, height): (u32, u32)) -> (Self, BindGroupLayout) {
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, -350.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new(&camera);
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(NonZeroU64::new(64).unwrap()),
                    },
                    count: None,
                }
            ],
        });
        let camera_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("camera_bind_group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
            });

        (Self {
            camera,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
        }, camera_bind_group_layout)
    }

    pub fn update_view_proj(&mut self) {
        self.camera_uniform.view_proj = self.camera.build_view_projection_matrix().into();
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_bind_group: BindGroup,
    pub camera_buffer: Buffer,
}