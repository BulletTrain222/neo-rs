//! The follow camera, mode 0 (SPEC-0027): profile tuning, right-stick spin and elevation, the
//! target distance, the eye on its sphere about the focus, the critically damped spring, the
//! chase and manual elevation, ground clearance, the eye wall push (through a caller hook) and
//! the R3 recentre, plus the load placement with the pull-in setter.
//! Profiles are selected by the ground kind under the hero; the line-of-sight timer and the
//! recentre's line test use the caller's segment query (`clear`).
//! The hold and the side lean frame the hero's target (the host's point). Not covered: the script
//! override profile, the turn leans of states 0x12 and 0x17, the spring's contact-normal exception, the eye pass failure flag and
//! stuck timer, the camera visibility test, scripted plays, shakes, the fade ramp and modes
//! other than 0.

use crate::hero::Vec3;

/// SPEC-0027 Projection: focal length 512 on a 640 × 480 canvas, near 1, far 5280.
pub const FOCAL: f32 = 512.0;
pub const SCREEN: (f32, f32) = (640.0, 480.0);
pub const NEAR: f32 = 1.0;
pub const FAR: f32 = 5280.0;

/// The vertical field of view in radians: 2 atan(H / 2S), 50.2 degrees.
pub fn vertical_fov() -> f32 {
    2.0 * (SCREEN.1 / (2.0 * FOCAL)).atan()
}

/// One camera profile (SPEC-0027 Camera profile record): follow distance, default elevation,
/// spring stiffness, spin gain and decay, minimum eye height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub distance: f32,
    pub elevation: f32,
    pub stiffness: f32,
    pub spin_gain: f32,
    pub spin_decay: f32,
}

impl Profile {
    /// Profile 0, the main camera.
    pub const MAIN: Profile = Profile { distance: 10.0, elevation: 0.2, stiffness: 0.085, spin_gain: 0.005, spin_decay: 0.125 };

    /// Profiles 0 to 3 (main, interior, small room, special); only distance and elevation
    /// differ.
    pub fn table(index: usize) -> Profile {
        let (distance, elevation) = [(10.0, 0.2), (9.0, 0.25), (10.0, 0.2), (15.0, 0.6)][index.min(3)];
        Profile { distance, elevation, ..Self::MAIN }
    }

    /// The profile chosen by the ground kind (entry +0x38) under the hero: 3 gives 2, 4 gives
    /// 1, 5 or 6 give 3, any other value 0.
    pub fn for_ground(kind: u32) -> usize {
        match kind {
            3 => 2,
            4 => 1,
            5 | 6 => 3,
            _ => 0,
        }
    }

    fn damping(&self) -> f32 {
        2.0 * self.stiffness.sqrt()
    }
}

/// Ground clearance above the eye's ground (0x2DB3C4) and the 0.65 distance factor (0x2DB3F0).
const CLEARANCE: f32 = 2.0;
const DISTANCE_SHRINK: f32 = 0.65;
/// The eye's push radius (0x2DB400), used by the caller's wall hook.
pub const EYE_RADIUS: f32 = 1.25;

/// Polar (r, yaw, elevation) of an offset (SPEC-0027 Polar convention).
pub fn polar(v: Vec3) -> [f32; 3] {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if r == 0.0 {
        return [0.0; 3];
    }
    [r, v[2].atan2(v[0]), std::f32::consts::FRAC_PI_2 - (v[1] / r).clamp(-1.0, 1.0).acos()]
}

/// The inverse: (r cos yaw cos e, r sin e, r sin yaw cos e).
pub fn sphere(p: [f32; 3]) -> Vec3 {
    let [r, yaw, e] = p;
    [r * yaw.cos() * e.cos(), r * e.sin(), r * yaw.sin() * e.cos()]
}

fn wrap(a: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut a = a % TAU;
    if a > PI {
        a -= TAU;
    } else if a < -PI {
        a += TAU;
    }
    a
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Converter A (SPEC-0027 Pad-axis converters): centre 127.5, dead zone 40, range ±87.5.
pub fn converter_a(byte: u8) -> f32 {
    let d = byte as f32 - 127.5;
    if d > 40.0 {
        d - 40.0
    } else if d < -40.0 {
        d + 40.0
    } else {
        0.0
    }
}

/// The camera's pad input for one tick: the right-stick bytes (0 to 255; X right = 255, Y up =
/// 0) and the R3 held and pressed bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPad {
    pub right_x: u8,
    pub right_y: u8,
    pub r3_held: bool,
    pub r3_pressed: bool,
}

impl Default for CameraPad {
    fn default() -> Self {
        Self { right_x: 128, right_y: 128, r3_held: false, r3_pressed: false }
    }
}

/// What the camera reads of the followed hero (its previous tick's pose).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Followed {
    pub position: Vec3,
    pub facing: Vec3,
    pub height: f32,
    /// Recentring gate T: the hero moved in its last motion step and is not in state 1.
    pub moving: bool,
    /// Ground kind of the patch the hero stands on (None: no patch this tick).
    pub ground_kind: Option<u32>,
    /// A camera-locked hero state (Follow camera step 9), and the hero's last step length.
    pub locked: bool,
    pub step: f32,
}

impl Followed {
    fn focus(&self) -> Vec3 {
        add(self.position, [0.0, self.height, 0.0])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FollowCamera {
    pub profile: Profile,
    pub eye: Vec3,
    pub focus: Vec3,
    /// Target polar P and current polar C (spun yaw).
    pub target: [f32; 3],
    pub current: [f32; 3],
    velocity: Vec3,
    /// Spin value I (radians per tick).
    pub spin: f32,
    /// Elevation target t_e, manual elevation m and the sticky manual flag.
    pub elevation_target: f32,
    /// The negation test of the elevation target (the host's word; SPEC-0027).
    pub negate_elevation: bool,
    pub manual: f32,
    pub manual_flag: bool,
    side_offset: f32,
    /// The side target the offset glides toward (kept unchanged through the hold).
    side_target: f32,
    /// The blended transition (mode 8), while it runs.
    pub blend: Option<Blend>,
    /// The hold counter, and the hero's target (the host's point; SPEC-0027 Hold and lean terms).
    hold: u32,
    pub target_point: Option<Vec3>,
    r3_latch: bool,
    profile_index: Option<usize>,
    /// Consecutive ticks with the eye-to-focus line blocked.
    blocked: u32,
}

impl FollowCamera {
    /// The attach of SPEC-0027 Placement (boot and world load, hook flag 1): the eye at the
    /// position minus d times the facing, through the pull-in setter, then camera reset.
    /// `ground(x, z, ceiling)` is the SPEC-0018 probe; `push` is the eye wall push.
    pub fn attach(hero: &Followed, ground: impl Fn(f32, f32, f32) -> f32, push: impl Fn(&mut Vec3)) -> Self {
        let mut cam = Self {
            profile: Profile::MAIN,
            eye: [0.0; 3],
            focus: hero.focus(),
            target: [0.0; 3],
            current: [0.0; 3],
            velocity: [0.0; 3],
            spin: 0.0,
            elevation_target: 0.0,
            negate_elevation: false,
            manual: 0.0,
            manual_flag: false,
            side_offset: 0.0,
            side_target: 0.0,
            blend: None,
            hold: 0,
            target_point: None,
            r3_latch: false,
            profile_index: None,
            blocked: 0,
        };
        cam.select_profile(hero);
        let d = cam.target_distance(0.0);
        let f = hero.facing;
        let wanted = sub(hero.position, [d * f[0], 0.0, d * f[2]]);
        cam.pull_in(hero, wanted, ground, push);
        cam
    }

    /// The pull-in setter: the eye drops to the focus height, its horizontal offset is halved
    /// when the ground there rises more than 5 above the focus, then the wall push and reset.
    pub fn pull_in(&mut self, hero: &Followed, wanted: Vec3, ground: impl Fn(f32, f32, f32) -> f32, push: impl Fn(&mut Vec3)) {
        let f = hero.focus();
        let mut e = [wanted[0], f[1], wanted[2]];
        if ground(e[0], e[2], f[1] + 5.0) > f[1] + 5.0 {
            e = [f[0] + 0.5 * (e[0] - f[0]), f[1], f[2] + 0.5 * (e[2] - f[2])];
        }
        push(&mut e);
        self.eye = e;
        self.focus = f;
        self.reset();
    }

    /// The world-load attach and the return to normal (SPEC-0027 Placement after a load or
    /// restart, Pose setter with pull-in): the eye the target distance behind the hero's facing,
    /// then the pull-in (level with the focus, halved under high ground), the wall push and reset.
    /// The earlier 5 units at the feet started the camera low and looking up, then it crept back
    /// for about 200 ticks (user's report 26).
    pub fn place_behind(&mut self, hero: &Followed, ground: impl Fn(f32, f32, f32) -> f32, push: impl Fn(&mut Vec3)) {
        self.select_profile(hero);
        let d = self.target_distance(0.0);
        let f = hero.facing;
        let wanted = sub(hero.position, [d * f[0], 0.0, d * f[2]]);
        self.pull_in(hero, wanted, ground, push);
    }

    /// SetToNormalCamera(distance, angle, release) (SPEC-0027 Return to normal with an angle):
    /// eye = feet + distance x unit(A cos(angle) + B sin(angle)), A the facing and B its first row
    /// (0, 1, 0) x A, then the pull-in. Angle 0 is in front of the hero.
    pub fn place_orbit(&mut self, hero: &Followed, distance: f32, angle: f32, ground: impl Fn(f32, f32, f32) -> f32, push: impl Fn(&mut Vec3)) {
        self.select_profile(hero);
        let a = hero.facing;
        let b = [a[2], 0.0, -a[0]];
        let (c, s) = (angle.cos(), angle.sin());
        let w = [a[0] * c + b[0] * s, a[1] * c, a[2] * c + b[2] * s];
        let l = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt().max(1e-6);
        let wanted = [hero.position[0] + distance * w[0] / l, hero.position[1] + distance * w[1] / l, hero.position[2] + distance * w[2] / l];
        self.pull_in(hero, wanted, ground, push);
    }

    /// Camera reset: spin cleared, both polars from the eye, t_e and m its elevation.
    pub fn reset(&mut self) {
        self.spin = 0.0;
        let p = polar(sub(self.eye, self.focus));
        self.target = p;
        self.current = p;
        self.elevation_target = p[2];
        self.manual = p[2];
    }

    /// Target distance: profile distance × (1 − 0.65 sin² e) + side offset; e ≤ 0 drops the
    /// 0.65 term.
    /// SPEC-0027 Blended transition, the entry: the pivot is the follow focus; no placement, no pivot step.
    pub fn start_blend(&mut self, radius: f32, offset: f32, height: f32, fraction: f32) {
        self.blend = Some(Blend { pivot: self.focus, distance: (radius * radius + height * height).sqrt(), offset, elevation: (height / radius).asin(), fraction, latched: false });
    }

    /// Mode 8's update: the contact pass (a push latches the camera in place), then distance, yaw and
    /// elevation each blended by the fraction toward the hero's forward row plus the offset.
    pub fn blend_step(&mut self, facing: Vec3, push: impl Fn(&mut Vec3)) {
        let Some(mut b) = self.blend else { return };
        if b.latched {
            return;
        }
        let before = self.eye;
        push(&mut self.eye);
        if self.eye != before {
            b.latched = true;
            self.blend = Some(b);
            return;
        }
        let c = polar(sub(self.eye, b.pivot));
        let k = b.fraction;
        let mut yaw = facing[2].atan2(facing[0]) + b.offset;
        // The sign test, not a shortest arc: opposite signs shift the desired yaw by a full turn.
        if (c[1] < 0.0) != (yaw < 0.0) {
            yaw += if yaw < 0.0 { std::f32::consts::TAU } else { -std::f32::consts::TAU };
        }
        let mix = |old: f32, want: f32, snap: f32| {
            let new = old * (1.0 - k) + want * k;
            if (new - want).abs() < snap { want } else { new }
        };
        let now = [mix(c[0], b.distance, 0.1), mix(c[1], yaw, 1e-6), mix(c[2], b.elevation, 1e-6)];
        self.eye = add(b.pivot, sphere(now));
        self.focus = b.pivot;
        self.velocity = [0.0; 3];
        self.blend = Some(b);
    }

    pub fn target_distance(&self, e: f32) -> f32 {
        let shrink = if e > 0.0 { 1.0 - DISTANCE_SHRINK * e.sin() * e.sin() } else { 1.0 };
        self.profile.distance * shrink + self.side_offset
    }

    /// Step 2: the profile follows the ground kind; with no ground patch it is kept.
    fn select_profile(&mut self, hero: &Followed) {
        if let Some(kind) = hero.ground_kind {
            let index = Profile::for_ground(kind);
            if self.profile_index != Some(index) {
                self.profile_index = Some(index);
                self.profile = Profile::table(index);
            }
        }
    }

    /// One follow tick (SPEC-0027 Follow camera, steps 2 to 11). `clear(a, b)` is the segment
    /// query with the restriction flag set.
    pub fn update(
        &mut self,
        hero: &Followed,
        pad: CameraPad,
        ground: impl Fn(f32, f32, f32) -> f32,
        push: impl Fn(&mut Vec3),
        clear: impl Fn(Vec3, Vec3) -> bool,
    ) {
        self.select_profile(hero);
        // The camera's back direction, from the previous tick's rows.
        let back = sub(self.eye, self.focus);
        // Step 3: stick value s and spin input D (no leans yet).
        let s = -converter_a(pad.right_y) / 87.5;
        let b = -converter_a(pad.right_x) / 87.5;
        // The hold: a pushed stick re-arms it for 150 ticks, the side offset and lean cleared meanwhile.
        let base = b * b.abs();
        if base != 0.0 {
            self.hold = 150;
        }
        let mut lean = 0.0;
        if self.hold >= 1 {
            self.hold -= 1;
            self.side_offset = 0.0;
        } else {
            let (forward, first) = self.rows();
            let (l, side) = lean_terms(hero.position, self.target_point, forward, first);
            lean = l;
            self.side_target = side;
        }
        // The glide runs on every call, also in the hold, so a hold tick leaves the offset at
        // min(side target, 0.1) (SPEC-0027 Hold and lean terms).
        self.side_offset += (self.side_target - self.side_offset).clamp(-0.1, 0.1);
        let mut d = base + lean;
        d *= d.abs();
        if d > 0.99 {
            d = 1.3;
        } else if d < -0.99 {
            d = -1.3;
        }
        // Step 4: target distance from the elevation stored on the previous tick.
        self.target[0] = self.target_distance(self.current[2]);
        // Step 5: focus.
        self.focus = hero.focus();
        // Step 6: spin, then the eye's polar about the new focus with the yaw spun.
        let p = self.profile;
        self.spin = self.spin + d * p.spin_gain - self.spin * p.spin_decay;
        let mut c = polar(sub(self.eye, self.focus));
        c[1] = wrap(c[1] + self.spin);
        self.current = c;
        // Step 7: elevation target.
        if !hero.moving && self.manual_flag {
            self.manual_step(s);
        } else {
            self.chase_step(s, hero.moving);
        }
        // Step 8: the eye on its sphere (distance and elevation unchanged, yaw spun).
        self.eye = add(self.focus, sphere(c));
        // Step 9: in normal states the target yaw is the eye's own yaw; in a camera-locked state, the yaw
        // of the camera's back direction, and with no spin the eye moves rigidly with the hero.
        let v = sub(self.eye, self.focus);
        self.target[1] = v[2].atan2(v[0]);
        if hero.locked {
            self.target[1] = back[2].atan2(back[0]);
            let far = add(self.focus, sphere([self.target[0] + 2.0, self.target[1], self.target[2]]));
            if !clear(far, self.focus) && c[0] < self.target[0] {
                self.target[0] = c[0] + hero.step.max(0.1);
            }
            if d == 0.0 {
                self.eye = add(self.focus, sphere(self.target));
                self.velocity = [0.0; 3];
            }
        }
        // Step 10: spring toward T = F + sphere(P).
        let t = add(self.focus, sphere(self.target));
        let (k, damping) = (p.stiffness, p.damping());
        for i in 0..3 {
            self.velocity[i] += -k * (self.eye[i] - t[i]) - damping * self.velocity[i];
            self.eye[i] += self.velocity[i];
        }
        // Step 11: ground clearance, walls, R3 recentre.
        self.clear_ground(&ground);
        push(&mut self.eye);
        // Line-of-sight timer: the 61st consecutive blocked tick runs the recentre pass.
        if clear(self.eye, self.focus) {
            self.blocked = 0;
        } else {
            self.blocked += 1;
            if self.blocked > 60 {
                self.blocked = 0;
                self.recentre(hero, &ground, &push, &clear);
            }
        }
        if !pad.r3_held && self.r3_latch {
            self.r3_latch = false;
        }
        if pad.r3_pressed && !self.r3_latch {
            self.recentre(hero, &ground, &push, &clear);
        }
    }

    fn chase_step(&mut self, s: f32, moving: bool) {
        // The target is negated while the hero hangs under a panel (state 21, sub-mode 4).
        let goal = (self.profile.elevation + s) * if self.negate_elevation { -1.0 } else { 1.0 };
        let step = if s.abs() > 0.99 { 0.0175 } else { 0.01 };
        let diff = goal - self.elevation_target;
        self.elevation_target = if diff.abs() < step { goal } else { self.elevation_target + step * diff.signum() };
        if self.target[0] > 0.1 {
            self.target[2] = self.elevation_target;
            self.manual = self.elevation_target;
        }
        self.manual_flag = !moving && s != 0.0;
    }

    fn manual_step(&mut self, s: f32) {
        let k = if s.abs() > 0.99 { 0.0175 } else { 0.01 };
        if s > 0.0 {
            self.manual = if self.manual < 1.5 { self.manual + k * s * s.abs() } else { 1.5 };
        } else if s < 0.0 {
            self.manual = if self.manual > -1.5 { self.manual + k * s * s.abs() } else { -1.5 };
        }
        self.target[2] = self.manual;
        self.elevation_target = self.manual;
    }

    /// Ground clearance: the eye stays 2.0 above the ground under it, sliding toward the focus
    /// when the focus is higher, or rising straight up otherwise.
    fn clear_ground(&mut self, ground: &impl Fn(f32, f32, f32) -> f32) {
        let e = self.eye;
        let h = ground(e[0], e[2], e[1] + 2.5);
        if h < -1.0e8 || e[1] >= h + CLEARANCE {
            return;
        }
        let f = self.focus;
        if f[1] > e[1] && h + CLEARANCE < f[1] {
            let t = (h + CLEARANCE - e[1]) / (f[1] - e[1]);
            self.eye = add(e, sub(f, e).map(|c| c * t));
            let p = polar(sub(self.eye, f));
            self.current = p;
            self.target = p;
            self.elevation_target = p[2];
            self.manual = p[2];
        } else {
            self.eye[1] = h + CLEARANCE;
        }
    }

    /// The recentre pass: the eye behind the hero at the default elevation, or in front when
    /// the line from behind is blocked, shrunk toward the focus by 0.25 while its line is
    /// blocked or it is buried (ground more than 5 above the focus), then the wall push and
    /// camera reset. When both candidates are blocked the behind one is shrunk (Medium).
    pub fn recentre(
        &mut self,
        hero: &Followed,
        ground: &impl Fn(f32, f32, f32) -> f32,
        push: &impl Fn(&mut Vec3),
        clear: &impl Fn(Vec3, Vec3) -> bool,
    ) {
        self.r3_latch = true;
        self.manual_flag = false;
        let before = self.eye;
        let f = self.focus;
        let d = self.target_distance(self.current[2]);
        // The spring velocity is zeroed on every pass, before any test.
        self.velocity = [0.0; 3];
        let elevation = self.profile.elevation;
        self.elevation_target = elevation;
        // Either test blocks: the line from the focus, or the buried probe (ground more than 5
        // above the focus under a ceiling 5 above it).
        let blocked = |e: Vec3| !clear(e, f) || ground(e[0], e[2], f[1] + 5.0) > f[1] + 5.0;
        let flat = |sign: f32| [sign * d * hero.facing[0], 0.0, sign * d * hero.facing[2]];
        let front = {
            let q = polar(flat(1.0));
            [q[0], q[1], elevation]
        };
        // The shrink: a pass runs only while r is above 1.0 (a loop guard, not a floor), so the
        // last radius can be in (0.75, 1.0] and is not tested.
        let shrink = |mut p: [f32; 3]| {
            while p[0] > 1.0 {
                if !blocked(add(f, sphere(p))) {
                    break;
                }
                p[0] -= 0.25;
            }
            p
        };
        let behind = {
            let q = polar(flat(-1.0));
            [q[0], q[1], elevation]
        };
        let p = if !blocked(add(f, sphere(behind))) {
            shrink(behind)
        } else if !blocked(add(f, flat(1.0))) {
            // The front point is tested flat at the focus height; the elevated eye is used untested.
            front
        } else {
            shrink(behind)
        };
        let commit = |c: &mut Self, p: [f32; 3]| {
            c.eye = add(f, sphere(p));
            push(&mut c.eye);
            let v = sub(c.eye, f);
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() >= 0.1
        };
        // The second pass (the committed eye closer than 0.1 to the focus) tries the front
        // candidate only; if that fails too the remembered eye is restored. Reset always runs.
        if !commit(self, p) && !commit(self, shrink(front)) {
            self.eye = before;
        }
        self.reset();
    }

    /// The view rows: forward from the eye to the focus, and the row R = up × forward.
    pub fn rows(&self) -> (Vec3, Vec3) {
        let v = sub(self.focus, self.eye);
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(f32::MIN_POSITIVE);
        let f = v.map(|c| c / l);
        let r = [f[2], 0.0, -f[0]];
        (f, r)
    }
}

/// The blended transition's stored values (SPEC-0027 mode 8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    pub pivot: Vec3,
    distance: f32,
    offset: f32,
    elevation: f32,
    fraction: f32,
    latched: bool,
}

/// SPEC-0027 Hold and lean terms: the side lean (1 - d) / 2 x 2.5, negated when the target lies on the first
/// row's side, and the side target (|s| - 2.0, at most 10.0, while d is below 0.5). No target: both 0.
fn lean_terms(hero: Vec3, target: Option<Vec3>, forward: Vec3, first: Vec3) -> (f32, f32) {
    let Some(t) = target else { return (0.0, 0.0) };
    let v = [t[0] - hero[0], 0.0, t[2] - hero[2]];
    let l = (v[0] * v[0] + v[2] * v[2]).sqrt().max(1e-6);
    let u = [v[0] / l, 0.0, v[2] / l];
    let dot3 = |a: Vec3, b: Vec3| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let d = dot3(u, forward);
    let mut lean = (1.0 - d) / 2.0 * 2.5;
    if dot3(u, first) > 0.0 {
        lean = -lean;
    }
    let s = dot3(v, first).abs();
    let side = if d < 0.5 && s > 2.0 { (s - 2.0).min(10.0) } else { 0.0 };
    (lean, side)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: f32, _: f32, _: f32) -> f32 {
        -1.0e9
    }

    fn still(height: f32) -> Followed {
        Followed { position: [0.0; 3], facing: [0.0, 0.0, 1.0], height, moving: false, ground_kind: None, locked: false, step: 0.0 }
    }

    fn camera_at(eye: Vec3, hero: &Followed) -> FollowCamera {
        let mut c = FollowCamera::attach(hero, none, |_| {});
        c.eye = eye;
        c.focus = hero.focus();
        c.reset();
        c
    }

    #[test]
    fn polar_matches_spec_vector() {
        let p = polar([0.0, 3.9357 - 2.0, -9.5493]);
        assert!((p[0] - 9.7435).abs() < 1e-3 && (p[1] + std::f32::consts::FRAC_PI_2).abs() < 1e-5 && (p[2] - 0.2).abs() < 1e-4, "{p:?}");
        let s = sphere(p);
        assert!((s[1] - 1.9357).abs() < 1e-3 && (s[2] + 9.5493).abs() < 1e-3);
    }

    #[test]
    fn projection_is_64_by_50_degrees() {
        assert!((vertical_fov().to_degrees() - 50.2).abs() < 0.05);
    }

    /// SPEC-0055: the death camera's distance 15.0748 and elevation 5.74 degrees (asin of 0.1) settle in.
    #[test]
    fn blend_settles() {
        let hero = still(2.0);
        let mut c = camera_at([0.0, 2.0, -10.0], &hero);
        c.start_blend(15.0, 0.0, 1.5, 0.01);
        for _ in 0..1500 {
            c.blend_step([0.0, 0.0, 1.0], |_| {});
        }
        let p = polar(sub(c.eye, c.blend.unwrap().pivot));
        assert!((p[0] - 15.0748).abs() < 1e-3 && (p[2] - 0.1f32.asin()).abs() < 1e-3, "{p:?}");
        assert!(p[1].abs() > 0.0);
    }

    /// SPEC-0027's vectors: hero at the origin, camera forward +z, first row +x.
    #[test]
    fn lean_vectors() {
        let (f, x) = ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let (l, s) = lean_terms([0.0; 3], Some([3.0, 0.0, 4.0]), f, x);
        assert!((l + 0.25).abs() < 1e-5 && s == 0.0);
        let (l, s) = lean_terms([0.0; 3], Some([5.0, 0.0, 1.0]), f, x);
        assert!((l + 1.004855).abs() < 1e-4 && (s - 3.0).abs() < 1e-5, "{l} {s}");
        assert_eq!(lean_terms([0.0; 3], None, f, x), (0.0, 0.0));
    }

    #[test]
    fn target_distance_values() {
        let c = camera_at([0.0, 2.0, -10.0], &still(2.0));
        assert!((c.target_distance(0.2) - 9.7434).abs() < 1e-3);
        assert!((c.target_distance(1.2) - 4.3535).abs() < 1e-3);
        assert!((c.target_distance(-0.1) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn settles_after_a_restart_placement() {
        // SPEC-0027: eye (0, 2.5, -10), focus (0, 2, 0); (r, e) after ticks 1, 5, 10, 20, 30, 60.
        let hero = still(2.0);
        let mut c = camera_at([0.0, 2.5, -10.0], &hero);
        let want = [(1, 10.0100, 0.0508), (5, 9.9955, 0.0652), (10, 9.9740, 0.1007), (20, 9.8865, 0.1774), (30, 9.7914, 0.1965), (60, 9.7439, 0.2000)];
        let mut got = Vec::new();
        for t in 1..=60 {
            c.update(&hero, CameraPad::default(), none, |_| {}, |_, _| true);
            got.push((t, polar(sub(c.eye, c.focus))));
        }
        for (t, r, e) in want {
            let p = got[t - 1].1;
            assert!((p[0] - r).abs() < 2e-3 && (p[2] - e).abs() < 2e-3, "tick {t}: {p:?} vs ({r}, {e})");
        }
        assert!((c.eye[1] - 3.9357).abs() < 2e-3 && (c.eye[2] + 9.5493).abs() < 2e-3, "{:?}", c.eye);
    }

    #[test]
    fn spin_follows_spec_rates() {
        // Profile 0, D = -1.3 held (right stick fully right).
        let hero = still(2.0);
        let mut c = camera_at([0.0, 3.9357, -9.5493], &hero);
        let pad = CameraPad { right_x: 255, ..Default::default() };
        let mut spins = Vec::new();
        for _ in 0..60 {
            c.update(&hero, pad, none, |_| {}, |_, _| true);
            spins.push(c.spin);
        }
        for (t, want) in [(1, -0.0065), (2, -0.012188), (6, -0.028663), (20, -0.048401), (60, -0.051983)] {
            assert!((spins[t - 1] - want).abs() < 1e-5, "tick {t}: {}", spins[t - 1]);
        }
    }

    #[test]
    fn chase_and_manual_elevation() {
        let mut c = camera_at([0.0, 2.0, -10.0], &still(2.0));
        c.elevation_target = 0.0;
        c.target[0] = 10.0;
        // Chase toward 1.0 (profile 0.2 + stick 0.8), moving: 0.01 per tick.
        c.chase_step(0.8, true);
        assert!((c.elevation_target - 0.01).abs() < 1e-6);
        for _ in 1..99 {
            c.chase_step(0.8, true);
        }
        assert!((c.elevation_target - 0.99).abs() < 1e-4);
        // Manual overshoot: from 1.499 with a full push, 1.5165 then 1.5.
        c.manual = 1.499;
        c.manual_step(1.0);
        assert!((c.manual - 1.5165).abs() < 1e-5);
        c.manual_step(1.0);
        assert_eq!(c.manual, 1.5);
    }

    #[test]
    fn manual_raise_while_standing() {
        // Right stick Y full up while standing, profile 0: (e, r) at ticks 30, 60, 120.
        let hero = still(2.0);
        let mut c = camera_at([0.0, 3.9357, -9.5493], &hero);
        let pad = CameraPad { right_y: 0, ..Default::default() };
        let mut got = Vec::new();
        for _ in 0..120 {
            c.update(&hero, pad, none, |_| {}, |_, _| true);
            got.push(polar(sub(c.eye, c.focus)));
        }
        for (t, e, r) in [(30, 0.6185, 8.5087), (60, 1.1375, 5.3794), (120, 1.5, 3.5330)] {
            let p = got[t - 1];
            assert!((p[2] - e).abs() < 3e-3 && (p[0] - r).abs() < 3e-3, "tick {t}: {p:?}");
        }
    }

    #[test]
    fn pull_in_drops_to_focus_height_and_halves_under_high_ground() {
        // SPEC-0027 Placement: P (10, 0, 5), f (0.6, 0, 0.8), height H: the posed eye (4, ·, -3)
        // becomes (4, H, -3), or (7, H, 1) when the ground there is above H + 5.
        let hero = Followed { locked: false, step: 0.0, position: [10.0, 0.0, 5.0], facing: [0.6, 0.0, 0.8], height: 3.5, moving: false, ground_kind: None };
        let mut c = FollowCamera::attach(&hero, none, |_| {});
        c.pull_in(&hero, [4.0, 2.5, -3.0], none, |_| {});
        assert!((c.eye[0] - 4.0).abs() < 1e-5 && c.eye[1] == 3.5 && (c.eye[2] + 3.0).abs() < 1e-5);
        c.pull_in(&hero, [4.0, 2.5, -3.0], |_, _, _| 9.0, |_| {});
        assert!((c.eye[0] - 7.0).abs() < 1e-5 && (c.eye[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn profiles_follow_the_ground_kind() {
        assert_eq!([3, 4, 5, 6, 0, 7].map(Profile::for_ground), [2, 1, 3, 3, 0, 0]);
        let mut hero = still(2.0);
        let mut c = camera_at([0.0, 2.0, -10.0], &hero);
        hero.ground_kind = Some(5);
        c.update(&hero, CameraPad::default(), none, |_| {}, |_, _| true);
        assert_eq!((c.profile.distance, c.profile.elevation), (15.0, 0.6));
        // Profile 3 at elevation 0.6: target distance 11.8915.
        assert!((c.target_distance(0.6) - 11.8915).abs() < 1e-3);
        hero.ground_kind = None;
        c.update(&hero, CameraPad::default(), none, |_| {}, |_, _| true);
        assert_eq!(c.profile.distance, 15.0, "kept without a ground patch");
    }

    #[test]
    fn blocked_line_recentres_on_the_61st_tick() {
        let hero = still(2.0);
        let mut c = camera_at([3.0, 2.0, 9.0], &hero);
        for _ in 0..60 {
            c.update(&hero, CameraPad::default(), none, |_| {}, |a, _| a[2] < 0.0);
        }
        assert!(c.eye[2] > 0.0, "still in front after 60 blocked ticks");
        c.update(&hero, CameraPad::default(), none, |_| {}, |a, _| a[2] < 0.0);
        assert!(c.eye[2] < 0.0, "behind the hero after the 61st: {:?}", c.eye);
    }

    #[test]
    fn recentre_shrinks_while_the_line_is_blocked() {
        // Behind is clear only within 3.0; the front is always blocked.
        let hero = still(2.0);
        let mut c = camera_at([0.0, 2.0, 9.0], &hero);
        let f = c.focus;
        let clear = |a: Vec3, b: Vec3| {
            let v = sub(a, b);
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() <= 3.0 + 1e-4 && a[2] < 0.0
        };
        c.recentre(&hero, &none, &|_: &mut Vec3| {}, &clear);
        let r = polar(sub(c.eye, f))[0];
        assert!(r <= 3.0 + 1e-4 && r > 2.75, "{r}");
    }

    #[test]
    fn hold_glides_the_side_offset_to_at_most_its_step() {
        // SPEC-0027 Hold and lean terms: the glide runs in the hold too, toward the stale target.
        let hero = still(2.0);
        let mut c = camera_at([0.0, 4.0, -9.0], &hero);
        c.side_target = 3.0;
        c.hold = 10;
        c.update(&hero, CameraPad::default(), none, |_| {}, |_, _| true);
        assert!((c.side_offset - 0.1).abs() < 1e-6, "{}", c.side_offset);
    }
}
