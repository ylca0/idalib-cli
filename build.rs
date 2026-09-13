fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=IDASDKDIR");
    println!("cargo:rerun-if-env-changed=IDADIR");

    // When compiling against the dev stub (feature `stub-idalib`), skip all
    // IDA-specific build script logic entirely.
    if std::env::var("CARGO_FEATURE_STUB_IDALIB").is_ok() {
        return Ok(());
    }

    // `idalib-sys` (a dependency) REQUIRES the full IDA SDK headers to run
    // bindgen. We can't compile at all without it, so give a friendly error
    // before the dependency's cryptic panic surfaces.
    match std::env::var("IDASDKDIR") {
        Ok(dir) => {
            let pro_h = std::path::PathBuf::from(&dir).join("include").join("pro.h");
            if !pro_h.exists() {
                eprintln!(
                    "warning: IDASDKDIR={dir} does not contain include/pro.h; \
                     idalib-sys bindgen will fail. Point IDASDKDIR at the IDA SDK."
                );
            }
        }
        Err(_) => {
            eprintln!(
                "error: IDASDKDIR is not set.\n\
                 The idalib-rs bindings generate their FFI via bindgen at compile time,\n\
                 which parses the IDA SDK headers (pro.h, ida.hpp, ...). These ship\n\
                 only with the IDA SDK - not with the IDA application - and cannot be\n\
                 redistributed. Download the SDK for your license from the Hex-Rays\n\
                 user portal (my.hex-rays.com -> Downloads -> IDA SDK 9.1), unzip it,\n\
                 then set:\n\
                 \n\
                   export IDASDKDIR=$HOME/idasdk91\n\
                   export IDADIR=\"/Applications/IDA Professional 9.1.app/Contents/MacOS\"\n"
            );
        }
    }

    // Prefer linking against the installed IDA libraries (IDADIR) with RPATH.
    // If no IDA installation is present (e.g. CI), fall back to the SDK stub
    // libraries - the official idalib-rs pattern. Stubs are link-time shells;
    // at runtime the binary still needs a licensed IDA installation.
    let (install_path, ida_path, idalib_path) = idalib_build::idalib_install_paths_with(false);
    if cfg!(target_os = "windows") {
        idalib_build::configure_idasdk_linkage();
    } else if ida_path.exists() && idalib_path.exists() {
        idalib_build::configure_linkage()?;
    } else {
        println!("cargo::warning=IDA installation not found; linking against SDK stubs.");
        idalib_build::configure_idasdk_linkage();
    }
    let _ = install_path;
    Ok(())
}
