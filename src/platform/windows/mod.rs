//! Windows sandboxing support
//!
//!

use std::{
    ffi::{OsStr, OsString},
    io, ptr,
};

use thiserror::Error;
use tracing::debug;
use widestring::U16String;
use windows::{
    core::{PCWSTR, PWSTR},
    w,
    Win32::{
        Foundation::{GetLastError, HANDLE, WIN32_ERROR},
        Security::{CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, TOKEN_ALL_ACCESS, TOKEN_READ},
        System::Threading::{
            CreateProcessAsUserW, CreateProcessWithLogonW, CreateProcessWithTokenW,
            GetCurrentProcess, GetStartupInfoW, OpenProcessToken, CREATE_NEW_CONSOLE,
            CREATE_NEW_PROCESS_GROUP, CREATE_PROCESS_LOGON_FLAGS, CREATE_UNICODE_ENVIRONMENT,
            NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

use crate::{
    profile::{self, OperationSupport, OperationSupportLevel, Profile},
    sandbox::Command,
};

pub mod process;

#[derive(Clone, Debug)]
pub struct Operation;

impl OperationSupport for profile::Operation {
    fn support(&self) -> OperationSupportLevel {
        match self {
            // Say everything is always allowed because we have not implemented any
            // Windows sandboxing.
            _ => OperationSupportLevel::AlwaysAllowed,
        }
    }
}
pub struct ChildSandbox {
    profile: Profile,
}

impl ChildSandbox {
    pub fn new(profile: Profile) -> ChildSandbox {
        ChildSandbox { profile }
    }

    pub fn activate(&self) -> Result<(), SandboxError> {
        // FIXME: Everything is not OK
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("failed to get current process token")]
    ProcessToken,
    #[error("failed to create restricted token: {0:?}")]
    RestrictedToken(WIN32_ERROR),
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("failed to create child process: {0:?}")]
    CreateChildProcess(WIN32_ERROR),

    /// Indicates that the module name was not a valid utf-8 string
    #[error("invalid module name")]
    InvalidCommandLine,
}

pub struct Sandbox {
    profile: Profile,
}

impl Sandbox {
    pub fn new(profile: Profile) -> Sandbox {
        Sandbox { profile }
    }

    pub fn start(&self, command: &mut Command) -> Result<process::Process, SandboxError> {
        let current_token = get_current_token()?;
        let child_token = create_restricted_token(current_token)?;

        let proc = create_process_with_token(command, current_token)?;

        Ok(process::Process(proc))
    }
}

fn get_current_token() -> Result<HANDLE, SandboxError> {
    let mut token_handle: HANDLE = HANDLE::default();
    unsafe {
        if !OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut token_handle).as_bool() {
            return Err(SandboxError::ProcessToken);
        }
    }
    Ok(token_handle)
}

fn create_restricted_token(token: HANDLE) -> Result<HANDLE, SandboxError> {
    let mut restricted_token = HANDLE::default();
    unsafe {
        if !CreateRestrictedToken(
            token,
            DISABLE_MAX_PRIVILEGE,
            None,
            None,
            None,
            &mut restricted_token,
        )
        .as_bool()
        {
            return Err(SandboxError::RestrictedToken(GetLastError()));
        }
    }
    Ok(restricted_token)
}

fn create_process_with_token(
    command: &Command,
    token: HANDLE,
) -> Result<PROCESS_INFORMATION, SandboxError> {
    let mut process_info = PROCESS_INFORMATION::default();

    // convert the module_path to a wide string for use by win32
    let app_name = U16String::from_str(&format!(
        "{}\0",
        command
            .module_path
            .to_str()
            .map_err(|_| SandboxError::InvalidCommandLine)?,
    ));

    // Generate the command line
    let mut command_line = app_name[..app_name.len() - 1].to_owned();
    for arg in command.args.iter() {
        // FIXME: we can look forward to all kinds of horror getting the
        // escaping here right.
        command_line.push_str(&format!(
            " {}",
            arg.to_str().map_err(|_| SandboxError::InvalidCommandLine)?
        ));
    }

    // create the environment block
    let mut env = U16String::new();
    for (k, v) in command.env.iter() {
        env.push_str(&format!(
            "{}={}\0",
            k.to_str().map_err(|_| SandboxError::InvalidCommandLine)?,
            v.to_str().map_err(|_| SandboxError::InvalidCommandLine)?,
        ));
    }
    env.push_str("RUST_LOG=debug\0RUST_BACKTRACE=1\0");
    env.push_str("\0");

    let mut startup_info = STARTUPINFOW::default();

    debug!("launching child process");
    debug!("command_line: {:?}", command_line);
    debug!("env: {:?}", env);

    unsafe {
        GetStartupInfoW(&mut startup_info);

        if !CreateProcessAsUserW(
            HANDLE(0),           //token,
            PCWSTR(ptr::null()), //PCWSTR(app_name.as_ptr()),
            PWSTR(command_line.as_mut_ptr()),
            None,
            None,
            true, // inherit handles so we can see stdout/stderr
            CREATE_UNICODE_ENVIRONMENT,
            Some(env.as_ptr() as _),
            PCWSTR(ptr::null()),
            &startup_info,
            &mut process_info,
        )
        .as_bool()
        {
            return Err(SandboxError::CreateChildProcess(GetLastError()));
        }
    }
    Ok(process_info)
}
