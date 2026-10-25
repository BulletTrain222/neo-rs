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
}
