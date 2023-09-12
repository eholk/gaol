// Any copyright is dedicated to the Public Domain.
// http://creativecommons.org/publicdomain/zero/1.0/

use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use gaol::profile::{Operation, PathPattern, Profile};
use gaol::sandbox::{ChildSandbox, ChildSandboxMethods, Command, Sandbox, SandboxMethods};
use libc::c_char;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use std::ffi::{CString, OsStr};
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;

// A conservative overapproximation of `PATH_MAX` on all platforms.
const PATH_MAX: usize = 4096;

fn allowance_profile(path: &PathBuf) -> Profile {
    Profile::new(vec![Operation::FileReadAll(PathPattern::Literal(
        path.clone(),
    ))])
    .unwrap()
}

fn prohibition_profile() -> Profile {
    Profile::new(vec![Operation::FileReadAll(PathPattern::Subpath(
        PathBuf::from("/bogus"),
    ))])
    .unwrap()
}

fn allowance_test() -> eyre::Result<()> {
    debug!("allowance_test");
    let path = PathBuf::from(env::var("GAOL_TEMP_FILE")?);
    ChildSandbox::new(allowance_profile(&path))
        .activate()
        .unwrap();
    drop(File::open(&path)?);
    Ok(())
}

fn prohibition_test() -> eyre::Result<()> {
    let path = PathBuf::from(env::var("GAOL_TEMP_FILE")?);
    ChildSandbox::new(prohibition_profile()).activate().unwrap();
    drop(File::open(&path)?);
    Ok(())
}

pub fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match env::args().skip(1).next() {
        Some(ref arg) if arg == "allowance_test" => return allowance_test(),
        Some(ref arg) if arg == "prohibition_test" => return prohibition_test(),
        _ => {}
    }

    // Need to use `realpath` here for Mac OS X, because the temporary directory is usually a
    // symlink.
    let mut temp_path = env::temp_dir();
    unsafe {
        let c_temp_path = CString::new(temp_path.as_os_str().to_str().unwrap().as_bytes())?;
        let mut new_temp_path = [0u8; PATH_MAX];
        let _ = realpath(
            c_temp_path.as_ptr(),
            new_temp_path.as_mut_ptr() as *mut c_char,
        );
        let pos = new_temp_path.iter().position(|&x| x == 0).unwrap();
        temp_path = PathBuf::from(OsStr::from_bytes(&new_temp_path[..pos]));
    }

    let mut rng = rand::thread_rng();
    let suffix: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(6)
        .collect();

    temp_path.push(format!("gaoltest.{}", suffix));
    File::create(&temp_path)?.write_all(b"super secret\n")?;

    let allowance_status = Sandbox::new(allowance_profile(&temp_path))
        .start(
            &mut Command::me()?
                .arg("allowance_test")
                .env("GAOL_TEMP_FILE", temp_path.clone())
                .env("RUST_BACKTRACE", "1"),
        )?
        .wait()?;
    debug!("child process exited with {allowance_status:?}");
    assert!(allowance_status.success());

    let prohibition_status = Sandbox::new(prohibition_profile())
        .start(
            Command::me()?
                .arg("prohibition_test")
                .env("GAOL_TEMP_FILE", temp_path.clone())
                .env("RUST_BACKTRACE", "1"),
        )?
        .wait()?;
    assert!(!prohibition_status.success());
    Ok(())
}

extern "C" {
    fn realpath(file_name: *const c_char, resolved_name: *mut c_char) -> *mut c_char;
}
