//! Characters darken with the ground they stand on (this build's presentation, not in any spec;
//! the user's choice after noticing that NPCs keep one brightness in dark and shaded places).
//! SPEC-0046 lights a character only from the area's light set, so the scene's baked shading
//! (its vertex colours: shade under trees, dark corners, the night swamp) never reaches it. Each
//! world scene's ground parts give a grid of their vertices' brightness (4-unit cells, with the
//! height of each sample). A character's draw tint is scaled by the brightness near its feet
//! against daylit ground (DAY: the bright ground of the day worlds measured, 909, 934, 696 and
//! 1023, is 0.84 to 0.97), between MIN and 1, so a character in the open by day is drawn as
//! before, one in shade darkens, and one in a night world (722's ground is about 0.3) darkens most. NEOPETS_NO_GROUND_SHADE
//! turns it off (diagnostic).
use std::collections::HashMap;
use std::sync::Mutex;

const CELL: f32 = 4.0;
/// The darkest a character is drawn, as a share of its colour.
const MIN: f32 = 0.45;
/// The ground brightness at which a character is drawn in full.
const DAY: f32 = 0.8;

struct Grid {
    cells: HashMap<(i32, i32), Vec<(f32, f32)>>,
    reference: f32,
}

static GRIDS: Mutex<Option<HashMap<u32, Grid>>> = Mutex::new(None);

fn key(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (z / CELL).floor() as i32)
}

/// Records a world scene's ground brightness: `samples` are (position, brightness 0 to 1) of
/// its ground parts' vertices.
pub fn record(pack: u32, samples: impl Iterator<Item = ([f32; 3], f32)>) {
    let mut cells: HashMap<(i32, i32), Vec<(f32, f32)>> = HashMap::new();
    for (p, b) in samples {
        cells.entry(key(p[0], p[2])).or_default().push((p[1], b));
    }
    let mut means: Vec<f32> = cells.values().map(|v| v.iter().map(|s| s.1).sum::<f32>() / v.len() as f32).collect();
    if means.is_empty() {
        return;
    }
    means.sort_by(f32::total_cmp);
    let reference = means[(means.len() * 4 / 5).min(means.len() - 1)].max(1e-3);
    if std::env::var_os("NEOPETS_TRACE_GROUND_SHADE").is_some() {
        println!("GROUND SHADE recorded {pack}: {} cells, reference {reference:.3}", cells.len());
    }
    let mut g = GRIDS.lock().unwrap_or_else(|e| e.into_inner());
    g.get_or_insert_with(HashMap::new).insert(pack, Grid { cells, reference });
}

/// The light scale of a character at `at` in one of `packs`: 1 where nothing is recorded.
pub fn factor(packs: &[u32], at: [f32; 3]) -> f32 {
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *OFF.get_or_init(|| std::env::var_os("NEOPETS_NO_GROUND_SHADE").is_some()) {
        return 1.0;
    }
    let g = GRIDS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(grids) = g.as_ref() else { return 1.0 };
    let (cx, cz) = key(at[0], at[2]);
    for pack in packs {
        let Some(grid) = grids.get(pack) else { continue };
        let (mut sum, mut n) = (0.0, 0);
        for dx in -1..=1 {
            for dz in -1..=1 {
                for &(y, b) in grid.cells.get(&(cx + dx, cz + dz)).into_iter().flatten() {
                    // Ground near the feet's height only: not a bridge above or water far below.
                    if (y - at[1]).abs() < 3.0 {
                        sum += b;
                        n += 1;
                    }
                }
            }
        }
        if n > 0 {
            let k = (sum / n as f32 / DAY).clamp(MIN, 1.0);
            if std::env::var_os("NEOPETS_TRACE_GROUND_SHADE").is_some() {
                println!("GROUND SHADE pack {pack} at ({:.0}, {:.1}, {:.0}): mean {:.3} reference {:.3} -> {k:.2}", at[0], at[1], at[2], sum / n as f32, grid.reference);
            }
            return k;
        }
    }
    1.0
}
