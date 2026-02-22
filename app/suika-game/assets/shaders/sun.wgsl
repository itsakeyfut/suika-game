// Sun shader
//
// Renders a warm glowing sun disc with:
//   - Bright core with inner radiance
//   - Soft outer halo that fades to transparent
//   - Eight light rays that rotate slowly over time

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SunMaterial {
    color: vec4<f32>,
    // params.x = elapsed time in seconds (for ray rotation)
    // params.yzw = unused padding
    params: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: SunMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv   = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let dist = length(uv);
    let t    = material.params.x;

    // --- Core disc ---
    let core = 1.0 - smoothstep(0.22, 0.27, dist);

    // Extra brightness at the very centre
    let centre_radiance = (1.0 - smoothstep(0.0, 0.22, dist)) * 0.55;

    // --- Outer halo --- fades from disc edge to transparent
    let halo = (1.0 - smoothstep(0.27, 1.05, dist)) * 0.55 * max(0.0, 1.0 - dist);

    // --- Rotating light rays (8 spokes) ---
    let angle    = atan2(uv.y, uv.x) + t * 0.18;
    let ray_ring = smoothstep(0.27, 0.38, dist) * (1.0 - smoothstep(0.38, 0.92, dist));
    let rays     = max(0.0, cos(angle * 8.0)) * ray_ring * 0.42;

    // --- Secondary thin rays between main rays ---
    let thin_rays = max(0.0, cos(angle * 8.0 + 3.14159 / 8.0)) * ray_ring * 0.18;

    let raw_alpha = clamp(core + centre_radiance + halo + rays + thin_rays, 0.0, 1.0);
    let alpha     = raw_alpha * material.color.a;

    // Warm golden colour at the core, slightly cooler at halo edges
    let warm_core = vec3<f32>(1.00, 0.95, 0.60);
    let warm_halo = vec3<f32>(1.00, 0.80, 0.30);
    let t_edge    = clamp(dist / 1.05, 0.0, 1.0);
    let base_rgb  = mix(warm_core, warm_halo, t_edge) * material.color.rgb;

    return vec4<f32>(min(base_rgb, vec3<f32>(1.0)), alpha);
}
