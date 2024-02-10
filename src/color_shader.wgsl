var<private> VERTICES: array<Vertex, 6> = array<Vertex, 6>(
    Vertex(vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, 1.0)),
    Vertex(vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0)),
    Vertex(vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0)),
        // Triangle 2
    Vertex(vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, 1.0)),
    Vertex(vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0)),
    Vertex(vec2<f32>(-1.0, 1.0), vec2<f32>(0.0, 0.0)),
);

struct Vertex{
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput{
  @builtin(position) clip_position: vec4<f32>,
  @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) i_index: u32,
) -> VertexOutput{
      var out: VertexOutput;

      let local_position = VERTICES[index + i_index * 3];

      out.clip_position = vec4<f32>(local_position.pos, 0.0, 1.0);
      out.uv = local_position.uv;
      return out;
}

fn magma_quintic(in: f32) -> vec3<f32>{
        let x = clamp(in, 0.0, 1.0);
        let x1 = vec4<f32>( 1.0, x, x * x, x * x * x ); // 1 x x2 x3
        let x2 = x1 * x1.w * x; // x4 x5 x6 x7
    return vec3(
        dot( x1.xyzw, vec4( -0.0023226960, 1.087154378, -0.109964741, 6.333665763 ) ) + dot( x2.xy, vec2( -11.640596589, 5.337625354 ) ),
        dot( x1.xyzw, vec4( 0.010680993,0.176613780, 1.638227448, -6.743522237 ) ) + dot( x2.xy, vec2( 11.426396979, -5.523236379 ) ),
        dot( x1.xyzw, vec4( -0.008260782,2.244286052, 3.005587601, -24.279769818 ) ) + dot( x2.xy, vec2( 32.484310068, -12.688259703 ) ) );
}



@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
       let color = textureSample(texture, t_sampler, in.uv);
       let alpha = abs(color.w);
       return vec4<f32>(magma_quintic(alpha), 1.0);
//    return vec4<f32>(alpha, alpha, alpha, 1.0);
//    return vec4<f32>(0.0, 0.01, 0.0, 1.0);
}