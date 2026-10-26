struct Uniforme {
    curseur: vec2<f32>,
    rayon: f32,
    epaisseur: f32,
    couleur: vec3<f32>,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforme;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let d = abs(distance(pos.xy, u.curseur) - u.rayon);
    let a = 1.0 - smoothstep(u.epaisseur * 0.5 - 0.75, u.epaisseur * 0.5 + 0.75, d);
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(u.couleur, a);
    }
    return vec4<f32>(u.couleur * a, a);
}
