struct Uniforme {
    // Une ondulation par entrée : x, y (pixels), avancement de 0 à 1, bouton (0 gauche, 1 droit).
    entrees: array<vec4<f32>, 8>,
    // Couleur RGB et opacité.
    couleur_gauche: vec4<f32>,
    couleur_droit: vec4<f32>,
    taille: vec2<f32>,
    rayon_max: f32,
    epaisseur: f32,
    // 1.0 si la surface attend de l'alpha non prémultiplié.
    non_premultiplie: f32,
    // Trois flottants et non un vec3, qui porterait la structure à 208 octets.
    remplissage_a: f32,
    remplissage_b: f32,
    remplissage_c: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforme;

struct Sortie {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) centre: vec2<f32>,
    // Rayon courant, avancement, bouton.
    @location(1) @interpolate(flat) etat: vec3<f32>,
};

const COINS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

// Le rayon grandit vite puis ralentit.
fn rayon(avancement: f32) -> f32 {
    let reste = 1.0 - avancement;
    return u.rayon_max * (1.0 - reste * reste * reste);
}

fn anneau(d: f32, r: f32, epaisseur: f32) -> f32 {
    return clamp(epaisseur * 0.5 + 0.5 - abs(d - r), 0.0, 1.0);
}

// Une instance par ondulation : un carré qui contient son anneau.
@vertex
fn vs(@builtin(vertex_index) i: u32, @builtin(instance_index) k: u32) -> Sortie {
    let e = u.entrees[k];
    let r = rayon(e.z);
    let p = e.xy + COINS[i] * (r + u.epaisseur + 2.0);
    var s: Sortie;
    s.position = vec4<f32>(p.x / u.taille.x * 2.0 - 1.0, 1.0 - p.y / u.taille.y * 2.0, 0.0, 1.0);
    s.centre = e.xy;
    s.etat = vec3<f32>(r, e.z, e.w);
    return s;
}

// Clic gauche : un anneau sur un léger disque. Clic droit : deux anneaux concentriques. Dans les
// deux cas l'opacité baisse jusqu'à zéro à la fin.
@fragment
fn fs(s: Sortie) -> @location(0) vec4<f32> {
    let d = distance(s.position.xy, s.centre);
    let r = s.etat.x;
    var couverture: f32;
    var couleur: vec4<f32>;
    if (s.etat.z > 0.5) {
        couverture = max(anneau(d, r, u.epaisseur), anneau(d, r * 0.6, u.epaisseur * 0.8));
        couleur = u.couleur_droit;
    } else {
        couverture = max(anneau(d, r, u.epaisseur), 0.15 * clamp(r - d + 0.5, 0.0, 1.0));
        couleur = u.couleur_gauche;
    }
    let reste = 1.0 - s.etat.y;
    let a = couleur.w * couverture * reste * reste;
    if (u.non_premultiplie > 0.5) {
        return vec4<f32>(couleur.xyz, a);
    }
    return vec4<f32>(couleur.xyz * a, a);
}
