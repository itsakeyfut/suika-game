// Screen-droplet fragment shader
//
// Renders a translucent water-drop blob that simulates a droplet on the camera
// lens.  Uses a hollow ring with a bright highlight to suggest light refraction
// through a water surface.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct ScreenDropletMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: ScreenDropletMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv   = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let dist = length(uv);

    // Outer soft boundary
    let outer = 1.0 - smoothstep(0.75, 1.0, dist);
    // Inner transparent hole (gives hollow-droplet appearance)
    let inner = smoothstep(0.35, 0.55, dist);
    let ring  = outer * inner;

    // Small bright highlight at top-left (simulates light refraction)
    let hl_pos  = uv - vec2<f32>(-0.35, 0.35);
    let hl_dist = length(hl_pos);
    let highlight = (1.0 - smoothstep(0.0, 0.25, hl_dist)) * 0.45;

    let alpha = clamp(ring * 0.55 + highlight, 0.0, 1.0) * material.color.a;

    // Slightly brighten the highlight region
    let hl_bright = material.color.rgb + vec3<f32>(0.12, 0.12, 0.12) * highlight;
    return vec4<f32>(hl_bright, alpha);
}
