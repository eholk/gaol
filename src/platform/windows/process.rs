use std::{
    io,
    process::{self, Child},
};

use windows::Win32::System::Threading::PROCESS_INFORMATION;

pub struct Process(pub(super) PROCESS_INFORMATION);

impl Process {
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        unimplemented!()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // FIXME: close all the handles in PROCESS_INFORMATION
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum ExitStatus {
    Status(process::ExitStatus),
}

impl ExitStatus {
    pub fn success(&self) -> bool {
        match self {
            ExitStatus::Status(status) => status.success(),
        }
    }
}
