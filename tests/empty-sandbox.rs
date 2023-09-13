//! Launch a sandbox with no permissions to see if the basic spawning process
//! works. We also try to perform an operation to make sure the sandbox is
//! actually restricted.

use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use gaol::profile::Profile;
use gaol::sandbox::{ChildSandbox, Command, Sandbox};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn get_profile() -> eyre::Result<Profile> {
    Ok(Profile::new(vec![])?)
}

fn sandbox_test() -> eyre::Result<()> {
    let path = PathBuf::from(env::var("GAOL_TEMP_FILE")?);
    ChildSandbox::new(get_profile()?).activate()?;
    assert!(File::open(path).is_err());
    Ok(())
}

pub fn main() -> eyre::Result<()> {
    // FIXME: this doesn't quite work on Windows yet
    #[cfg(windows)]
    {
        println!("This test is not yet supported on Windows.");
        return Ok(());
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    color_eyre::install()?;

    match env::args().skip(1).next() {
        Some(ref arg) if arg == "child_process" => return sandbox_test(),
        _ => {}
    }

    let mut temp_path = env::temp_dir();

    let mut rng = rand::thread_rng();
    let suffix: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(6)
        .collect();

    temp_path.push(format!("gaoltest.{}", suffix));
    File::create(&temp_path)?.write_all(b"super secret\n")?;

    let child_status = Sandbox::new(get_profile()?)
        .start(
            &mut Command::me()?
                .arg("child_process")
                .env("GAOL_TEMP_FILE", temp_path.clone())
                .env("RUST_BACKTRACE", "1"),
        )?
        .wait()?;
    debug!("child process exited with {child_status:?}");
    assert!(child_status.success());
    Ok(())
}
