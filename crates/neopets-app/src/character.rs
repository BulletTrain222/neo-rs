//! A hero outfit playing its animations (SPEC-0005): three families (main, sub-part, face),
//! each with a playback record from neopets-sim, sampled into its joint list and composed into
//! skin matrices for the outfit's bone table.
use neopets_data::anim::{self, Affine, BoneTable, Family, FamilyData, Joint};
use neopets_sim::clip::{ClipInfo, PlaybackRecord};
use neopets_sim::hero::ClipState;

#[derive(Clone)]
struct FamilyState {
    data: FamilyData,
    record: PlaybackRecord,
    joints: Vec<Option<Joint>>,
    /// Mask bit of the family's first joint: 0 for main, N_main for sub-part and face.
    mask_base: u32,
    /// Skin bone of the family's first joint.
    skin_base: usize,
    current: Option<usize>,
    /// The playing clip is mirrored (SPEC-0005 Mirrored clips; main family only).
    mirror: bool,
    /// The kept root step of a mode 0 or 1 clip.
    kept: [f32; 3],
    /// The family's role: 0 main, 1 sub-part, 2 face (region characters may lack some).
    role: u8,
}

#[derive(Clone)]
pub struct HeroAnim {
    bones: BoneTable,
    families: Vec<FamilyState>,
    action: u32,
    /// The face override id (SPEC-0040 Behavior 3D): its face clip plays instead of the main
    /// id's until it reaches progress 0.99, then the main id's face clip returns.
    face: Option<u32>,
    /// The region's kind-16 effect lists (SPEC-0034 Layout section 1), shared by its characters.
    effect_lists: std::sync::Arc<Vec<Vec<neopets_data::region::EffectEntry>>>,
    /// The pendulum chains on the sub-part joints (SPEC-0040 Behavior 25, modes 1 and 2), and
    /// the owner's position on the last chain step.
    chains: Vec<neopets_sim::chain::Chain>,
    last_owner: Option<[f32; 3]>,
    /// The main family's translation scale selector (SPEC-0005; 0 for the heroes).
    scale_set: i16,
    /// The hero's head-look point in model space for this tick (SPEC-0040 Behavior 23A head
    /// step on joint 6), cleared when no look is requested.
    look: Option<[f32; 3]>,
    /// The neck look's joints and angles for this tick (SPEC-0040 Behavior 23A mode 1).
    neck: Option<(Vec<usize>, [f32; 6])>,
}

/// The head joint of the hero's look (SPEC-0040 Behavior 23A, mode 0).
pub const HEAD_JOINT: usize = 6;
/// Word A bit 13 of the main entry: head look allowed (SPEC-0005).
const HEAD_LOOK_BIT: u32 = 1 << 13;

/// Action ids valid for the hero tables (SPEC-0005: ids 0 to 192).
pub const ACTIONS: u32 = 193;

impl HeroAnim {
    /// `families` is main, sub-part, face, as in the hero resource table.
    pub fn new(bones: BoneTable, families: Vec<FamilyData>) -> Self {
        Self::with_roles(bones, families.into_iter().enumerate().map(|(i, f)| (i as u8, f)).collect())
    }

    /// Families with their roles (0 main, 1 sub-part, 2 face).
    pub fn with_roles(bones: BoneTable, families: Vec<(u8, FamilyData)>) -> Self {
        let n_main = families.first().map_or(0, |f| f.1.def.len());
        let mut skin_base = 0;
        let families = families
            .into_iter()
            .enumerate()
            .map(|(i, (role, data))| {
                let n = data.def.len();
                let state = FamilyState {
                    joints: vec![None; n],
                    mask_base: if i == 0 { 0 } else { n_main as u32 },
                    skin_base,
                    data,
                    record: PlaybackRecord::default(),
                    current: None,
                    mirror: false, kept: [0.0; 3],
                    role,
                };
                skin_base += n;
                state
            })
            .collect();
        let mut hero = Self { bones, families, action: 0, face: None, effect_lists: Default::default(), chains: Vec::new(), last_owner: None, scale_set: 0, look: None, neck: None };
        hero.play(3);
        hero
    }

    pub fn action(&self) -> u32 {
        self.action
    }

    /// Whether the current main id's word A has the head-look bit (13) set (SPEC-0005).
    pub fn head_look_allowed(&self) -> bool {
        self.families.first().and_then(|f| f.data.actions.get(self.action as usize)).is_some_and(|&a| a & HEAD_LOOK_BIT != 0)
    }

    /// The head-look point in model space for this tick, or None (no look: the head keeps its
    /// animated pose).
    pub fn set_look(&mut self, point: Option<[f32; 3]>) {
        self.look = point;
    }

    /// The neck look for this tick: each listed joint turned about the up axis by its angle.
    pub fn set_neck(&mut self, neck: Option<(Vec<usize>, [f32; 6])>) {
        self.neck = neck;
    }

    /// Turn joint `j` and the joints below it about the up axis through the joint by `angle`.
    fn turn_about_up(&self, m: &mut [Affine], parents: &[Option<usize>], j: usize, angle: f32) {
        let Some(o) = m.get(j).map(|a| a.apply(self.bones.rest_position.get(j).copied().unwrap_or_default())) else { return };
        let (s, c) = angle.sin_cos();
        let rot = |v: [f32; 3]| [c * v[0] + s * v[2], v[1], -s * v[0] + c * v[2]];
        let under = |mut i: usize| loop {
            if i == j {
                break true;
            }
            match parents.get(i).copied().flatten() {
                Some(p) if p != i => i = p,
                _ => break false,
            }
        };
        for (i, a) in m.iter_mut().enumerate() {
            if under(i) {
                a.cols = a.cols.map(rot);
                let d = rot([a.t[0] - o[0], a.t[1] - o[1], a.t[2] - o[2]]);
                a.t = [o[0] + d[0], o[1] + d[1], o[2] + d[2]];
            }
        }
    }

    /// The parent of each skin index across the families (main joints, then each other
    /// family's joints after its skin base).
    fn skin_parents(&self) -> Vec<Option<usize>> {
        let n_main = self.families.first().map_or(0, |f| f.data.def.len());
        let mut out = vec![None; self.bones.parents.len()];
        for (fi, f) in self.families.iter().enumerate() {
            for (i, p) in f.data.def.parents.iter().enumerate() {
                let skin = f.skin_base + i;
                let Some(slot) = out.get_mut(skin) else { break };
                *slot = p.map(|p| p as usize).map(|p| if fi == 0 || p < n_main { p } else { f.skin_base + (p - n_main) });
            }
        }
        out
    }

    /// The head step (SPEC-0040 Behavior 23A): joint 6 and the joints below it are turned about
    /// the head's position so that the head's forward axis (Z) points at the look point with its
    /// up axis levelled to world up (the level rotation and the yaw and pitch turn taken
    /// together; the stand-in for the spec's two-step joint math, Medium-Low there).
    fn apply_look(&self, m: &mut [Affine], point: [f32; 3]) {
        let Some(head) = m.get(HEAD_JOINT).copied() else { return };
        let o = head.apply(self.bones.rest_position.get(HEAD_JOINT).copied().unwrap_or_default());
        let norm = |v: [f32; 3]| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            (l > 1e-6).then(|| [v[0] / l, v[1] / l, v[2] / l])
        };
        let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        // The controller's look point lies below the head for a target above (its pitch quirk);
        // the head step's pitch is negated for a point above, so the elevation is mirrored back
        // (Medium-Low for the net direction in the spec).
        let Some(fwd) = norm([point[0] - o[0], o[1] - point[1], point[2] - o[2]]) else { return };
        let Some(side) = norm(cross([0.0, 1.0, 0.0], fwd)) else { return };
        let up = cross(fwd, side);
        // R = target frame x current frame transposed (the current frame's columns, made unit).
        let cur: Vec<[f32; 3]> = head.cols.iter().map(|&c| norm(c).unwrap_or(c)).collect();
        let tgt = [side, up, fwd];
        let mut r = [[0.0f32; 3]; 3];
        for (col, rc) in r.iter_mut().enumerate() {
            for (row, v) in rc.iter_mut().enumerate() {
                *v = (0..3).map(|k| tgt[k][row] * cur[k][col]).sum();
            }
        }
        let rot = |v: [f32; 3]| [0, 1, 2].map(|row| (0..3).map(|c| r[c][row] * v[c]).sum::<f32>());
        let parents = self.skin_parents();
        let under = |mut i: usize| loop {
            if i == HEAD_JOINT {
                break true;
            }
            match parents.get(i).copied().flatten() {
                Some(p) if p != i => i = p,
                _ => break false,
            }
        };
        for (i, a) in m.iter_mut().enumerate() {
            if !under(i) {
                continue;
            }
            a.cols = a.cols.map(rot);
            let d = rot([a.t[0] - o[0], a.t[1] - o[1], a.t[2] - o[2]]);
            a.t = [o[0] + d[0], o[1] + d[1], o[2] + d[2]];
        }
    }

    pub fn set_effect_lists(&mut self, lists: std::sync::Arc<Vec<Vec<neopets_data::region::EffectEntry>>>) {
        self.effect_lists = lists;
    }

    /// SPEC-0034 1.2 clip-to-list lookup: the main table entry's word A bits 14 to 21 name the
    /// region's effect list (0xFF none; an index past the table or the lists is none).
    pub fn effect_list(&self) -> Option<(usize, &[neopets_data::region::EffectEntry])> {
        let f = self.families.first()?;
        let a = *f.data.actions.get(self.action as usize)?;
        let i = ((a >> 14) & 0xFF) as usize;
        if i == 0xFF {
            return None;
        }
        self.effect_lists.get(i).map(|l| (i, l.as_slice()))
    }

    /// The main layer's cursor t, the clip's frame count D and the speed L in frames per tick.
    pub fn layer0_window(&self) -> Option<(f32, f32, f32)> {
        let f = self.families.first()?;
        let c = f.current?;
        Some((f.record.cursor(), f.data.clips[c].frames as f32, f.record.speed()))
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

    /// The main layer's clip for the block state (SPEC-0033 Behavior 4A.2): id, phase, cursor and
    /// key count.
    pub fn hold_clip(&self) -> neopets_sim::block::HoldClip {
        let record = self.families.first().map(|f| &f.record);
        neopets_sim::block::HoldClip {
            action: self.action,
            progress: record.map_or(1.0, |r| r.progress()),
            cursor: record.map_or(0.0, |r| r.cursor()),
            keys: record.and_then(|r| r.clip()).map_or(0, |c| c.frames as u32),
        }
    }

    /// Diagnostic: the main family's clip for an action: its key count, speed byte and the sum
    /// of its root records (the travel of one play at play rate 1 is this times 0.5 / speed).
    pub fn root_sum(&self, action: u32) -> Option<(usize, [f32; 3])> {
        let f = self.families.first()?;
        let c = &f.data.clips[f.data.clip_for(action)?];
        let mut sum = c.root.iter().fold([0.0f32; 3], |a, r| [a[0] + r[0], a[1] + r[1], a[2] + r[2]]);
        if f.data.actions.get(action as usize).is_some_and(|&a| a & anim::MIRROR_BIT != 0) {
            sum[0] = -sum[0];
        }
        Some((c.root.len(), sum))
    }

    /// The main family's table names a clip for the action (no lookup-miss fallback to clip 0;
    /// SPEC-0038 Layout section 6 and Behavior 13: a missing phase is skipped).
    pub fn has(&self, action: u32) -> bool {
        self.families.first().is_some_and(|f| f.data.named_clip(action).is_some())
    }

    /// Request an action id. A change of main clip restarts the main, sub-part and face clips;
    /// the same main clip already playing starts nothing (SPEC-0005, Clip start).
    pub fn play(&mut self, action: u32) {
        self.request(action, false);
    }

    /// Request an action, restarting its clip even when it is current when `force` is set (the
    /// forced start of SPEC-0040, used by chained attack repeats). The main clip starts unless
    /// its layer holds the same clip with the same mirror flag (word A bit 12; SPEC-0005 Clip
    /// start, Mirrored clips). Sub-part and face clips are never mirrored (none is on disc).
    pub fn request(&mut self, action: u32, force: bool) {
        self.action = action;
        // SPEC-0036: an id with no clip is not ignored; it plays clip 0 of the family.
        let main = self.families.first().filter(|f| !f.data.clips.is_empty()).map(|f| f.data.clip_for(action).unwrap_or(0));
        let mirror = self.families.first().and_then(|f| f.data.actions.get(action as usize)).is_some_and(|&a| a & anim::MIRROR_BIT != 0);
        let restart = main.is_some() && (force || main != self.families[0].current || mirror != self.families[0].mirror);
        if !restart {
            return;
        }
        let face = self.face;
        for f in &mut self.families {
            // The face clip is the override's when one is set, else the main id's.
            let id = if f.role == 2 { face.unwrap_or(action) } else { action };
            if f.data.clips.is_empty() {
                continue;
            }
            let c = f.data.clip_for(id).unwrap_or(0);
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
            f.mirror = f.role == 0 && mirror;
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
        self.face_step();
        let mut root = [0.0; 3];
        let sel = self.scale_set;
        for (i, f) in self.families.iter_mut().enumerate() {
            let (Some(c), Some(s)) = (f.current, f.record.tick()) else { continue };
            if i == 0 {
                let clip = &f.data.clips[c];
                let scale = f.record.speed() * if f.record.finished() { 2.0 } else { 1.0 };
                // SPEC-0005 Root step in modes 0 and 1: the next key's root, set only while k < k2; otherwise the step is
                // kept, and a finished clip doubles it in place each tick (the Flower Monsters' confused clip). The clamp
                // stands in for the mesh border before the value overflows.
                if clip.mode != 2 {
                    if s.key < s.next_key {
                        f.kept = clip.root.get(s.next_key as usize).map_or([0.0; 3], |r| r.map(|v| v * f.record.speed()));
                    }
                    if f.record.finished() {
                        f.kept = f.kept.map(|v| (v * 2.0).clamp(-1.0e4, 1.0e4));
                    }
                    root = f.kept;
                    if f.mirror {
                        root[0] = -root[0];
                    }
                } else if let Some(r) = clip.root.get(s.key as usize) {
                    root = r.map(|v| v * scale);
                    // A mirrored clip's root step has its x negated (Playback 5).
                    if f.mirror {
                        root[0] = -root[0];
                    }
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
                // Only main-family samples are scaled.
                &|j| if i == 0 { f.data.def.translation_scale(j, sel) } else { 1.0 },
                f.mirror.then_some(f.data.def.partners.as_slice()),
            );
        }
        root
    }

    /// Set the main family's translation scale selector (kind-3 data +0x8C).
    pub fn set_scale_set(&mut self, sel: i16) {
        self.scale_set = sel;
    }

    /// Set the chain mode's chains. A link whose joint does not hang from its link joint in the
    /// sub-part skeleton is left out; returns how many were left out.
    pub fn set_chains(&mut self, chains: Vec<neopets_sim::chain::Chain>) -> usize {
        let Some(sub) = self.families.get(1) else {
            self.chains.clear();
            return chains.iter().map(Vec::len).sum();
        };
        let n_main = self.families[0].data.def.len() as u32;
        let parents = &sub.data.def.parents;
        let mut dropped = 0;
        self.chains = chains
            .into_iter()
            .map(|c| {
                c.into_iter()
                    .filter(|l| {
                        let ok = parents.get(l.joint).copied().flatten() == Some(n_main + l.hangs_from as u32);
                        dropped += usize::from(!ok);
                        ok
                    })
                    .collect()
            })
            .collect();
        self.last_owner = None;
        dropped
    }

    /// The chain step after the pose (SPEC-0040 Behavior 25, Per-tick step): the links of each
    /// chain in order, each from its link joint's world frame as the pose (with the links
    /// already stepped) gives it, with the owner placed at `position` facing `facing`. Each
    /// link's joint takes its yaw and pitch as local rotation and (0, 0, -a) as translation.
    pub fn chain_step(&mut self, position: [f32; 3], facing: [f32; 3]) {
        use neopets_sim::chain::{Frame, link_rotation, motion_unit, step_link};
        if self.chains.is_empty() || self.families.len() < 2 {
            return;
        }
        let moved = self.last_owner.map_or([0.0; 3], |l| std::array::from_fn(|i| position[i] - l[i]));
        let u = motion_unit(moved);
        self.last_owner = Some(position);
        // Characters face +Z in model space: the owner's frame turns +Z onto the facing.
        let (s, c) = facing[0].atan2(facing[2]).sin_cos();
        let root = Affine { cols: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], t: position };
        let base = self.families[1].skin_base;
        let mut chains = std::mem::take(&mut self.chains);
        for link in chains.iter_mut().flatten() {
            let Some(w) = self.bone_frames().get(base + link.hangs_from).copied() else { continue };
            let m = root.mul(&w);
            let f = Frame { x: m.cols[0], y: m.cols[1], z: m.cols[2], p: m.t };
            let (theta, phi) = step_link(link, &f, u);
            if let Some(j) = self.families[1].joints.get_mut(link.joint) {
                *j = Some((link_rotation(theta, phi), [0.0, 0.0, -link.k.a]));
            }
        }
        self.chains = chains;
    }

    /// Diagnostic: per main joint, (joint, parent, sampled translation, rest local translation).
    pub fn main_translations(&self) -> Vec<(usize, Option<u32>, Option<[f32; 3]>, [f32; 3])> {
        let Some(f) = self.families.first() else { return Vec::new() };
        (0..f.joints.len())
            .map(|i| (i, f.data.def.parents.get(i).copied().flatten(), f.joints[i].map(|j| j.1), self.bones.rest_local.get(i).copied().unwrap_or_default()))
            .collect()
    }

    /// The bone table the animation poses.
    pub fn bones(&self) -> &BoneTable {
        &self.bones
    }

    /// The face-id setter: writes only the override (its clip starts on the next step).
    pub fn set_face(&mut self, id: Option<u32>) {
        self.face = id;
    }

    fn start_face(f: &mut FamilyState, id: u32) {
        let Some(c) = f.data.clip_for(id) else { return };
        let clip = &f.data.clips[c];
        f.record.start(ClipInfo { frames: clip.frames as u32, looping: clip.looping, speed_byte: clip.speed_byte, fade_in: clip.fade_in }, 1.0);
        f.current = Some(c);
    }

    /// Behavior 3D step 6: the override's face clip starts unless the face record holds it;
    /// once it holds it at progress 0.99 or more, the override is cleared and the main id's face
    /// clip starts at once.
    fn face_step(&mut self) {
        let (Some(id), action) = (self.face, self.action) else { return };
        let Some(f) = self.families.iter_mut().find(|f| f.role == 2) else { return };
        let wanted = f.data.clip_for(id);
        if wanted.is_some() && f.current != wanted {
            Self::start_face(f, id);
        } else if wanted.is_some() && f.record.progress() >= 0.99 {
            self.face = None;
            Self::start_face(f, action);
        }
    }

    /// The main clip's and the face clip's progress and loop flags (SPEC-0040 Behavior 22B).
    pub fn emote_view(&self) -> neopets_sim::emote::Actor {
        let state = |role: u8| {
            self.families.iter().find(|f| f.role == role).map_or((1.0, false), |f| {
                let looping = f.current.is_some_and(|c| f.data.clips[c].looping);
                (if f.record.finished() { 1.0 } else { f.record.progress() }, looping)
            })
        };
        let (mp, ml) = state(0);
        let (fp, fl) = state(2);
        neopets_sim::emote::Actor {
            main: self.action as i32,
            main_progress: mp,
            main_looping: ml,
            face: self.face.map_or(-1, |f| f as i32),
            face_progress: fp,
            face_looping: fl,
        }
    }

    /// The packed sound word (word B) of an action in the main family's table (SPEC-0031
    /// section 10).
    pub fn sound_word(&self, action: u32) -> Option<u32> {
        self.families.first()?.data.sounds.get(action as usize).copied()
    }

    /// The blocks the first 167 entries of the main table name (creature-block references).
    pub fn sound_blocks(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.families.first().map_or(&[][..], |f| &f.data.sounds[..]).iter().take(167).filter_map(|&w| sound_block(w)).collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The layer-0 family's playing clip and the integer part of its frame cursor (the footstep
    /// rule's remembered frame, SPEC-0031).
    pub fn layer0(&self) -> Option<(usize, i32)> {
        let f = self.families.first()?;
        Some((f.current?, f.record.cursor() as i32))
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
        // A stand-in with no families (placeholder models) has no bones to pose.
        let Some(main) = self.families.first() else { return Vec::new() };
        let others: Vec<Family> = self.families.iter().skip(1).map(family).collect();
        let mut m = anim::skin_matrices(&self.bones, &family(main), &others);
        if let Some(p) = self.look {
            self.apply_look(&mut m, p);
        }
        if let Some((joints, angles)) = &self.neck {
            let parents = self.skin_parents();
            for (&j, &a) in joints.iter().zip(angles) {
                if a != 0.0 {
                    self.turn_about_up(&mut m, &parents, j, a);
                }
            }
        }
        m
    }

    pub fn status(&self) -> String {
        let Some(main) = self.families.first() else { return "Animation: no families".into() };
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

/// The block a packed sound word names: bits 0 to 7 the index (255 none), bits 8 to 10 the group
/// (0: 347 + index, 1: 350 + index, 2: 395 + index).
pub fn sound_block(w: u32) -> Option<u32> {
    let index = w & 0xFF;
    if index == 255 {
        return None;
    }
    match (w >> 8) & 7 {
        0 => Some(347 + index),
        1 => Some(350 + index),
        2 => Some(395 + index),
        _ => None,
    }
}

/// The cue of a packed sound word (bits 11 to 19; 511 none) and its keep-alive bit (31).
pub fn sound_cue(w: u32) -> Option<(i32, bool)> {
    let cue = (w >> 11) & 0x1FF;
    (cue != 511).then_some((cue as i32, w & 0x8000_0000 != 0))
}

#[cfg(test)]
mod sound_word_tests {
    use super::*;

    #[test]
    fn packed_sound_word() {
        // 0x80002101: index 1, group 1, cue 4, keep-alive: block 351, cue 4.
        assert_eq!(sound_block(0x8000_2101), Some(351));
        assert_eq!(sound_cue(0x8000_2101), Some((4, true)));
        assert_eq!(sound_block(0x0000_21FF), None);
    }
}
