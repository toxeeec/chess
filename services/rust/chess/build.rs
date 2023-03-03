use magics::{bishop_magics, rook_magics};
use quote::quote;
use std::process::Command;
use std::{env, fs, path::Path};

fn main() {
    let (rook_magics, rook_attacks) = rook_magics();
    let (bishop_magics, bishop_attacks) = bishop_magics();
    let rook_attacks_len = rook_attacks.len();
    let rook_tokens = quote! {
        pub static ROOK_MAGICS: [Magic; 64] = [#(#rook_magics),*];
        pub static ROOK_ATTACKS: [Bitboard; #rook_attacks_len] = [#(#rook_attacks),*];
    };
    let bishop_attacks_len = bishop_attacks.len();
    let bishop_tokens = quote! {
        pub static BISHOP_MAGICS: [Magic; 64] = [#(#bishop_magics),*];
        pub static BISHOP_ATTACKS: [Bitboard; #bishop_attacks_len] = [#(#bishop_attacks),*];
    };
    let magics_tokens = quote! {
        #rook_tokens
        #bishop_tokens
    };

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let magics_path = Path::new(&out_dir).join("magics.rs");
    fs::write(&magics_path, magics_tokens.to_string()).unwrap();
    Command::new("rustfmt").arg(&magics_path).output().unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
