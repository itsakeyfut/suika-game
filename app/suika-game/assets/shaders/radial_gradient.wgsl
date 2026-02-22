// Radial gradient fragment shader
//
// Bright centre that fades to transparent at the edges.
// Used for LocalFlash merge effects (replaces the flat-colour sprite).

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct RadialGradientMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: RadialGradientMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv   = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let dist = length(uv); // 0 at centre, 1 at edge

    // Radial gradient: full opacity at centre, transparent at edge
    let gradient = 1.0 - smoothstep(0.0, 1.0, dist);

    return vec4<f32>(material.color.rgb, material.color.a * gradient);
}
