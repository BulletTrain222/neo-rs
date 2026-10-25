//! LZO1X decompression, from the public LZO1X format. Animation clips and two boot pictures
//! are stored as single LZO1X blocks (SPEC-0002, SPEC-0005). Every read and back-reference is
//! bounds-checked, and the output must be exactly the expected size.

/// Decode `input` into exactly `expected` bytes. Returns the bytes and how much input the
/// stream consumed (up to and including its end marker).
pub fn decompress(input: &[u8], expected: usize) -> Result<(Vec<u8>, usize), String> {
    let mut d = Decoder {
        input,
        ip: 0,
        out: Vec::with_capacity(expected),
        limit: expected,
    };
    d.run()?;
    if d.out.len() != expected {
        return Err(format!("LZO1X output is {} bytes, expected {expected}", d.out.len()));
    }
    Ok((d.out, d.ip))
}

struct Decoder<'a> {
    input: &'a [u8],
    ip: usize,
    out: Vec<u8>,
    limit: usize,
}

const M2_MAX_OFFSET: usize = 0x0800;

impl Decoder<'_> {
    fn byte(&mut self) -> Result<usize, String> {
        let b = *self.input.get(self.ip).ok_or("LZO1X input ends early")?;
        self.ip += 1;
        Ok(b as usize)
    }

    fn le16(&mut self) -> Result<usize, String> {
        Ok(self.byte()? | self.byte()? << 8)
    }

    /// A run length extended by zero bytes (each adds 255) and a final byte.
    fn extended(&mut self, base: usize) -> Result<usize, String> {
        let mut t = 0;
        loop {
            let b = self.byte()?;
            if b != 0 {
                return Ok(t + base + b);
            }
            t += 255;
            if t > self.limit {
                return Err("LZO1X run length exceeds the output size".into());
            }
        }
    }

    fn literals(&mut self, n: usize) -> Result<(), String> {
        let src = self
            .input
            .get(self.ip..self.ip + n)
            .ok_or("LZO1X literal run past the input")?;
        if self.out.len() + n > self.limit {
            return Err("LZO1X literals past the expected output size".into());
        }
        self.out.extend_from_slice(src);
        self.ip += n;
        Ok(())
    }

    /// Copy `n` bytes from `distance` back in the output (overlap allowed).
    fn copy_match(&mut self, distance: usize, n: usize) -> Result<(), String> {
        if distance == 0 || distance > self.out.len() {
            return Err("LZO1X match refers before the output start".into());
        }
        if self.out.len() + n > self.limit {
            return Err("LZO1X match past the expected output size".into());
        }
        let start = self.out.len() - distance;
        for i in 0..n {
            let b = self.out[start + i];
            self.out.push(b);
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), String> {
        let mut t;
        // Low two bits of the instruction byte two positions back: trailing literals (0–3).
        let mut first_literal_run = false;
        let mut pending_literals = None;
        let first = *self.input.first().ok_or("LZO1X input is empty")? as usize;
        if first > 17 {
            self.ip = 1;
            let n = first - 17;
            if n < 4 {
                pending_literals = Some(n);
            } else {
                self.literals(n)?;
                first_literal_run = true;
            }
        }
        loop {
            if let Some(n) = pending_literals.take() {
                // Trailing literals of the previous match, then the next instruction is a match.
                self.literals(n)?;
                t = self.byte()?;
            } else if first_literal_run {
                first_literal_run = false;
                t = self.byte()?;
                if t < 16 {
                    // After a literal run, a short match of 3 bytes with a far base.
                    let distance = 1 + M2_MAX_OFFSET + (t >> 2) + (self.byte()? << 2);
                    self.copy_match(distance, 3)?;
                    match self.trailing()? {
                        0 => continue,
                        n => {
                            pending_literals = Some(n);
                            continue;
                        }
                    }
                }
            } else {
                t = self.byte()?;
                if t < 16 {
                    // A literal run of t + 3 bytes (t = 0: extended).
                    let n = if t == 0 { self.extended(15)? } else { t };
                    self.literals(n + 3)?;
                    first_literal_run = true;
                    continue;
                }
            }
            // A match instruction t >= 16 (or t < 16 right after trailing literals).
            let (distance, len) = if t >= 64 {
                let distance = 1 + ((t >> 2) & 7) + (self.byte()? << 3);
                (distance, (t >> 5) - 1 + 2)
            } else if t >= 32 {
                let n = t & 31;
                let n = if n == 0 { self.extended(31)? } else { n };
                let distance = 1 + (self.le16()? >> 2);
                (distance, n + 2)
            } else if t >= 16 {
                let high = (t & 8) << 11;
                let n = t & 7;
                let n = if n == 0 { self.extended(7)? } else { n };
                let low = self.le16()? >> 2;
                if high == 0 && low == 0 {
                    // End of stream marker.
                    return Ok(());
                }
                (high + low + 0x4000, n + 2)
            } else {
                // A two-byte match after trailing literals.
                let distance = 1 + (t >> 2) + (self.byte()? << 2);
                (distance, 2)
            };
            self.copy_match(distance, len)?;
            match self.trailing()? {
                0 => {}
                n => pending_literals = Some(n),
            }
        }
    }

    /// The low two bits of the byte two positions back give 0–3 trailing literals.
    fn trailing(&self) -> Result<usize, String> {
        let at = self.ip.checked_sub(2).ok_or("LZO1X stream too short")?;
        Ok(self.input[at] as usize & 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-of-stream marker: M4 instruction 0x11 with a zero distance.
    const EOS: [u8; 3] = [0x11, 0x00, 0x00];

    #[test]
    fn decodes_an_initial_literal_run() {
        // First byte 17 + 5 means five literals, then the end marker.
        let mut s = vec![17 + 5];
        s.extend(b"hello");
        s.extend(EOS);
        let (out, used) = decompress(&s, 5).unwrap();
        assert_eq!(out, b"hello");
        assert_eq!(used, s.len());
    }

    #[test]
    fn decodes_matches_and_trailing_literals() {
        // "abcd" literals, then an M2 match of length 3 at distance 4 ("abc") followed by
        // one trailing literal 'z', then the end marker.
        // M2: t >= 64, len = (t >> 5) - 1 + 2 = 3 -> t >> 5 = 2; distance - 1 = (t>>2 & 7) + b<<3 = 3.
        let t = (2 << 5) | (3 << 2) | 1; // low bits 1 = one trailing literal
        let mut s = vec![17 + 4];
        s.extend(b"abcd");
        s.extend([t as u8, 0x00, b'z']);
        s.extend(EOS);
        let (out, _) = decompress(&s, 8).unwrap();
        assert_eq!(out, b"abcdabcz");
    }

    #[test]
    fn decodes_long_overlapping_m3_match() {
        // "ab", then an M3 match (t >= 32) of 10 bytes at distance 2: "ababababab".
        let len = 10 - 2;
        let mut s = vec![17 + 2];
        s.extend(b"ab");
        s.push(32 | len as u8);
        s.extend((((2 - 1) << 2) as u16).to_le_bytes());
        s.extend(EOS);
        let (out, _) = decompress(&s, 12).unwrap();
        assert_eq!(out, b"abababababab");
    }

    #[test]
    fn rejects_bad_streams() {
        assert!(decompress(&[], 0).is_err());
        let mut s = vec![17 + 5];
        s.extend(b"hello");
        s.extend(EOS);
        assert!(decompress(&s, 4).is_err(), "more output than expected");
        assert!(decompress(&s, 6).is_err(), "less output than expected");
        assert!(decompress(&s[..s.len() - 1], 5).is_err(), "truncated");
        // A match reaching before the output start.
        let bad = [17 + 1, b'x', (2 << 5) | (7 << 2), 0xFF, 0x11, 0, 0];
        assert!(decompress(&bad, 10).is_err());
    }
}
