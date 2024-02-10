use image::{ImageBuffer, Rgba};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferAddress, Color, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Device, FragmentState, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, include_wgsl, LoadOp, Maintain, Operations, Origin3d, PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StencilState, StoreOp, Texture, TextureAspect, TextureDescriptor, TextureSampleType, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use wgpu::TextureFormat::Rgba8Unorm;
use crate::{HEIGHT, PADDED_BYTES_PER_ROW, UNPADDED_BYTES_PER_ROW, WIDTH};
use crate::camera::CameraState;

pub struct RenderState {
    pub texture_desc: TextureDescriptor<'static>,
    pub texture_view: TextureView,
    pub output_buffer: Buffer,
    pub texture: Texture,

    pub camera_state: CameraState,

    pub render_pipeline: RenderPipeline,

    pub output_texture: Texture,
    pub output_texture_view: TextureView,

    pub color_pass: RenderPipeline,
    pub color_bind_group: BindGroup,
}

impl RenderState {
    pub fn new(device: &Device) -> RenderState {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };

        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("Texture View"),
            format: Some(Rgba8Unorm),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let out_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
            ,
            label: None,
            view_formats: &[],
        };

        let out_texture = device.create_texture(&out_texture_desc);
        let out_texture_view = out_texture.create_view(&TextureViewDescriptor {
            label: Some("Out Texture View"),
            format: Some(Rgba8Unorm),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let out_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        let output_buffer_size = (PADDED_BYTES_PER_ROW * HEIGHT) as wgpu::BufferAddress;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let (camera_state, camera_bind_group_layout) = CameraState::new(device, (WIDTH, HEIGHT));

        let pipeline_layout = device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(include_wgsl!("rend_shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    //todo fix
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }
                        ],
                    }
                ],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(
                FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(ColorTargetState {
                            format: Rgba8Unorm,
                            blend: Some(BlendState{
                                color: BlendComponent{
                                    src_factor: BlendFactor::Zero,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Add,
                                },
                                alpha: BlendComponent{
                                    src_factor: BlendFactor::One,
                                    dst_factor: BlendFactor::One,
                                    operation: BlendOperation::Add,
                                },
                            }),
                            write_mask: ColorWrites::ALL,
                        })
                    ],
                }
            ),
            multiview: None,
        });


        let color_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let color_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Color bind group"),
            layout: &color_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&out_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&out_texture_sampler),
                },
            ],
        });

        let color_pipeline_layout = device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &color_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });


        let color_shader = device.create_shader_module(include_wgsl!("color_shader.wgsl"));

        let color_pass = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Color render Pipeline"),
            layout: Some(&color_pipeline_layout),
            vertex: VertexState {
                module: &color_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(
                FragmentState {
                    module: &color_shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(ColorTargetState {
                            format: Rgba8Unorm,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })
                    ],
                }
            ),
            multiview: None,
        });


        Self {
            render_pipeline,

            output_buffer,
            texture_view,
            texture_desc,
            texture,
            camera_state,

            color_pass,
            color_bind_group,

            output_texture: out_texture,
            output_texture_view: out_texture_view,
        }
    }

    pub fn render(&mut self, device: &Device, queue: &Queue, input_buffer: &Buffer, input_len: u32) {
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &self.texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::TRANSPARENT),
                            store: StoreOp::Store,
                        },
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_state.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, input_buffer.slice(..));
            render_pass.draw(0..3, 0..input_len);

            drop(render_pass);
        }

        encoder.copy_texture_to_texture(ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        }, ImageCopyTexture {
            texture: &self.output_texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        }, self.texture.size());


        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &self.texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.color_pass);
            render_pass.set_bind_group(0, &self.color_bind_group, &[]);
            render_pass.draw(0..3, 0..2);

            drop(render_pass);
        }

        encoder.copy_texture_to_buffer(ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        }, ImageCopyBuffer {
            buffer: &self.output_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(PADDED_BYTES_PER_ROW),
                rows_per_image: Some(HEIGHT as u32),
            },
        }, self.texture_desc.size);

        let submission_index = queue.submit(Some(encoder.finish()));
        device.poll(Maintain::wait_for(submission_index));
        return;
    }

    pub async fn save_buffer_to_image(&self, filename: u32, device: &Device) {
        {
            let buffer_slice = self.output_buffer.slice(..);
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            // wait for the GPU to finish
            device.poll(wgpu::Maintain::Wait);

            match rx.receive().await {
                Some(Ok(())) => {
                    let padded_data = buffer_slice.get_mapped_range();
                    let data = padded_data
                        .chunks(PADDED_BYTES_PER_ROW as _)
                        .map(|chunk| &chunk[..UNPADDED_BYTES_PER_ROW as _])
                        .flatten()
                        .map(|x| *x)
                        .collect::<Vec<_>>();
                    drop(padded_data);
                    self.output_buffer.unmap();

                    let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(WIDTH, HEIGHT, data).unwrap();
                    image_buffer.save(format!("output/{:0>5}.jpeg", filename)).unwrap();
                    // println!("{filename}");
                }
                _ => eprintln!("Something went wrong"),
            }
        }
    }
}

// pub fn to_linear_rgb(c: Color) -> [f32; 3] {
//     let f = |xu: u32| {
//         let x = (xu & 0xFF) as f32 / 255.0;
//         if x > 0.04045 {
//             ((x + 0.055) / 1.055).powf(2.4)
//         } else {
//             x / 12.92
//         }
//     };
//     [f(c >> 16), f(c >> 8), f(c)]
// }
