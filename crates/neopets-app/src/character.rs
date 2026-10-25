//! A hero outfit playing its animations (SPEC-0005): three families (main, sub-part, face),
//! each with a playback record from neopets-sim, sampled into its joint list and composed into
//! skin matrices for the outfit's bone table.
use neopets_data::anim::{self, Affine, BoneTable, Family, FamilyData, Joint};
use neopets_sim::clip::{ClipInfo, PlaybackRecord};
use neopets_sim::hero::ClipState;

struct FamilyState {
    data: FamilyData,
    record: PlaybackRecord,
    joints: Vec<Option<Joint>>,
    /// Mask bit of the family's first joint: 0 for main, N_main for sub-part and face.
    mask_base: u32,
    /// Skin bone of the family's first joint.
    skin_base: usize,
    current: Option<usize>,
}

pub struct HeroAnim {
    bones: BoneTable,
    families: Vec<FamilyState>,
    action: u32,
}

/// Action ids valid for the hero tables (SPEC-0005: ids 0 to 192).
pub const ACTIONS: u32 = 193;

impl HeroAnim {
    /// `families` is main, sub-part, face, as in the hero resource table.
    pub fn new(bones: BoneTable, families: Vec<FamilyData>) -> Self {
        let n_main = families.first().map_or(0, |f| f.def.len());
        let mut skin_base = 0;
        let families = families
            .into_iter()
            .enumerate()
            .map(|(i, data)| {
                let n = data.def.len();
                let state = FamilyState {
                    joints: vec![None; n],
                    mask_base: if i == 0 { 0 } else { n_main as u32 },
                    skin_base,
                    data,
                    record: PlaybackRecord::default(),
                    current: None,
                };
                skin_base += n;
                state
            })
            .collect();
        let mut hero = Self { bones, families, action: 0 };
        hero.play(3);
        hero
    }

    pub fn action(&self) -> u32 {
        self.action
    }

    /// The main family's playing clip, for the hero update of the next tick.
    pub fn clip_state(&self) -> ClipState {
        let record = self.families.first().map(|f| &f.record);
        ClipState {
            action: self.action,
            progress: record.map_or(0.0, |r| r.progress()),
            finished: record.is_some_and(|r| r.finished()),
        }
    }

    /// Request an action id. A change of main clip restarts the main, sub-part and face clips;
    /// the same main clip already playing starts nothing (SPEC-0005, Clip start).
    pub fn play(&mut self, action: u32) {
        self.request(action, false);
    }

    /// Request an action, restarting its clip even when it is current when `force` is set (the
    /// forced start of SPEC-0040, used by chained attack repeats).
    pub fn request(&mut self, action: u32, force: bool) {
        self.action = action;
        let main = self.families.first().and_then(|f| f.data.clip_for(action));
        let restart = main.is_some() && (force || main != self.families[0].current);
        if !restart {
            return;
        }
        for f in &mut self.families {
            let Some(c) = f.data.clip_for(action) else { continue };
            let clip = &f.data.clips[c];
            f.record.start(
                ClipInfo {
                    frames: clip.frames as u32,
                    looping: clip.looping,
                    speed_byte: clip.speed_byte,
                    fade_in: clip.fade_in,
                },
                1.0,
            );
            f.current = Some(c);
        }
    }

    /// Set the play rate R of every family (hero clip speeds follow R every pass).
    pub fn set_rate(&mut self, rate: f32) {
        for f in &mut self.families {
            f.record.set_rate(rate);
        }
    }

    /// One game tick: step each family's record and combine its sample into its joints. Returns
    /// the main layer's root step for the next tick: the root record of the current key times
    /// the clip speed, doubled while a finished one-shot is held (SPEC-0040 Behavior 14C).
    pub fn tick(&mut self) -> [f32; 3] {
        let mut root = [0.0; 3];
        for (i, f) in self.families.iter_mut().enumerate() {
            let (Some(c), Some(s)) = (f.current, f.record.tick()) else { continue };
            if i == 0 {
                let clip = &f.data.clips[c];
                let scale = f.record.speed() * if f.record.finished() { 2.0 } else { 1.0 };
                if let Some(r) = clip.root.get(s.key as usize) {
                    root = r.map(|v| v * scale);
                }
            }
            anim::sample_into(
                &f.data.clips[c],
                s.key as usize,
                s.next_key as usize,
                s.fraction,
                f.mask_base,
                &mut f.joints,
                s.replace,
                s.blend,
            );
        }
        root
    }

    /// The bone table the animation poses.
    pub fn bones(&self) -> &BoneTable {
        &self.bones
    }

    /// Table-B frame bits on the current frame of every family's playing clip, ORed (the
    /// attack window query of SPEC-0005/SPEC-0036, single-frame form).
    pub fn frame_bits(&self) -> u32 {
        self.families
            .iter()
            .filter_map(|f| {
                let c = f.current?;
                if f.record.finished() {
                    return None;
                }
                f.data.clips[c].frame_bits.get(f.record.cursor() as usize).copied()
            })
            .fold(0, |a, b| a | b)
    }

    /// Bone world frames in model space: W_i = S_i × translate(rest position_i).
    pub fn bone_frames(&self) -> Vec<Affine> {
        self.skin_matrices()
            .into_iter()
            .zip(&self.bones.rest_position)
            .map(|(s, &rest)| Affine { cols: s.cols, t: s.apply(rest) })
            .collect()
    }

    pub fn skin_matrices(&self) -> Vec<Affine> {
        fn family(f: &FamilyState) -> Family<'_> {
            Family {
                def: &f.data.def,
                skin_base: f.skin_base,
                joints: &f.joints,
            }
        }
        let others: Vec<Family> = self.families.iter().skip(1).map(family).collect();
        anim::skin_matrices(&self.bones, &family(&self.families[0]), &others)
    }

    pub fn status(&self) -> String {
        let main = &self.families[0];
        match main.current {
            Some(c) => {
                let clip = &main.data.clips[c];
                format!(
                    "Animation (SPEC-0005): action {} -> main clip {c} ({} frames, {}, S {}), cursor {:.2}. - and = change action",
                    self.action,
                    clip.frames,
                    if clip.looping { "loop" } else { "one-shot" },
                    clip.speed_byte,
                    main.record.cursor()
                )
            }
            None => format!("Animation: action {} has no clip. - and = change action", self.action),
        }
    }
}

/// Skin one part: v' = Σ w_k S_(bone_k) v for positions; normals turn with the first bone's
/// rotation only, as the vertex program does (SPEC-0004). The normal is renormalised here; the
/// lit program uses it for lighting.
pub fn skin_part(
    matrices: &[Affine],
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    weights: &[[f32; 3]],
    bones: &[[u16; 3]],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let get = |b: u16| matrices.get(b as usize).unwrap_or(&Affine::IDENTITY);
    let pos = positions
        .iter()
        .zip(weights.iter().zip(bones))
        .map(|(p, (w, b))| {
            let mut out = [0.0; 3];
            for k in 0..3 {
                let q = get(b[k]).apply(*p);
                for (o, v) in out.iter_mut().zip(q) {
                    *o += w[k] * v;
                }
            }
            out
        })
        .collect();
    let nrm = normals
        .iter()
        .zip(bones)
        .map(|(n, b)| {
            let v = get(b[0]).rotate(*n);
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if len > 0.0 { v.map(|c| c / len) } else { *n }
        })
        .collect();
    (pos, nrm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skinning_matches_spec_vector() {
        // SPEC-0004: bone 0 translates by (0, 1, 0), bone 1 by (0, 0, 2); p = (1, 0, 0).
        let m = [
            Affine { t: [0.0, 1.0, 0.0], ..Affine::IDENTITY },
            Affine { t: [0.0, 0.0, 2.0], ..Affine::IDENTITY },
        ];
        let (p, _) = skin_part(&m, &[[1.0, 0.0, 0.0]], &[], &[[0.5, 0.5, 0.0]], &[[0, 1, 0]]);
        assert_eq!(p[0], [1.0, 0.5, 1.0]);
        // Weights are not renormalised.
        let (p, _) = skin_part(&m, &[[1.0, 0.0, 0.0]], &[], &[[0.5, 0.25, 0.0]], &[[0, 1, 0]]);
        assert_eq!(p[0], [0.75, 0.5, 0.5]);
    }
}
