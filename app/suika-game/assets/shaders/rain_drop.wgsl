// Rain-drop streak fragment shader
//
// Renders a soft elongated streak that simulates a falling rain drop.
// Features:
//   - Horizontal: Gaussian-like falloff (very narrow, no hard edges)
//   - Vertical: bright at centre, fades to transparent at both tips
//   - Centre glow: subtle brightness boost to simulate light refraction

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct RainDropMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: RainDropMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Remap UV [0,1] -> centred [-1, 1]
    let uv = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    // ---- Horizontal (across the streak width) ----
    // Aggressive smoothstep: opaque only very near the centre line
    let x_fade = 1.0 - smoothstep(0.0, 1.0, abs(uv.x) * 3.5);

    // ---- Vertical (along the streak length) ----
    // Full opacity in the middle 50%, taper to transparent at both tips
    let y_fade = 1.0 - smoothstep(0.5, 1.0, abs(uv.y));

    // ---- Combined streak shape ----
    let streak = x_fade * y_fade;

    // ---- Centre glow: simulate light refraction through water ----
    // A narrow bright line down the centre of the streak
    let centre_glow = clamp(1.0 - abs(uv.x) * 5.0, 0.0, 1.0) * 0.35;
    let brightness   = 1.0 + centre_glow;

    let alpha = clamp(streak, 0.0, 1.0) * material.color.a;
    let rgb   = min(material.color.rgb * brightness, vec3<f32>(1.0));

    return vec4<f32>(rgb, alpha);
}
