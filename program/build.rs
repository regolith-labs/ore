use std::env;
use std::path::PathBuf;

use solana_include_idl::compress_idl;

/// Build script to compress the IDL file to a zip file when building the program.
///
/// The compressed IDL is then included in a separate ELF section on the program binary
/// when the program is built.
fn main() {
    // Get the IDL path.
    let idl_path = PathBuf::from("../api").join("idl.json");
    // Compress the IDL file to a zip file.
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("codama.idl.zip");

    compress_idl(&idl_path, &dest_path);
}
