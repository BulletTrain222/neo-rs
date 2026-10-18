//! All test bytes below are synthetic; no original executable or asset data.
use neopets_data::{AssetSource, Disc};
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
