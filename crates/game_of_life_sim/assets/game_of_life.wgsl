#import core.wgsl

@group(0) @binding(0)
var image: texture_storage_2d<rgba16float, read_write>;
@group(0) @binding(1)
var data_in: texture_storage_2d<rgba16float, read_write>;

// =============================== INIT =============================== //

@compute @workgroup_size(32, 32, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let randomNumber = randomFloat(vec2<f32>(invocation_id.xy), 0.5);
    let alive = randomNumber > 1.0;
    let color = vec4<f32>(f32(alive), 0.0, 0.0, 1.0);

    textureStore(data_in, location, color);
}

// =============================== COMPUTE =============================== //

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> u32 {
    let size = vec2<i32>(textureDimensions(data_in));
    let loc = (location + vec2<i32>(offset_x, offset_y)) % vec2<i32>(size);
    let value: vec4<f32> = textureLoad(data_in, loc);
    return u32(value.x);
}

fn count_neighbors_simple(location: vec2<i32>) -> u32 {
    var result: u32 = 0u;
    for (var x: i32 = -1; x < 2; x++) {
        for (var y: i32 = -1; y < 2; y++) {
            if x == 0 && y == 0 {
                continue;
            }

            result += is_alive(location, x, y); 
        }
    }
    
    return result;
}

@compute @workgroup_size(32, 32, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let num_neighbors = count_neighbors_simple(location);
    let is_alive = bool(is_alive(location, 0, 0));

    // This will be replaced by the DSL. Look at `dsl.rs` for more info.
    // It expects a result value of type `u32`.
    var result: u32 = 0u;
    
    {PLACEHOLDER}

    let color = vec4<f32>(f32(result), 0.0, 0.0, 1.0);
    textureStore(image, location, color);
}