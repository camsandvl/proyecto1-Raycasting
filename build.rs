//! Script de build de Cargo (corre antes de compilar `src/`).
//!
//! SDL2 es una librería en C, no algo que Cargo pueda descargar y compilar solo.
//! Este script:
//! 1. le dice al linker dónde están los `.lib` de SDL2/SDL2_mixer/SDL2_ttf
//!    (`vendor/sdl2/lib/`) para que `cargo build` enlace correctamente.
//! 2. copia los `.dll` correspondientes (`vendor/sdl2/bin/`) junto al `.exe`
//!    compilado (`target/debug/` o `target/release/`), porque Windows necesita
//!    encontrarlos ahí (o en el PATH) para poder *ejecutar* el juego.
//!
//! Así el repo no queda con `.dll` sueltos en la raíz — viven ordenados en
//! `vendor/`, y este script los pone donde hacen falta automáticamente.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("vendor/sdl2/lib");
    let bin_dir = manifest_dir.join("vendor/sdl2/bin");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // OUT_DIR es algo como target/debug/build/<pkg>-<hash>/out; el ejecutable
    // final vive 3 niveles arriba de ahí (target/debug/).
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("no se pudo resolver el directorio target desde OUT_DIR")
        .to_path_buf();

    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "dll") {
                let dest = target_dir.join(path.file_name().unwrap());
                if let Err(err) = fs::copy(&path, &dest) {
                    println!(
                        "cargo:warning=no se pudo copiar {} a {}: {}",
                        path.display(),
                        dest.display(),
                        err
                    );
                }
            }
        }
    }
}
