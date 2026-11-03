//! The see-through world parts in the original's order (SPEC-0004 Drawing a node tree 3b; SPEC-0046 Material draw list):
//! buckets by shader list and texture chunk in order of first use, four sub-lists by the pass byte, then queue order;
//! the dome first, the second world next, the current world last. Nothing is sorted by distance, and SPEC-0046 warns
//! that doing so makes blended layers swap as the camera moves. Bevy sorts its blended draws by mesh centre each frame,
//! so the ranked parts have their sort value replaced just before that sort; the sort value is not used by the GPU.

use std::collections::HashMap;

use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::prelude::*;
use bevy::render::render_phase::{sort_phase_system, ViewSortedRenderPhases};
use bevy::render::view::ExtractedView;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};
use neopets_data::model::ModelData;

/// A world part's place in the draw: its world (None for the dome) and its rank within it.
#[derive(Component, Clone, Copy, Debug)]
pub struct DrawRank {
    pub pack: Option<u32>,
    pub rank: u32,
}

/// Each leaf's rank from its bucket and sub-list, the leaves given in queue order. The ranks are even, for a part's
/// second pass to draw straight after its first.
fn order<B: PartialEq>(leaves: &[(B, u8)]) -> Vec<u32> {
    let mut first_use: Vec<&B> = Vec::new();
    let mut keys: Vec<(usize, u8, usize)> = Vec::with_capacity(leaves.len());
    for (i, (bucket, pass)) in leaves.iter().enumerate() {
        let b = first_use.iter().position(|x| *x == bucket).unwrap_or_else(|| {
            first_use.push(bucket);
            first_use.len() - 1
        });
        keys.push((b, *pass, i));
    }
    keys.sort();
    let mut out = vec![0; leaves.len()];
    for (position, k) in keys.iter().enumerate() {
        out[k.2] = 2 * position as u32;
    }
    out
}

pub fn ranks(m: &ModelData) -> Vec<u32> {
    let leaves: Vec<((usize, u8), u8)> = m.parts.iter().map(|p| (p.bucket, p.pass)).collect();
    order(&leaves)
}

/// The sort value of each ranked entity this frame.
#[derive(Resource, Default)]
struct SortValues(HashMap<Entity, f32>);

/// Below every distance a world's draws reach; the worlds' ranges stay apart.
const FIRST: f32 = -1.0e6;
const WORLD_SPAN: f32 = 2.0e5;

fn extract(mut values: ResMut<SortValues>, parts: Extract<Query<(Entity, &DrawRank)>>, play: Extract<Option<Res<crate::PlayState>>>) {
    let current = play.as_ref().map(|p| p.play.world.pack);
    values.0.clear();
    for (e, r) in &parts {
        let world = match r.pack {
            None => 0.0,
            Some(p) if Some(p) == current => 2.0,
            Some(_) => 1.0,
        };
        values.0.insert(e, FIRST + world * WORLD_SPAN + r.rank as f32);
    }
}

fn apply(views: Query<&ExtractedView>, mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>, values: Res<SortValues>) {
    for view in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else { continue };
        // At the eye the distance is zero, which leaves the value alone.
        let eye = view.world_from_view.translation();
        for item in phase.items.values_mut() {
            if let Some(&v) = values.0.get(&item.entity.1.id()) {
                item.sorting_info = TransparentSortingInfo3d::Sorted { mesh_center: eye, depth_bias: v };
            }
        }
    }
}

pub struct DrawOrderPlugin;

impl Plugin for DrawOrderPlugin {
    fn build(&self, app: &mut App) {
        let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
        r.init_resource::<SortValues>();
        r.add_systems(ExtractSchedule, extract);
        r.add_systems(Render, apply.in_set(RenderSystems::PhaseSort).before(sort_phase_system::<Transparent3d>));
    }
}

#[cfg(test)]
mod tests {
    use super::order;

    // SPEC-0004 3b's test vector: A (pass 0), B (pass 2), C (pass 0) in one bucket draw A, C, B.
    #[test]
    fn sub_lists_draw_in_order_within_a_bucket() {
        assert_eq!(order(&[(0, 0), (0, 2), (0, 0)]), vec![0, 4, 2]);
    }

    // Buckets flush in order of first use, whatever their leaves' pass bytes.
    #[test]
    fn buckets_flush_in_order_of_first_use() {
        assert_eq!(order(&[(7, 3), (2, 0), (7, 1)]), vec![2, 4, 0]);
    }
}
