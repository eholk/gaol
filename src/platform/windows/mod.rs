use std::io;

use crate::{
    profile::{self, AddressPattern, OperationSupport, OperationSupportLevel, Profile},
    sandbox::Command,
};

pub mod process;

#[derive(Clone, Debug)]
pub struct Operation;

impl OperationSupport for profile::Operation {
    fn support(&self) -> OperationSupportLevel {
        // Say everything is always allowed because we have not implemented any
        // Windows sandboxing.
        OperationSupportLevel::AlwaysAllowed
    }
}
pub struct ChildSandbox {
    profile: Profile,
}

impl ChildSandbox {
    pub fn new(profile: Profile) -> ChildSandbox {
        ChildSandbox { profile }
    }

    pub fn activate(&self) -> Result<(), ()> {
        unimplemented!("ChildSandbox::activate")
    }
}

pub struct Sandbox {
    profile: Profile,
}

impl Sandbox {
    pub fn new(profile: Profile) -> Sandbox {
        Sandbox { profile }
    }

    pub fn start(&self, command: &mut Command) -> Result<process::Process, io::Error> {
        unimplemented!("Sandbox::start")
    }
}
