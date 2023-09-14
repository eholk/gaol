use std::io;

use windows::Win32::{
    Foundation::{CloseHandle, WAIT_FAILED},
    System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE, PROCESS_INFORMATION},
};

pub struct Process(pub(super) PROCESS_INFORMATION);

impl Process {
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        // SAFETY: FFI calls
        unsafe {
            let mut exit_code = 0;
            let result = WaitForSingleObject(self.0.hProcess, INFINITE);
            if result == WAIT_FAILED {
                return Err(io::Error::last_os_error());
            }
            if GetExitCodeProcess(self.0.hProcess, &mut exit_code).as_bool() {
                Ok(ExitStatus::Status(exit_code))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // SAFETY: FFI calls
        unsafe {
            CloseHandle(self.0.hProcess);
            CloseHandle(self.0.hThread);
        }
    }
}

#[derive(Debug)]
pub enum ExitStatus {
    Status(u32),
}

impl ExitStatus {
    pub fn success(&self) -> bool {
        match self {
            ExitStatus::Status(code) => *code == 0,
        }
    }
}
