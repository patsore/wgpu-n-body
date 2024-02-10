var<private> VERTICES: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.7321,-1.0),
    vec2<f32>( 1.7321,-1.0), // sqrt(3) ≈ 1.7321
    vec2<f32>( 0.0, 2.0),
);

//struct Body{
//    @location(0) world_pos: vec2<f32>,
//    @location(1) mass: f32,
//    @location(2) density: f32,
//    @location(3) radius: f32,
//}

struct Body{
    @location(0) velocity: vec2<f32>,
    @location(1) world_pos: vec2<f32>,
    //going to ignore this for now
    @location(2) mass: f32,
    @location(3) padding: u32,
}

struct VertexOutput{
  @builtin(position) clip_position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) local_position: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
//    body: Body,
    @location(0) world_pos: vec2<f32>,

) -> VertexOutput{
      var out: VertexOutput;

      let local_position = VERTICES[index];

      out.clip_position = vec4<f32>((local_position  + world_pos), 0.0, 1.0) * camera.view_proj;

      out.local_position = local_position;
//      out.color = vec4<f32>(viridis_quintic(body.mass), 1.0);
      out.color = vec4<f32>(1.0, 1.0, 1.0, 0.0392156862);
      return out;
}

fn viridis_quintic(in: f32) -> vec3<f32>{
        let x = clamp(in, 0.0, 1.0);
        let x1 = vec4<f32>( 1.0, x, x * x, x * x * x ); // 1 x x2 x3
        let x2 = x1 * x1.w * x; // x4 x5 x6 x7
        return vec3<f32>(
                dot( x1.xyzw, vec4( 0.0280268003, -0.143510503, 2.225793877, -14.815088879 ) ) + dot( x2.xy, vec2( 25.212752309, -11.772589584 ) ),
                dot( x1.xyzw, vec4( -0.002117546, 1.617109353, -1.909305070, 2.701152864 ) ) + dot( x2.xy, vec2( -1.685288385, 0.178738871 ) ),
                dot( x1.xyzw, vec4( 0.300805501, 2.614650302, -12.019139090, 28.933559110 ) ) + dot( x2.xy, vec2( -33.491294770, 13.762053843 ) ) );

}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let vec_len = dot(in.local_position, in.local_position);

    let glow_start = 0.25;
    let dist = (1.0 - vec_len) * (1.0 + glow_start);
    let brightness = pow(dist, 20.0); //falloff, higher = sharper = looks more like glow
    let intensity = clamp(brightness, 0.0, 1.0);

    let bg_color = vec4<f32>(0.0, 0.0, 0.0, 0.0); //needs to be able to blend with other particles

    let finalColor = mix(bg_color, in.color, intensity);



    return finalColor;
}