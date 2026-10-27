struct Uniforme {
    curseur: vec2<f32>,
    taille: vec2<f32>,
    couleur: vec3<f32>,
    opacite: f32,
    rayon: f32,
    epaisseur: f32,
    demi_cote: f32,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforme;

const COINS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

// Un carré autour du curseur, en pixels : seule cette zone est remplie par la carte.
@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = u.curseur + COINS[i] * u.demi_cote;
    return vec4<f32>(p.x / u.taille.x * 2.0 - 1.0, 1.0 - p.y / u.taille.y * 2.0, 0.0, 1.0);
}

// Un trait de l'épaisseur voulue, centré sur le rayon, avec un pixel de bord adouci.
@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let d = distance(pos.xy, u.curseur);
    let a = u.opacite * clamp(u.epaisseur * 0.5 + 0.5 - abs(d - u.rayon), 0.0, 1.0);
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(u.couleur, a);
    }
    return vec4<f32>(u.couleur * a, a);
}
