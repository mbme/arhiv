use std::env;

use rs_utils::ensure_dir_exists;

/// Pinned NDK version, needs to be installed on machine.
const ANDROID_NDK_VERSION: &'static str = "25.2.9519653";

/// x86-64 linux standard library path inside NDK directory.
const LINUX_X86_64_LIB_DIR: &'static str =
    "/toolchains/llvm/prebuilt/linux-x86_64/lib64/clang/14.0.7/lib/linux/";

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // https://github.com/rusqlite/rusqlite/issues/1380
    // https://github.com/bbqsrc/cargo-ndk/issues/94
    // The new NDK doesn't link to `libgcc` anymore, which breaks rusqlite it
    // depends on the symbols from `libclang_rt.builtins-x86_64-android` like `__extenddftf2`
    // The change works around this by manually linking to the
    // `libclang_rt.builtins-x86_64-android` library in this case.
    if target_arch == "x86_64" && target_os == "android" {
        let android_home = env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME not set");

        let clang_path = format!("{android_home}/{ANDROID_NDK_VERSION}/{LINUX_X86_64_LIB_DIR}");
        ensure_dir_exists(&clang_path).expect("clang dir must exist");

        println!("cargo:rustc-link-search={clang_path}");
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
    }
}
