//! Mahrem benchmark CLI'ı
//!
//! Yer tutucu: şimdilik yalnızca sürümü yazdırır.

fn main() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
