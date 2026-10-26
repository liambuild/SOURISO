struct Uniforme {
    curseur: vec2<f32>,
    rayon: f32,
    adoucissement: f32,
    couleur: vec3<f32>,
    intensite: f32,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforme;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

// Transparent jusqu'au rayon, puis le voile monte jusqu'à son intensité sur la largeur de
// l'adoucissement.
@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let d = distance(pos.xy, u.curseur);
    let a = u.intensite * smoothstep(u.rayon, u.rayon + u.adoucissement, d);
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(u.couleur, a);
    }
    return vec4<f32>(u.couleur * a, a);
}
