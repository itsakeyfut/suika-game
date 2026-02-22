// Soft-circle SDF fragment shader
//
// Renders a filled circle with smooth anti-aliased edges using a signed-distance
// function.  Used for WaterDroplet particles instead of hard-edged sprites.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SoftCircleMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: SoftCircleMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Map UV [0,1] to centred coordinates: 0 at centre, 1 at edge
    let uv  = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let dist = length(uv);

    // SDF soft-circle: smooth falloff near the circumference
    let softness = 0.25;
    let alpha    = 1.0 - smoothstep(1.0 - softness, 1.0 + softness, dist);

    return vec4<f32>(material.color.rgb, material.color.a * alpha);
}
