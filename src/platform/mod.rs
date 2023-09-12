#[cfg(target_os = "freebsd")]
pub use freebsd::{ChildSandbox, Operation, Sandbox};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use linux::{ChildSandbox, Operation, Sandbox};
#[cfg(target_os = "macos")]
pub use macos::{ChildSandbox, Operation, Sandbox};
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd"
))]
pub use unix::process::{self, Process};
#[cfg(target_os = "windows")]
pub use windows::{ChildSandbox, Operation, Sandbox};

#[cfg(target_os = "freebsd")]
pub mod freebsd;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd"
))]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::process::{self, Process};
