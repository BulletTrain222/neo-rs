//! Emote clip lookup (SPEC-0009 Emote clip lookup; SPEC-0022 Dialogue control records): the
//! request kinds of the dialogue's control records, the row's roll and random step, and the
//! directory position: index + 74 x voice set for a partner body emote, index + step + 183 for
//! Roberta's body emotes, index + step for hero face emotes.

/// Partner body emote (0x10), hero body emote (0x17), hero face emote (0x18), partner face emote
/// with no clip (0x16).
pub const PARTNER_BODY: u8 = 0x10;
pub const HERO_BODY: u8 = 0x17;
pub const HERO_FACE: u8 = 0x18;
pub const PARTNER_FACE: u8 = 0x16;

/// A row of table 7 or 8: clip index (255 none), chance, variance byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Row {
    pub clip: u8,
    pub chance: u8,
    pub variance: u8,
}

/// The directory position a request gives (None: no clip), with the draws taken from `draw`
/// (a value below the bound it is given): the roll (0 to 99), then the step (0 to 2) when the
/// row has a variance byte. `roberta` selects the hero shift for body emotes.
pub fn position(kind: u8, row: Option<Row>, voice_set: i16, roberta: bool, draw: &mut dyn FnMut(u32) -> u32) -> Option<i32> {
    if kind == PARTNER_FACE {
        return None;
    }
    let row = row?;
    if draw(100) >= row.chance as u32 {
        return None;
    }
    let step = if row.variance != 0 { draw(3) as i32 - 1 } else { 0 };
    if row.clip == 255 {
        return None;
    }
    let index = row.clip as i32 + step;
    if index == -1 {
        return None;
    }
    Some(match kind {
        PARTNER_BODY => index + 74 * voice_set as i32,
        HERO_BODY => index + if roberta { 183 } else { 0 },
        _ => index,
    })
}

/// The playback bound: a position plays only when it is below the directory's entry count as an
/// unsigned comparison (a negative position is past the end) and its length is not zero.
pub fn playable(position: i32, directory: &[(u32, u32)]) -> Option<(u32, u32)> {
    let e = *directory.get(usize::try_from(position).ok()?)?;
    (e.1 > 0).then_some(e)
}

/// A queued emote (SPEC-0040 Behavior 22): its clip (an opaque reference the caller resolved at
/// request time, e.g. bank and directory entry) and its emote id (an animation id; -2 returns
/// to idle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry<C> {
    pub clip: Option<C>,
    pub id: i32,
}

/// The actor's animation as the consumer reads it: the main id with its clip's progress and loop
/// flag, and the face override id (-1 none) with the face clip's progress and loop flag.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Actor {
    pub main: i32,
    pub main_progress: f32,
    pub main_looping: bool,
    pub face: i32,
    pub face_progress: f32,
    pub face_looping: bool,
}

/// What a consumer run asks of the actor: a main request (main id B with face override F), or a
/// face override set alone; and the clip to start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Consumed<C> {
    pub request_main: Option<i32>,
    pub face: i32,
    pub clip: Option<C>,
}

/// An actor's body and face queues: four entries each, the oldest dropped when full.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queues<C> {
    pub body: Vec<Entry<C>>,
    pub face: Vec<Entry<C>>,
}

impl<C> Default for Queues<C> {
    fn default() -> Self {
        Self { body: Vec::new(), face: Vec::new() }
    }
}

impl<C: Copy> Queues<C> {
    /// Add a request: body kinds (0x10, 0x17) to the body queue, face kinds (0x18, 0x16) to the
    /// face queue.
    pub fn push(&mut self, kind: u8, entry: Entry<C>) {
        let q = if matches!(kind, PARTNER_BODY | HERO_BODY) { &mut self.body } else { &mut self.face };
        if q.len() >= 4 {
            q.remove(0);
        }
        q.push(entry);
    }

    /// Clear both queues (spawn, and the start of a conversation).
    pub fn clear(&mut self) {
        self.body.clear();
        self.face.clear();
    }

    /// One consumer run (in the talking state). `body` is whether body emotes are enabled.
    pub fn consume(&mut self, a: Actor, body: bool) -> Consumed<C> {
        // The pop test of a head against the playing id.
        fn pops<C>(q: &[Entry<C>], playing: i32, looping: bool, progress: f32) -> bool {
            match q.first().map(|e| e.id) {
                None | Some(-1) => false,
                Some(-2) => true,
                Some(e) => e == playing && ((looping && q.len() >= 2) || (!looping && progress >= 0.99)),
            }
        }
        if body && pops(&self.body, a.main, a.main_looping, a.main_progress) {
            self.body.remove(0);
        }
        let b = self.body.first().map_or(0, |e| e.id);
        let mut f = self.face.first().map_or(0, |e| e.id);
        if pops(&self.face, a.face, a.face_looping, a.face_progress) {
            let was = self.face.remove(0).id;
            f = match self.face.first() {
                Some(e) => e.id,
                None if was == -2 => 0,
                None => -1,
            };
        }
        let face_clip = self.face.first().and_then(|e| e.clip);
        if !body || a.main == b {
            let clip = if a.face != f { face_clip } else { None };
            Consumed { request_main: None, face: f, clip }
        } else {
            Consumed { request_main: Some(b), face: f, clip: face_clip.or(self.body.first().and_then(|e| e.clip)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(values: Vec<u32>) -> impl FnMut(u32) -> u32 {
        let mut it = values.into_iter();
        move |_| it.next().unwrap_or(0)
    }

    #[test]
    fn spec_vectors() {
        let always = |clip: u8, variance: u8| Some(Row { clip, chance: 100, variance });
        // Character emote, clip 5, voice set 2: 5 + 74 x 2 = 153.
        assert_eq!(position(PARTNER_BODY, always(5, 0), 2, false, &mut fixed(vec![0])), Some(153));
        // Hero body, clip 10, step -1 (draw 0), Roberta: 10 - 1 + 183 = 192; Tor, no step: 10.
        assert_eq!(position(HERO_BODY, always(10, 13), -1, true, &mut fixed(vec![0, 0])), Some(192));
        assert_eq!(position(HERO_BODY, always(10, 0), -1, false, &mut fixed(vec![0])), Some(10));
        // Hero face, the same row with step -1 for Roberta: 9 (no shift).
        assert_eq!(position(HERO_FACE, always(10, 13), -1, true, &mut fixed(vec![0, 0])), Some(9));
        // Chance 0 never plays (no second draw); chance 100 plays for roll 99; 255 no clip.
        assert_eq!(position(HERO_BODY, Some(Row { clip: 10, chance: 0, variance: 1 }), -1, false, &mut fixed(vec![0])), None);
        assert_eq!(position(PARTNER_BODY, always(5, 0), 0, false, &mut fixed(vec![99])), Some(5));
        assert_eq!(position(PARTNER_BODY, always(255, 0), 0, false, &mut fixed(vec![0])), None);
        // No voice set: clip 36 gives -38 (past the end), clip 74 gives 0.
        let dir = vec![(0u32, 4u32), (4, 0)];
        assert_eq!(position(PARTNER_BODY, always(36, 0), -1, false, &mut fixed(vec![0])), Some(-38));
        assert_eq!(playable(-38, &dir), None);
        assert_eq!(playable(position(PARTNER_BODY, always(74, 0), -1, false, &mut fixed(vec![0])).unwrap(), &dir), Some((0, 4)));
        // A zero-length position plays nothing; the partner face kind never has a clip.
        assert_eq!(playable(1, &dir), None);
        assert_eq!(position(PARTNER_FACE, always(5, 0), 0, false, &mut fixed(vec![0])), None);
    }

    #[test]
    fn queues_and_consumer() {
        let e = |id: i32, clip: Option<u8>| Entry { clip, id };
        let mut q: Queues<u8> = Queues::default();
        // Four entries at most; the oldest is dropped.
        for id in 1..=5 {
            q.push(PARTNER_BODY, e(id, None));
        }
        assert_eq!(q.body.iter().map(|x| x.id).collect::<Vec<_>>(), vec![2, 3, 4, 5]);
        q.clear();
        // A body emote 51 with a clip: the first run requests main 51 (face 0) and starts it.
        q.push(PARTNER_BODY, e(51, Some(7)));
        let idle = Actor { main: 0, face: -1, ..Default::default() };
        assert_eq!(q.consume(idle, true), Consumed { request_main: Some(51), face: 0, clip: Some(7) });
        // While 51 plays below 0.99 nothing changes; at 0.99 (one-shot) it pops and B is 0.
        let playing = Actor { main: 51, main_progress: 0.5, face: 0, ..Default::default() };
        assert_eq!(q.consume(playing, true), Consumed { request_main: None, face: 0, clip: None });
        let done = Actor { main_progress: 0.99, ..playing };
        assert_eq!(q.consume(done, true), Consumed { request_main: Some(0), face: 0, clip: None });
        assert!(q.body.is_empty());
        // A face emote 126 with a clip: the face override is set and its clip starts once.
        q.push(HERO_FACE, e(126, Some(9)));
        let a = Actor { main: 0, face: 0, ..Default::default() };
        assert_eq!(q.consume(a, true), Consumed { request_main: None, face: 126, clip: Some(9) });
        let a = Actor { main: 0, face: 126, face_progress: 0.3, ..Default::default() };
        assert_eq!(q.consume(a, true), Consumed { request_main: None, face: 126, clip: None });
        // Its end pops it and leaves an empty head: F = -1.
        let a = Actor { main: 0, face: 126, face_progress: 0.99, ..Default::default() };
        assert_eq!(q.consume(a, true).face, -1);
        // Body and face clips in one tick: the face clip plays.
        q.push(PARTNER_BODY, e(40, Some(1)));
        q.push(HERO_FACE, e(127, Some(2)));
        assert_eq!(q.consume(Actor { main: 0, face: -1, ..Default::default() }, true).clip, Some(2));
        // Body emotes disabled (a busy NPC): no main request; the face path still runs.
        let mut r: Queues<u8> = Queues::default();
        r.push(PARTNER_BODY, e(40, Some(1)));
        assert_eq!(r.consume(Actor { main: 70, face: 0, ..Default::default() }, false), Consumed { request_main: None, face: 0, clip: None });
        // -2 returns to idle: popped at once.
        let mut s: Queues<u8> = Queues::default();
        s.push(HERO_BODY, e(-2, None));
        s.push(HERO_BODY, e(30, None));
        assert_eq!(s.consume(Actor { main: 0, face: 0, ..Default::default() }, true).request_main, Some(30));
    }
}
