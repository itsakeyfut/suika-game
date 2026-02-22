// Cloud puff fragment shader
//
// Renders a fluffy cumulus-style cloud using smooth union of multiple SDF
// circles.  The resulting shape has a flat bottom and bumpy rounded top,
// matching the classic cartoon-cloud silhouette.
//
// Shape layout (UV space, y-up):
//
//       (o)  (o)
//      ( O  O  O )
//       (  base  )
//

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct CloudMaterial {
    color: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> material: CloudMaterial;

// Smooth minimum — blends two SDF distances for a organic union.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

// Signed distance from point `p` to a circle centred at `center` with
// the given `radius`.  Negative = inside, positive = outside.
fn circle_sdf(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Remap UV [0,1] -> centred [-1, 1]
    let uv = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    // Correct for the cloud quad's aspect ratio (width : height ≈ 1.82)
    // so that circle_sdf produces round blobs in screen space.
    let aspect = 1.82;
    let p = vec2<f32>(uv.x, uv.y * aspect);

    // --- Cloud silhouette: six overlapping SDF circles ---
    //
    // Wide, flat base + three dome bumps across the top.
    var d = circle_sdf(p, vec2<f32>( 0.00, -0.28), 0.60); // wide flat base
    d = smin(d, circle_sdf(p, vec2<f32>(-0.42,  0.03), 0.34), 0.20); // left mid
    d = smin(d, circle_sdf(p, vec2<f32>( 0.42,  0.00), 0.36), 0.20); // right mid
    d = smin(d, circle_sdf(p, vec2<f32>( 0.05,  0.30), 0.42), 0.22); // top centre dome
    d = smin(d, circle_sdf(p, vec2<f32>(-0.25,  0.38), 0.26), 0.17); // top-left bump
    d = smin(d, circle_sdf(p, vec2<f32>( 0.32,  0.35), 0.28), 0.17); // top-right bump

    // Soft edge: opaque well inside, transparent outside, smooth transition.
    let alpha = (1.0 - smoothstep(-0.04, 0.12, d)) * material.color.a;

    // Subtle interior brightness to give a fluffy, volumetric feel.
    let interior_t  = clamp(-d / 0.5, 0.0, 1.0);
    let brightness  = 1.0 + interior_t * 0.10;
    let rgb         = min(material.color.rgb * brightness, vec3<f32>(1.0));

    return vec4<f32>(rgb, clamp(alpha, 0.0, 1.0));
}
