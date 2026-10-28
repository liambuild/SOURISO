struct Uniforme {
    curseur: vec2<f32>,
    rayon: f32,
    adoucissement: f32,
    couleur: vec3<f32>,
    intensite: f32,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
    // Rectangle (gauche, haut, droite, bas) en pixels où le voile ne s'applique pas : la fenêtre
    // de réglages. Vide (droite <= gauche) : aucun trou.
    trou: vec4<f32>,
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
    var a = u.intensite * smoothstep(u.rayon, u.rayon + u.adoucissement, d);
    if (u.trou.z > u.trou.x) {
        // Distance au rectangle (négative dedans) : le voile est nul dedans et revient sur 2 px.
        let centre = (u.trou.xy + u.trou.zw) * 0.5;
        let demi = (u.trou.zw - u.trou.xy) * 0.5;
        let q = abs(pos.xy - centre) - demi;
        a = a * smoothstep(0.0, 2.0, max(q.x, q.y));
    }
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(u.couleur, a);
    }
    return vec4<f32>(u.couleur * a, a);
}
