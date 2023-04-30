// https://stackoverflow.com/questions/4200224/random-noise-functions-for-glsl
fn randomFloat(xy: vec2<f32>, seed: f32) -> f32 {
    let offset = vec2<f32>(0.12345, 0.54321);
    let xy = xy + offset;
    // Golden ratio
    var PHI = 1.61803398874989484820459;
    return abs(fract(tan(distance(xy * PHI, xy) * (seed + 0.765831)) * xy.x));
}