struct Uniforme {
    couleur: vec3<f32>,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
    taille: vec2<f32>,
    remplissage: vec2<f32>,
    // Un disque par entrée : x, y (pixels), rayon (pixels), opacité.
    points: array<vec4<f32>, 128>,
};

@group(0) @binding(0) var<uniform> u: Uniforme;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) disque: vec3<f32>,
    @location(1) @interpolate(flat) opacite: f32,
};

const COINS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

// Une instance par disque : un carré qui l'entoure.
@vertex
fn vs(@builtin(vertex_index) i: u32, @builtin(instance_index) k: u32) -> Sortie {
    let pt = u.points[k];
    let p = pt.xy + COINS[i] * (pt.z + 1.5);
    var s: Sortie;
    s.position = vec4<f32>(p.x / u.taille.x * 2.0 - 1.0, 1.0 - p.y / u.taille.y * 2.0, 0.0, 1.0);
    s.disque = vec3<f32>(pt.xy, pt.z);
    s.opacite = pt.w;
    return s;
}

// Disque au cœur plein et au bord doux.
@fragment
fn fs(s: Sortie) -> @location(0) vec4<f32> {
    let d = distance(s.position.xy, s.disque.xy);
    let a = s.opacite * (1.0 - smoothstep(s.disque.z * 0.4, s.disque.z, d));
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(u.couleur, a);
    }
    return vec4<f32>(u.couleur * a, a);
}
