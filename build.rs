//! Vendors `lib-char` (a pure package — the port) into `OUT_DIR` (ADR-0160/0161).
fn main() {
    handl_pkg_vendor::vendor("../lib-char").expect("vendor lib-char");
}
