//! Code for creating and interacting with Windows AppContainers.

use std::{ffi::CString, ptr};

use tracing::debug;
use widestring::U16CString;
use windows::{
    core::{PCSTR, PWSTR},
    w,
    Win32::{
        Foundation::{GetLastError, HANDLE, HLOCAL, NTSTATUS, PSID},
        Security::{
            Authorization::ConvertSidToStringSidW, CreateWellKnownSid, FreeSid, WinLowLabelSid,
            PSECURITY_DESCRIPTOR, SID, SID_AND_ATTRIBUTES, TOKEN_ACCESS_MASK, TOKEN_ALL_ACCESS,
            WELL_KNOWN_SID_TYPE,
        },
        Storage::FileSystem::STANDARD_RIGHTS_ALL,
        System::{
            Kernel::OBJ_INHERIT,
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            Memory::LocalFree,
            WindowsProgramming::OBJECT_ATTRIBUTES,
        },
    },
};

use super::SandboxError;

#[allow(non_camel_case_types)]
type ACCESS_MASK = u32;

type NtCreateLowBoxTokenFn = unsafe extern "system" fn(
    token_handle: *mut HANDLE,
    existing_token_handle: HANDLE,
    desired_access: ACCESS_MASK,
    object_attributes_opt: *const OBJECT_ATTRIBUTES,
    package_sid: *const SID,
    capability_count: u32,
    capabilities: *const SID_AND_ATTRIBUTES,
    handle_count: u32,
    handles: *const HANDLE,
) -> NTSTATUS;

pub fn create_app_container_token(parent: HANDLE) -> Result<HANDLE, SandboxError> {
    // First we have to use LoadLibrary on ntdll and GetProcAddress to find
    // NtCreateLowBoxToken.

    // SAFETY: FFI calls
    let ntdll = unsafe {
        // We use unwrap because if we can't load ntdll then things are so wrong
        // there's no point continuing.
        LoadLibraryW(w!("ntdll.dll")).unwrap()
    };

    // Now find NtCreateLowBoxToken

    // SAFETY: FFI calls
    let nt_create_low_box_token = unsafe {
        // We use unwrap because if we can't find NtCreateLowBoxToken then things
        // are so wrong there's no point continuing.
        std::mem::transmute::<_, NtCreateLowBoxTokenFn>(
            GetProcAddress(ntdll, PCSTR::from_raw("NtCreateLowBoxToken\0".as_ptr())).unwrap(),
        )
    };

    // now create a basic app container
    let mut token_handle = HANDLE(0);

    let sid = get_lowbox_package_sid()?;

    // let object_name = U16CString::from_str("GaelAppContainerNamedObjects\0").unwrap();
    // let attrs = OBJECT_ATTRIBUTES {
    //     Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
    //     RootDirectory: HANDLE(0),
    //     ObjectName: object_name.as_ptr() as *mut _,
    //     Attributes: OBJ_INHERIT as u32,
    //     SecurityDescriptor: ptr::null_mut(),
    //     SecurityQualityOfService: ptr::null_mut(),
    // };

    // SAFETY: FFI calls
    let status = unsafe {
        nt_create_low_box_token(
            &mut token_handle,
            parent,
            TOKEN_ALL_ACCESS.0,
            ptr::null(), // &attrs, // object attributes
            &sid,
            0,           // capability count
            ptr::null(), // capabilities
            0,           // handle count
            ptr::null(), // handles
        )
    };

    if status.is_ok() {
        Ok(token_handle)
    } else {
        Err(SandboxError::AppContainerToken(status))
    }
}

#[repr(transparent)]
struct OwnedSid {
    sid: PSID,
}

impl Drop for OwnedSid {
    fn drop(&mut self) {
        assert!(!self.sid.is_invalid());

        // SAFETY: FFI calls
        unsafe {
            FreeSid(self.sid);
        }
    }
}

fn get_lowbox_package_sid() -> Result<SID, SandboxError> {
    let mut sid = SID::default();
    let mut sid_size = std::mem::size_of_val(&sid) as u32;

    debug!("sid buffer size is {}", sid_size);

    // SAFETY: FFI call
    let result = unsafe {
        CreateWellKnownSid(
            WinLowLabelSid,
            None,
            PSID(&mut sid as *mut _ as *mut _),
            &mut sid_size,
        )
    };

    debug!("filled sid size is {}", sid_size);

    debug!(
        "app container sid is {}",
        format_sid(PSID(&mut sid as *mut _ as *mut _))
    );

    if result.as_bool() {
        Ok(sid)
    } else {
        Err(SandboxError::AppContainerSid(unsafe { GetLastError() }))
    }
}

fn format_sid(sid: PSID) -> String {
    let mut string_sid: PWSTR = PWSTR(ptr::null_mut());

    // SAFETY: FFI call
    let result = unsafe { ConvertSidToStringSidW(sid, &mut string_sid) };

    if result.as_bool() {
        // SAFETY: The string_sid pointer is valid and non-null
        let string_sid_str =
            unsafe { widestring::U16CString::from_ptr_str(string_sid.as_ptr()) }.to_string_lossy();

        // SAFETY: FFI call
        let _ = unsafe { LocalFree(HLOCAL(string_sid.0 as *mut _ as _)) };

        string_sid_str
    } else {
        panic!("failed to convert sid to string")
    }
}
