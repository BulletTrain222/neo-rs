//! A test tool of this build: F12 names what the view's centre is on (the + in the free camera), nearest first, on the
//! toast and in private/picks.txt, so a report can give the piece's identity and not only where it was seen.
//! NEOPETS_PICK="u,v;u,v" does the same once, seven seconds in, at those fractions of the screen (diagnostic).

use bevy::prelude::*;

use crate::dev_menu::DevMenu;
use crate::orbit::Orbit;

struct Hit {
    t: f32,
    entity: Entity,
    point: Vec3,
    verts: usize,
}

/// The ray's parameter where it meets the triangle, from either side.
fn meets(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1.0e-9 {
        return None;
    }
    let s = o - a;
    let u = s.dot(p) / det;
    let q = s.cross(e1);
    let v = d.dot(q) / det;
    let t = e2.dot(q) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0).then_some(t)
}

/// Each drawn mesh the ray meets, at its nearest, nearest mesh first.
fn hits(world: &mut World, ray: Ray3d) -> Vec<Hit> {
    let mut q = world.query::<(Entity, &Mesh3d, &GlobalTransform, &ViewVisibility)>();
    let world = &*world;
    let meshes = world.resource::<Assets<Mesh>>();
    let mut out = Vec::new();
    for (entity, mesh, at, shown) in q.iter(world) {
        if !shown.get() {
            continue;
        }
        let Some(m) = meshes.get(&mesh.0) else { continue };
        let Some(pos) = m.attribute(Mesh::ATTRIBUTE_POSITION).and_then(|a| a.as_float3()) else { continue };
        let Some(ix) = m.indices() else { continue };
        let ix: Vec<usize> = ix.iter().collect();
        let to = Mat4::from(at.affine());
        let from = to.inverse();
        let (o, d) = (from.transform_point3(ray.origin), from.transform_vector3(*ray.direction));
        let mut best: Option<f32> = None;
        for t in ix.chunks_exact(3) {
            let (Some(a), Some(b), Some(c)) = (pos.get(t[0]), pos.get(t[1]), pos.get(t[2])) else { continue };
            if let Some(h) = meets(o, d, Vec3::from(*a), Vec3::from(*b), Vec3::from(*c)) {
                best = Some(best.map_or(h, |x: f32| x.min(h)));
            }
        }
        if let Some(t) = best {
            out.push(Hit { t, entity, point: to.transform_point3(o + d * t), verts: pos.len() });
        }
    }
    out.sort_by(|a, b| a.t.total_cmp(&b.t));
    out
}

/// What the entity is: its plant patch and model, its scene part, its object, else this build's tags on it and its parents.
fn describe(world: &World, e: Entity) -> String {
    if let Some(p) = world.get::<crate::foliage_view::PatchPart>(e) {
        let live = world.get_resource::<crate::PlayState>().and_then(|s| s.play.foliage.get(&p.pack).and_then(|v| v.get(p.patch)).map(|l| (l.model, l.zone, l.on, l.plants.len(), l.bushes.len(), l.reeds.len(), l.kind.near, l.kind.far)));
        return format!("PLANT pack {} patch {} (model, zone, on, plants, bushes, reeds, near, far) {live:?}", p.pack, p.patch);
    }
    let mut s = String::new();
    if let Some(p) = world.get::<crate::PartSig>(e) {
        s.push_str(&format!("SCENE PART pack {} verts {} lo {:?} hi {:?} pass1 {} ", p.pack, p.count, p.lo, p.hi, p.pass1));
    }
    if let Some(r) = world.get::<crate::draw_order::DrawRank>(e) {
        s.push_str(&format!("rank {} ", r.rank));
    }
    let mut at = Some(e);
    while let Some(x) = at {
        if let Some(o) = world.get::<crate::ObjectRoot>(x) {
            s.push_str(&format!("OBJECT pack {} index {} ", o.0, o.1));
        }
        if let Ok(names) = world.inspect_entity(x) {
            let names: Vec<String> = names.map(|c| c.name().to_string()).filter(|n| n.starts_with("neopets_app")).map(|n| n.rsplit("::").next().unwrap_or("").to_string()).collect();
            if !names.is_empty() {
                s.push_str(&format!("{names:?} "));
            }
        }
        at = world.get::<ChildOf>(x).map(|c| c.parent());
    }
    let material = world.get::<MeshMaterial3d<StandardMaterial>>(e).and_then(|m| world.resource::<Assets<StandardMaterial>>().get(&m.0));
    if let Some(m) = material {
        s.push_str(&format!("alpha {:?}", m.alpha_mode));
    }
    s
}

pub fn pick(world: &mut World, mut fired: Local<bool>) {
    let mut at: Vec<(f32, f32)> = Vec::new();
    if world.resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::F12) {
        at.push((0.5, 0.5));
    }
    if !*fired && world.resource::<Time<bevy::time::Real>>().elapsed_secs() > 7.0 {
        *fired = true;
        if let Ok(v) = std::env::var("NEOPETS_PICK") {
            at.extend(v.split(';').filter_map(|p| p.split_once(',')).filter_map(|(u, v)| Some((u.trim().parse().ok()?, v.trim().parse().ok()?))));
        }
    }
    if at.is_empty() {
        return;
    }
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Orbit>>();
    let rays: Vec<Ray3d> = cameras.iter(world).next().map_or(Vec::new(), |(c, g)| {
        let Some(size) = c.logical_viewport_size() else { return Vec::new() };
        at.iter().filter_map(|(u, v)| c.viewport_to_world(g, size * Vec2::new(*u, *v)).ok()).collect()
    });
    let (pack, ticks) = world.get_resource::<crate::PlayState>().map_or((0, 0), |s| (s.play.world.pack, s.play.ticks));
    let mut text = String::new();
    let mut first = None;
    for ray in rays {
        let found = hits(world, ray);
        text.push_str(&format!("pick: pack {pack} tick {ticks} from {:.1},{:.1},{:.1} along {:.3},{:.3},{:.3}\n", ray.origin.x, ray.origin.y, ray.origin.z, ray.direction.x, ray.direction.y, ray.direction.z));
        for (i, h) in found.iter().take(6).enumerate() {
            let d = describe(world, h.entity);
            text.push_str(&format!("  {}. {:.1} away at {:.1},{:.1},{:.1}, mesh of {} verts: {d}\n", i + 1, h.t, h.point.x, h.point.y, h.point.z, h.verts));
            first.get_or_insert(d);
        }
        if found.is_empty() {
            text.push_str("  nothing\n");
        }
    }
    print!("{text}");
    let _ = std::fs::create_dir_all("private");
    let saved = std::fs::OpenOptions::new().create(true).append(true).open("private/picks.txt").and_then(|mut f| std::io::Write::write_all(&mut f, text.as_bytes()));
    if let Err(e) = saved {
        error!("F12: could not write private/picks.txt: {e}");
    }
    world.resource_mut::<DevMenu>().toast = Some((format!("F12 saved to private/picks.txt: {}", first.unwrap_or_else(|| "nothing here".into())), 300));
}

/// The + at the view's centre, while the free camera is on.
#[derive(Component)]
pub struct PickCross;

pub fn show_cross(mut commands: Commands, dev: Res<DevMenu>, crosses: Query<Entity, With<PickCross>>) {
    match (dev.free.is_some(), crosses.iter().next()) {
        (true, None) => {
            commands.spawn((
                Text::new("+"),
                TextFont { font_size: bevy::text::FontSize::Px(32.0), ..default() },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node { position_type: PositionType::Absolute, left: percent(50), top: percent(50), margin: UiRect { left: px(-10), top: px(-20), ..default() }, ..default() },
                GlobalZIndex(1000),
                PickCross,
            ));
        }
        (false, Some(e)) => commands.entity(e).despawn(),
        _ => {}
    }
}
