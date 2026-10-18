//! All test bytes below are synthetic; no original executable or asset data.
use neopets_data::{AssetSource, Disc, fingerprint};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture {
    root: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn record(name: &[u8], sector: u32, size: u32, directory: bool) -> Vec<u8> {
    let len = (33 + name.len() + 1) & !1;
    let mut r = vec![0; len];
    r[0] = len as u8;
    r[2..6].copy_from_slice(&sector.to_le_bytes());
    r[6..10].copy_from_slice(&sector.to_be_bytes());
    r[10..14].copy_from_slice(&size.to_le_bytes());
    r[14..18].copy_from_slice(&size.to_be_bytes());
    r[25] = if directory { 2 } else { 0 };
    r[28] = 1;
    r[31] = 1;
    r[32] = name.len() as u8;
    r[33..33 + name.len()].copy_from_slice(name);
    r
}
fn fixture() -> Fixture {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-fixtures");
    let root = base.join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut image = vec![0; 24 * 2048];
    let p = 16 * 2048;
    image[p] = 1;
    image[p + 1..p + 6].copy_from_slice(b"CD001");
    image[p + 6] = 1;
    image[p + 128..p + 130].copy_from_slice(&2048u16.to_le_bytes());
    image[p + 130..p + 132].copy_from_slice(&2048u16.to_be_bytes());
    let rootrec = record(&[0], 20, 2048, true);
    image[p + 156..p + 156 + rootrec.len()].copy_from_slice(&rootrec);
    let cnf = b"BOOT2 = cdrom0:\\SCUS_973.67;1\r\nVER = 1.00\r\n";
    image[21 * 2048..21 * 2048 + cnf.len()].copy_from_slice(cnf);
    image[22 * 2048..22 * 2048 + 5].copy_from_slice(b"hello");
    image[23 * 2048..23 * 2048 + 7].copy_from_slice(b"\x7fELF\x01\x01\x01");
    image[23 * 2048 + 18] = 8;
    let mut cursor = 20 * 2048;
    for r in [
        record(b"SYSTEM.CNF;1", 21, cnf.len() as u32, false),
        record(b"GAMEFILE.DAT;1", 22, 5, false),
        record(b"SCUS_973.67;1", 23, 64, false),
    ] {
        image[cursor..cursor + r.len()].copy_from_slice(&r);
        cursor += r.len();
    }
    std::fs::write(root.join("synthetic.iso"), image).unwrap();
    Fixture { root }
}
fn open(f: &Fixture) -> Disc {
    Disc::open(&f.root.join("synthetic.iso")).unwrap()
}

#[test]
fn reads_and_checks_synthetic_disc_without_extraction() {
    let f = fixture();
    let mut d = open(&f);
    assert!(d.verify_neopets().unwrap().contains("SCUS-97367"));
    assert_eq!(d.files().len(), 3);
    assert_eq!(d.read("/gamefile.dat;1").unwrap(), b"hello");
    assert_eq!(d.read_range("GAMEFILE.DAT", 1, 3).unwrap(), b"ell");
    assert!(d.read_range("GAMEFILE.DAT", 4, 2).is_err());
    assert!(d.read_range("GAMEFILE.DAT", u64::MAX, 2).is_err());
}
#[test]
fn rejects_corrupt_and_truncated_images() {
    let f = fixture();
    let p = f.root.join("synthetic.iso");
    let original = std::fs::read(&p).unwrap();
    let mut corrupt = original.clone();
    corrupt[20 * 2048] = 2;
    std::fs::write(&p, corrupt).unwrap();
    assert!(Disc::open(&p).is_err());
    std::fs::write(&p, &original[..2048]).unwrap();
    assert!(Disc::open(&p).is_err());
}
#[test]
fn refuses_other_games_and_missing_assets() {
    let f = fixture();
    let p = f.root.join("synthetic.iso");
    let mut bytes = std::fs::read(&p).unwrap();
    let index = bytes.windows(11).position(|w| w == b"SCUS_973.67").unwrap();
    // Alter the directory name: BOOT2 still claims this game, but its boot file is absent.
    bytes[index] = b'X';
    std::fs::write(&p, bytes).unwrap();
    assert!(open(&f).verify_neopets().is_err());
}
#[test]
fn mods_override_in_order_and_reject_traversal() {
    let f = fixture();
    let a = f.root.join("a");
    let b = f.root.join("b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    std::fs::write(a.join("GAMEFILE.DAT"), b"first").unwrap();
    std::fs::write(b.join("GAMEFILE.DAT"), b"second").unwrap();
    let mut source = AssetSource::new(open(&f), &[a, b]).unwrap();
    assert_eq!(source.read("gamefile.dat").unwrap(), b"second");
    assert!(source.read("SYSTEM.CNF").unwrap().starts_with(b"BOOT2"));
    assert!(source.read("../outside").is_err());
    assert!(source.read("C:/outside").is_err());
}
fn stream(d: &mut Disc, offset: u64, size: u64, chunk: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    d.stream_range("GAMEFILE.DAT", offset, size, chunk, |b| {
        assert!(!b.is_empty() && b.len() <= chunk);
        out.extend_from_slice(b);
        Ok(())
    })?;
    Ok(out)
}
/// Byte offset of a directory record in the synthetic image, found by its file name.
fn record_at(image: &[u8], name: &[u8]) -> usize {
    image.windows(name.len()).position(|w| w == name).unwrap() - 33
}
fn patched(f: &Fixture, patch: impl FnOnce(&mut Vec<u8>)) -> Result<Disc, String> {
    let p = f.root.join("synthetic.iso");
    let mut bytes = std::fs::read(&p).unwrap();
    patch(&mut bytes);
    std::fs::write(&p, bytes).unwrap();
    Disc::open(&p)
}

#[test]
fn streams_ranges_in_bounded_chunks() {
    let f = fixture();
    let mut d = open(&f);
    for chunk in [1, 2, 3, 1 << 20] {
        assert_eq!(stream(&mut d, 0, 5, chunk).unwrap(), b"hello");
        assert_eq!(stream(&mut d, 1, 3, chunk).unwrap(), b"ell");
    }
    assert_eq!(stream(&mut d, 5, 0, 4).unwrap(), b"");
    assert!(stream(&mut d, 0, 5, 0).is_err());
    assert!(stream(&mut d, 0, 5, (64 << 20) + 1).is_err());
    assert!(stream(&mut d, 3, 3, 4).is_err());
    assert!(stream(&mut d, u64::MAX, 2, 4).is_err());
    let mut calls = 0;
    let stopped = d.stream_range("GAMEFILE.DAT", 0, 5, 2, |_| {
        calls += 1;
        Err("stop".into())
    });
    assert_eq!((stopped, calls), (Err("stop".into()), 1));
}
#[test]
fn fingerprints_files_with_streaming_sha256() {
    let f = fixture();
    let mut d = open(&f);
    // Public SHA-256 test value for the ASCII bytes "hello".
    let hello = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
    let digest = fingerprint::sha256_file(&mut d, "gamefile.dat").unwrap();
    assert_eq!(fingerprint::to_hex(&digest), hello);
    let expected = fingerprint::parse_sha256(&hello.to_ascii_uppercase()).unwrap();
    assert!(fingerprint::verify_sha256(&mut d, "GAMEFILE.DAT", &expected).is_ok());
    assert!(fingerprint::verify_sha256(&mut d, "SYSTEM.CNF", &expected).is_err());
    assert!(fingerprint::sha256_file(&mut d, "MISSING.BIN").is_err());
    assert!(fingerprint::parse_sha256(&hello[1..]).is_err());
    assert!(fingerprint::parse_sha256(&format!("+{}", &hello[1..])).is_err());
}
#[test]
fn rejects_cyclic_overlong_and_out_of_bounds_records() {
    let make_cycle = |b: &mut Vec<u8>| {
        let r = record_at(b, b"GAMEFILE.DAT;1");
        b[r + 2..r + 6].copy_from_slice(&20u32.to_le_bytes());
        b[r + 10..r + 14].copy_from_slice(&2048u32.to_le_bytes());
        b[r + 25] = 2;
    };
    let past_end = |b: &mut Vec<u8>| {
        let r = record_at(b, b"GAMEFILE.DAT;1");
        b[r + 10..r + 14].copy_from_slice(&u32::MAX.to_le_bytes());
    };
    let long_name = |b: &mut Vec<u8>| {
        let r = record_at(b, b"GAMEFILE.DAT;1");
        b[r + 32] = 200;
    };
    let multi_extent = |b: &mut Vec<u8>| {
        let r = record_at(b, b"GAMEFILE.DAT;1");
        b[r + 25] = 0x80;
    };
    let patches: [fn(&mut Vec<u8>); 4] = [make_cycle, past_end, long_name, multi_extent];
    for patch in patches {
        let f = fixture();
        assert!(patched(&f, patch).is_err());
    }
}
#[test]
fn reports_reads_from_an_image_truncated_after_indexing() {
    let f = fixture();
    let p = f.root.join("synthetic.iso");
    let mut d = open(&f);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&p)
        .unwrap()
        .set_len(21 * 2048)
        .unwrap();
    let err = d.read("GAMEFILE.DAT").unwrap_err();
    assert!(err.contains("failed"), "{err}");
    assert!(stream(&mut d, 0, 5, 2).is_err());
}
