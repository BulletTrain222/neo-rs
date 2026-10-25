//! World buffers (SPEC-0006, World buffers and transitions): which buffer a neighbour named by
//! the pending step streams into, and whether a world swap may search the other buffer.
//! Not covered: doors, interiors, replacement loads and movie returns.

/// The three world buffers: T (transition), E (exterior) and I (interior).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    Transition,
    Exterior,
    Interior,
}

/// Pending step 3: a transition target streams into T while E is current, any other target into
/// E while T is current; nothing streams in any other case (I current, or the target's own
/// buffer current).
pub fn stream_buffer(current: Buffer, target_is_transition: bool) -> Option<Buffer> {
    match (current, target_is_transition) {
        (Buffer::Exterior, true) => Some(Buffer::Transition),
        (Buffer::Transition, false) => Some(Buffer::Exterior),
        _ => None,
    }
}

/// World swap: the hero's ground search repeats in the other buffer when the current world has
/// no ground there, the current buffer is T or E, and the neighbour-world test passes.
pub fn may_swap(current: Buffer, neighbour_test: bool) -> bool {
    current != Buffer::Interior && neighbour_test
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_follow_the_buffer_rule() {
        assert_eq!(stream_buffer(Buffer::Exterior, true), Some(Buffer::Transition));
        assert_eq!(stream_buffer(Buffer::Transition, false), Some(Buffer::Exterior));
        assert_eq!(stream_buffer(Buffer::Exterior, false), None);
        assert_eq!(stream_buffer(Buffer::Transition, true), None);
        assert_eq!(stream_buffer(Buffer::Interior, true), None);
        assert!(may_swap(Buffer::Transition, true) && !may_swap(Buffer::Interior, true) && !may_swap(Buffer::Exterior, false));
    }
}
