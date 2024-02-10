use std::f32::consts::PI;

use rand::distributions::Distribution;
use rand::{Rng, SeedableRng};
use rand_distr::Normal;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor, Device, include_wgsl, Maintain, PipelineLayoutDescriptor, Queue, ShaderStages};
use wgpu::util::{BufferInitDescriptor, DeviceExt};


pub struct SimState {
    pub input_masses: Buffer,
    pub positions_buffer: Buffer,

    pub output_positions: Buffer,

    pub compute_pipeline: ComputePipeline,
    pub bodies: Vec<Body>,
    pub input_bind_group: BindGroup,
}

impl SimState {
    pub fn new(device: &Device) -> Self {
        let mut bodies: Vec<Body> = gen_actual_spir_g([0.0, 32.5], [2.0, 6.0] ,50_000.0, 10_000, 2, true, 35.0);

        let mut bodies_1: Vec<Body> = gen_actual_spir_g([0.0, -32.5], [-2.0, -6.0] ,50_000.0, 10_000, 2, true, 35.0);

        bodies.append(&mut bodies_1);


        // let bodies = vec![
        //     Body::new(1000.0, [-3.0, 0.0], [0.0, -5.0]),
        //     Body::new(1000.0, [3.0, 0.0], [0.0, 5.0]),
        // ];

        let masses = bodies.iter().map(|b| {
            b.mass
        }).collect::<Vec<_>>();
        let positions = bodies.iter().map(|b| {
            b.position
        }).collect::<Vec<_>>();
        let velocities = bodies.iter().map(|b| {
            b.velocity
        }).collect::<Vec<_>>();


        let input_masses = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Body masses input buffer"),
            contents: bytemuck::cast_slice(masses.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let positions_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("GPU positions buffer"),
            contents: bytemuck::cast_slice(positions.as_slice()),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            // | wgpu::BufferUsages::VERTEX,
        });

        //todo initial velocities can be set through this buffer (unimplemented)
        let velocities_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("GPU-only velocities buffer"),
            contents: bytemuck::cast_slice(velocities.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });


        let output_positions = device.create_buffer(&BufferDescriptor {
            label: Some("Output positions buffer"),
            size: positions_buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let input_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Input bind group layout"),
            entries: &[
                //positions
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //masses
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                //the velocities buffer
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let input_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Input Bind Group"),
            layout: &input_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: positions_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_masses.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: velocities_buffer.as_entire_binding(),
                }
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Simulation pipeline layout"),
            bind_group_layouts: &[
                &input_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("comp_shader.wgsl"));

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Simulation compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            bodies,

            input_masses,
            positions_buffer,
            output_positions,

            input_bind_group,
            compute_pipeline,
        }
    }

    pub async fn tick(&mut self, device: &Device, queue: &Queue) { // -> Vec<[f32;2]> {
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None },
        );

        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_bind_group(0, &self.input_bind_group, &[]);
            compute_pass.set_pipeline(&self.compute_pipeline);
            //todo figure out better workgroup count
            compute_pass.dispatch_workgroups((self.bodies.len() as f32 / 256.0).ceil() as u32, 1, 1);

            drop(compute_pass);
        }
        encoder.copy_buffer_to_buffer(&self.positions_buffer, 0, &self.output_positions, 0, self.positions_buffer.size());

        let sub_index = queue.submit(Some(encoder.finish()));
        device.poll(Maintain::WaitForSubmissionIndex(sub_index));
    }
}

//struct to be passed to gpu
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Body {
    pub position: [f32; 2],
    pub mass: f32,
    pub velocity: [f32; 2],
}

impl Body {
    pub fn new(mass: f32, position: [f32; 2], velocity: [f32; 2]) -> Self {
        Self {
            mass,
            position,
            velocity,
        }
    }
}

//galaxies should be spinning counter-clockwise probably
pub fn gen_actual_spir_g(center_pos: [f32; 2], center_velocity: [f32;2], center_mass: f32, num_bodies: u32, num_arms: u32, clockwise: bool, radius: f32) -> Vec<Body> {
    let center_body = Body::new(center_mass, center_pos, center_velocity);

    let p_per_arm = (num_bodies / num_arms);


    const VEL_MULT: f32 = 30.0;
    //how closely to the center the arms will spin around.
    const ARM_ANGLE: f32 = 10.0;
    let arm_distance = (radius / p_per_arm as f32);

    let mut arms = (0..360).step_by(360 / num_arms as usize).map(|i| {
        let angle_rad = (i as f32).to_radians();
        (0..p_per_arm).map(move |j| {


            let angle_rad = ((i + j) as f32).to_radians();

            let tangent_angle = if clockwise {
                angle_rad + PI / 2.0 // Rotate clockwise by 90 degrees
            } else {
                angle_rad - PI / 2.0 // Rotate counterclockwise by 90 degrees
            };

            let radius = (j as f32) * arm_distance;
            let x = center_pos[0] + radius * angle_rad.cos();
            let y = center_pos[1] + radius * angle_rad.sin();
            let velocity = [tangent_angle.cos() * VEL_MULT, tangent_angle.sin() * VEL_MULT];
            Body::new(2.0, [x, y], velocity)
        })
    }).flatten().collect::<Vec<_>>();

    //generate particle cloud with all the remaining particles of the num_bodies (maybe replace the num_bodies here with something else)
    // Generate particles that don't belong to the arms
    let mut rng = rand::thread_rng();
    let mut other_particles = Vec::new();
    for _ in 0..(num_bodies) {
        let angle = rng.gen_range(0.0..2.0 * PI);
        let tangent_angle = if clockwise {
            angle + PI / 2.0 // Rotate clockwise by 90 degrees
        } else {
            angle - PI / 2.0 // Rotate counterclockwise by 90 degrees
        };

        let distance = rng.gen_range(0.0..radius);
        let x = center_pos[0] + distance * angle.cos();
        let y = center_pos[1] + distance * angle.sin();

        let velocity = [tangent_angle.cos() * VEL_MULT, tangent_angle.sin() * VEL_MULT];
        other_particles.push(Body::new(1.0, [x, y], velocity));
    }


    let mut bodies: Vec<Body> = vec![center_body];
    bodies.append(&mut arms);
    bodies.append(&mut other_particles);

    return bodies;
}

pub fn generate_spiral_galaxy(
    center_pos: [f32; 2],
    amount_of_particles: u32,
    center_mass: f32,
    radius: f32,
) -> Vec<Body> {
    const RADIUS_EXPONENT: f32 = 0.2;
    const ANGLE_EXPONENT: f32 = 0.5;

    let mut rng = rand::rngs::SmallRng::from_entropy();
    let normal_distribution = Normal::new(0.0f32, 1.0f32).unwrap();

    let mut bodies = Vec::with_capacity(amount_of_particles as usize);

    for _ in 0..amount_of_particles {
        let radius = rng.gen::<f32>().powf(RADIUS_EXPONENT) * radius;
        let angle = rng.gen::<f32>() * 2.0 * PI;

        let angle_noise = normal_distribution.sample(&mut rng);
        let angle_with_noise = angle + angle_noise * ANGLE_EXPONENT;

        let x = center_pos[0] + radius * angle_with_noise.cos();
        let y = center_pos[1] + radius * angle_with_noise.sin();

        let dx = x - center_pos[0];
        let dy = y - center_pos[1];
        let distance = (dx * dx + dy * dy).sqrt();

        let velocity_x = -dy * 50.0 / distance;
        let velocity_y = dx * 50.0 / distance;

        let velocity = [velocity_x, velocity_y];

        bodies.push(Body::new(1.0, velocity, [x, y]));
    }

    let mut central_bodies = Vec::with_capacity(4);
    for _ in 0..4 {
        central_bodies.push(Body::new(center_mass, [0.0, 0.0], center_pos));
    }

    bodies.extend(central_bodies);

    bodies
}


