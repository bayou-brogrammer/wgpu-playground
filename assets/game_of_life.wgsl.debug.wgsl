struct PushConstants {
    draw_start: vec2<f32>,
    draw_end: vec2<f32>,
    draw_radius: f32,
}
var<push_constant> pc: PushConstants;

// https://stackoverflow.com/questions/4200224/random-noise-functions-for-glsl
fn randomFloat(xy: vec2<f32>, seed: f32) -> f32 {
    let offset = vec2<f32>(0.12345, 0.54321);
    let xy = xy + offset;
    // Golden ratio
    var PHI = 1.61803398874989484820459;
    return abs(fract(tan(distance(xy * PHI, xy) * (seed + 0.765831)) * xy.x));
}

@group(0) @binding(0)
var image: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(1)
var data_in: texture_storage_2d<rgba16float, read_write>;

// =============================== INIT =============================== //

@compute @workgroup_size(32, 32, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let randomNumber = randomFloat(vec2<f32>(invocation_id.xy), 0.5);
    let alive = randomNumber > 0.8;
    let color = vec4<f32>(f32(alive), 0.0, 0.0, 1.0);

    textureStore(data_in, location, color);
}

// =============================== COMPUTE =============================== //

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: vec4<f32> = textureLoad(data_in, location + vec2<i32>(offset_x, offset_y));
    return i32(value.x);
}

@compute @workgroup_size(32, 32, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let is_alive = bool(is_alive(location, 0, 0));
    
    let num_neighbors: u32 = 2u;

    // This will be replaced by the DSL. Look at `dsl.rs` for more info.
    // It expects a result value of type `u32`.
    var result: u32 = 0u;

    if (is_alive) { result = ((u32((num_neighbors) == (2u))) | (u32((num_neighbors) == (3u)))); } else { result = u32((num_neighbors) == (3u)); }

    let color = vec4<f32>(f32(result), 0.0, 0.0, 1.0);
    textureStore(image, location, color);
}