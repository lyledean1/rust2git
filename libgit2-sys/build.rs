extern crate curl;
extern crate flate2;
extern crate pkg_config;
extern crate semver;
extern crate tar;

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use semver::Version;

const FRAMEWORK_LIBRARY: &str = "libgit2_framework";
const LIBRARY: &str = "libgit2";
const REPOSITORY: &str = "https://github.com/libgit2/libgit2";
const FRAMEWORK_TARGET: &str = "libgit2:libgit2_framework";
const TARGET: &str = "libgit2:libtensorflow";
// `VERSION` and `TAG` are separate because the tag is not always `'v' + VERSION`.
const TAG: &str = "v1.5.0";
const MIN_BAZEL: &str = "3.7.2";

macro_rules! get(($name:expr) => (ok!(env::var($name))));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("libgit2-sys/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("libgit2-sys/build.rs:{}: ", $fmt),
    line!(), $($arg)*));
}
macro_rules! log_var(($var:ident) => (log!(concat!(stringify!($var), " = {:?}"), $var)));

fn main() {
    // If we are doing runtime linking, just return.
    #[cfg(feature = "runtime_linking")]
    return;

    // DO NOT RELY ON THIS
    if cfg!(feature = "private-docs-rs") {
        log!("Returning early because private-docs-rs feature was enabled");
        return;
    }

    if check_windows_lib() {
        log!("Returning early because {} was already found", LIBRARY);
        return;
    }

    // Note that pkg_config will print cargo:rustc-link-lib and cargo:rustc-link-search as
    // appropriate if the library is found.
    if pkg_config::probe_library(LIBRARY).is_ok() {
        log!("Returning early because {} was already found", LIBRARY);
        return;
    }

    build_from_src();
}

fn target_os() -> String {
    get!("CARGO_CFG_TARGET_OS")
}

fn dll_suffix() -> &'static str {
    match &target_os() as &str {
        "windows" => ".dll",
        "macos" => ".dylib",
        _ => ".so",
    }
}

fn check_windows_lib() -> bool {
    if target_os() != "windows" {
        return false;
    }
    let windows_lib: &str = &format!("{}.lib", LIBRARY);
    if let Ok(path) = env::var("PATH") {
        for p in path.split(';') {
            let path = Path::new(p).join(windows_lib);
            if path.exists() {
                println!("cargo:rustc-link-lib=dylib={}", LIBRARY);
                println!("cargo:rustc-link-search=native={}", p);
                return true;
            }
        }
    }
    false
}

fn symlink<P: AsRef<Path>, P2: AsRef<Path>>(target: P, link: P2) {
    if link.as_ref().exists() {
        // Avoid errors if it already exists.
        fs::remove_file(link.as_ref()).unwrap();
    }
    log!(
        "Creating symlink {:?} pointing to {:?}",
        link.as_ref(),
        target.as_ref()
    );
    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_file(target, link).unwrap();
    #[cfg(not(target_os = "windows"))]
    std::os::unix::fs::symlink(target, link).unwrap();
}

fn build_from_src() {
    let dll_suffix = dll_suffix();
    let framework_target = FRAMEWORK_TARGET.to_string() + dll_suffix;
    let target = TARGET.to_string() + dll_suffix;

    let output = PathBuf::from(&get!("OUT_DIR"));
    log_var!(output);
    let source = PathBuf::from(&get!("CARGO_MANIFEST_DIR")).join(format!("target/source-{}", TAG));
    log_var!(source);
    let lib_dir = output.join(format!("lib-{}", TAG));
    log_var!(lib_dir);
    if lib_dir.exists() {
        log!("Directory {:?} already exists", lib_dir);
    } else {
        log!("Creating directory {:?}", lib_dir);
        fs::create_dir(lib_dir.clone()).unwrap();
    }
    let framework_unversioned_library_path = lib_dir.join(format!("lib{}.so", FRAMEWORK_LIBRARY));
    let framework_library_path = lib_dir.join(format!("lib{}.so.2", FRAMEWORK_LIBRARY));
    log_var!(framework_library_path);
    let unversioned_library_path = lib_dir.join(format!("lib{}.so", LIBRARY));
    let library_path = lib_dir.join(format!("lib{}.so.2", LIBRARY));
    log_var!(library_path);
    if library_path.exists() && framework_library_path.exists() {
        log!(
            "{:?} and {:?} already exist, not building",
            library_path,
            framework_library_path
        );
    } else {
        if let Err(e) = check_bazel() {
            println!(
                "cargo:error=Bazel must be installed at version {} or greater. (Error: {})",
                MIN_BAZEL, e
            );
            process::exit(1);
        }
        let framework_target_path = &format!("{}.2", framework_target.replace(':', "/"));
        log_var!(framework_target_path);
        let target_path = &format!("{}.so", TARGET.replace(':', "/"));
        log_var!(target_path);
        if !Path::new(&source.join(".git")).exists() {
            run("git", |command| {
                command
                    .arg("clone")
                    .arg(format!("--branch={}", TAG))
                    .arg("--recursive")
                    .arg(REPOSITORY)
                    .arg(&source)
            });
        }
        // Only configure if not previously configured.  Configuring runs a
        // `bazel clean`, which we don't want, because we want to be able to
        // continue from a cancelled build.
        let configure_hint_file_pb = source.join(".rust-configured");
        let configure_hint_file = Path::new(&configure_hint_file_pb);
        if !configure_hint_file.exists() {
            fs::create_dir_all("/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build");
            run("cmake", |command| {
                command
                    // replace
                    .current_dir("/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build")
                    .arg("..")
            });
            run("cmake", |command| {
                command
                    // replace
                    .current_dir("/Users/lyledean/rust/rust2git/libgit2-sys/target/source-v1.5.0/build")
                    .arg("--build")
                    .arg(".")
            });
            File::create(configure_hint_file).unwrap();
        }
  
    symlink(
        framework_library_path.file_name().unwrap(),
        framework_unversioned_library_path,
    );
    symlink(library_path.file_name().unwrap(), unversioned_library_path);
    println!("cargo:rustc-link-lib=dylib={}", FRAMEWORK_LIBRARY);
    println!("cargo:rustc-link-lib=dylib={}", LIBRARY);
    println!("cargo:rustc-link-search={}", lib_dir.display());
    }
}

fn run<F>(name: &str, mut configure: F)
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    log!("Executing {:?}", configured);
    if !ok!(configured.status()).success() {
        panic!("failed to execute {:?}", configured);
    }
    log!("Command {:?} finished successfully", configured);
}

// Building TF 0.11.0rc1 with Bazel 0.3.0 gives this error when running `configure`:
//   expected ConfigurationTransition or NoneType for 'cfg' while calling label_list but got
// string instead:     data.
//       ERROR: com.google.devtools.build.lib.packages.BuildFileContainsErrorsException: error
// loading package '': Extension file 'tensorflow/tensorflow.bzl' has errors.
// And the simple solution is to require Bazel 0.3.1 or higher.
fn check_bazel() -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("bazel");
    command.arg("version");
    log!("Executing {:?}", command);
    let out = command.output()?;
    log!("Command {:?} finished successfully", command);
    let stdout = String::from_utf8(out.stdout)?;
    let mut found_version = false;
    for line in stdout.lines() {
        if line.starts_with("Build label:") {
            found_version = true;
            let mut version_str = line
                .split(':')
                .nth(1)
                .unwrap()
                .split(' ')
                .nth(1)
                .unwrap()
                .trim();
            if version_str.ends_with('-') {
                // hyphen is 1 byte long, so it's safe
                version_str = &version_str[..version_str.len() - 1];
            }
            let version = Version::parse(version_str)?;
            let want = Version::parse(MIN_BAZEL)?;
            if version < want {
                return Err(format!(
                    "Installed version {} is less than required version {}",
                    version_str, MIN_BAZEL
                )
                .into());
            }
        }
    }
    if !found_version {
        return Err("Did not find version number in `bazel version` output.".into());
    }
    Ok(())
}