//! Windows sandboxing support
//!
//!

use std::{
    ffi::{OsStr, OsString},
    io,
};

use thiserror::Error;

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
        // FIXME: this is a total lie!
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SandboxError {}

pub struct Sandbox {
    profile: Profile,
}

impl Sandbox {
    pub fn new(profile: Profile) -> Sandbox {
        Sandbox { profile }
    }

    pub fn start(&self, command: &mut Command) -> Result<process::Process, io::Error> {
        // For now just use the standard library to launch a new process.

        let proc =
            std::process::Command::new(OsString::from(command.module_path.to_str().unwrap()))
                .args(
                    command
                        .args
                        .iter()
                        .map(|s| OsString::from(s.to_str().unwrap())),
                )
                .envs(command.env.iter().map(|(k, v)| {
                    (
                        OsString::from(k.to_str().unwrap()),
                        OsString::from(v.to_str().unwrap()),
                    )
                }))
                .spawn()?;
        Ok(process::Process(proc))
    }
}
