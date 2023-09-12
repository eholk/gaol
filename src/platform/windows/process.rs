use std::io;

pub struct Process;

impl Process {
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        unimplemented!("Process::wait")
    }
}

#[derive(Debug)]
pub enum ExitStatus {}

impl ExitStatus {
    pub fn success(&self) -> bool {
        unimplemented!("ExitStatus::success")
    }
}