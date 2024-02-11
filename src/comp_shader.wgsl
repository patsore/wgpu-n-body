//some of this code was taken (and modified to use 2 dimensions, different buffer layout) from 
//https://github.com/Canleskis/particular/blob/main/particular/src/compute_method/gpu_compute/compute.wgsl 

const kSoftening = 1.0;
const DT = 0.01;


@group(0)
@binding(0)
var<storage, read_write> positions: array<vec2<f32>>;
@group(0)
@binding(1)
var <storage, read> masses: array<f32>;
@group(0)
@binding(2)
var <storage, read_write> velocities: array<vec2<f32>>;

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;

    let pos_from = positions[i];
    var acceleration = vec2<f32>(0.0);

    for (var j = 0u; j < arrayLength(&positions); j++) {
        let pos_to = positions[j];

        let dirx = pos_to.x - pos_from.x;
        let diry = pos_to.y - pos_from.y;

        let norm = dirx * dirx + diry * diry;
        //todo could change to inverse sq
        let inv = masses[j] * inverseSqrt(norm * norm * norm);

        let ax = dirx * inv;
        let ay = diry * inv;

        if (norm > 0.3) {
            acceleration.x += ax;
            acceleration.y += ay;
        }
    }

    let velocity = velocities[i];
    let new_velocity = velocity + acceleration * DT;

    velocities[i] = new_velocity;

    let new_pos = pos_from + new_velocity * DT;
    positions[i] = new_pos;
}
