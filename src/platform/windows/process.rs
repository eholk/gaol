use std::{
    io,
    process::{self, Child},
};

pub struct Process(pub(super) Child);

impl Process {
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0.wait().map(|status| ExitStatus::Status(status))
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
