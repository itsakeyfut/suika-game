// Weather post-process overlay shader
//
// Draws a full-screen colour vignette tinted by the current weather state.
//   Sunny  -> transparent (no overlay spawned)
//   Rainy  -> cool blue tint with soft vignette
//   Cloudy -> desaturated grey wash with soft vignette

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct WeatherPostprocessMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: WeatherPostprocessMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    // Soft vignette: slightly more opaque at the edges
    let vignette = 1.0 - dot(uv * 0.45, uv * 0.45);
    let alpha    = material.color.a * max(0.0, vignette);

    return vec4<f32>(material.color.rgb, alpha);
}
