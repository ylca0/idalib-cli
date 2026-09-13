use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=IDASDKDIR");
    println!("cargo:rerun-if-env-changed=IDADIR");

    // When compiling against the dev stub (feature `stub-idalib`), skip all
    // IDA-specific build script logic entirely.
    if std::env::var("CARGO_FEATURE_STUB_IDALIB").is_ok() {
        return Ok(());
    }

    // ---- Hard prerequisite 1: IDA SDK (build-time, for bindgen) ----
    let sdk_dir = match std::env::var("IDASDKDIR") {
        Ok(d) => {
            let p = PathBuf::from(d);
            // Relative paths break inside the build script: cargo compiles
            // build scripts with a different CWD (the package dir, not the
            // user's shell CWD), and autocxx/clang then can't find headers.
            // Relative paths DO NOT WORK: cargo runs each build script with
            // its own package dir as CWD, so `idalib-sys` (a dependency)
            // resolves ./xxx against ~/.cargo/registry/... and fails with a
            // cryptic "auto.hpp not found". Catch it here with actionable
            // guidance instead.
            if !p.is_absolute() {
                let abs = std::env::current_dir().expect("current dir").join(&p);
                let p_display = p.display();
                let abs_display = abs.display();
                eprintln!(
                    "\n\
                     ============================================================\n\
                     \x20 IDASDKDIR must be an ABSOLUTE path.\n\
                     ============================================================\n\
                     \n\
                     Got: {p_display}\n\
                     Relative paths are resolved by dependency build scripts\n\
                     against their own package directory, not your shell.\n\
                     \n\
                     Fix (copy-paste):\n\
                     \n\
                       export IDASDKDIR={abs_display}\n"
                );
                std::process::exit(1);
            }
            p
        }
        Err(_) => {
            eprintln!(
                "\n\
                 ============================================================\n\
                 \x20 IDASDKDIR is not set — cannot build.\n\
                 ============================================================\n\
                 \n\
                 idalib-rs generates its FFI bindings at compile time by\n\
                 parsing the IDA SDK headers (pro.h, auto.hpp, ...). The SDK\n\
                 ships only with your Hex-Rays license and is never\n\
                 redistributed.\n\
                 \n\
                 1. Download the SDK: my.hex-rays.com -> Downloads ->\n\
                    IDA SDK 9.1 (must match your IDA version)\n\
                 2. Unzip it, then set:\n\
                 \n\
                      export IDASDKDIR=$HOME/idasdk91      # the unzipped dir\n\
                      export IDADIR=\"/Applications/IDA Professional 9.1.app/Contents/MacOS\"\n\
                 \n\
                 3. Re-run: cargo install --path .\n"
            );
            std::process::exit(1);
        }
    };
    let pro_h = sdk_dir.join("include").join("pro.h");
    if !pro_h.exists() {
        let sdk_display = sdk_dir.display();
        let pro_display = pro_h.display();
        eprintln!(
            "\n\
             ============================================================\n\
             \x20 IDASDKDIR={sdk_display} is not a usable IDA SDK.\n\
             ============================================================\n\
             \n\
             Expected: {pro_display}\n\
             Point IDASDKDIR at the unzipped SDK directory that contains\n\
             include/ and lib/.\n"
        );
        std::process::exit(1);
    }

    // ---- Hard prerequisite 2: IDA installation (runtime linkage) ----
    let lib_name = if cfg!(target_os = "macos") {
        "libidalib.dylib"
    } else if cfg!(target_os = "linux") {
        "libidalib.so"
    } else {
        "idalib.lib"
    };
    let (install_path, ida_path, idalib_path) = idalib_build::idalib_install_paths_with(false);
    if !ida_path.exists() || !idalib_path.exists() {
        let lib_display = idalib_path.display();
        eprintln!(
            "\n\
             ============================================================\n\
             \x20 IDA installation not found — cannot link.\n\
             ============================================================\n\
             \n\
             Expected: {lib_display}\n\
             \n\
             Set IDADIR to the directory containing {lib_name}, e.g.:\n\
             \n\
               export IDADIR=\"/Applications/IDA Professional 9.1.app/Contents/MacOS\"\n\
             \n\
             (Probed locations: IDADIR env, then common install paths.)\n"
        );
        std::process::exit(1);
    }
    let _ = &install_path;

    // Link against the installed IDA libraries and bake the RPATH so the
    // binary finds libida/libidalib at runtime.
    if cfg!(target_os = "windows") {
        idalib_build::configure_idasdk_linkage();
    } else {
        idalib_build::configure_linkage()?;
    }
    Ok(())
}
