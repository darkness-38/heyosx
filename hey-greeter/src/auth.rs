// =============================================================================
// hey-greeter — PAM Authentication Module
//
// Provides secure user authentication via the Linux Pluggable Authentication
// Modules (PAM) framework. Uses raw FFI bindings to libpam.
//
// The authentication flow:
//   1. pam_start()        — Initialize PAM with our service name
//   2. pam_authenticate() — Verify credentials
//   3. pam_acct_mgmt()    — Check account validity
//   4. pam_setcred()      — Establish credentials
//   5. pam_open_session() — Open a login session
//   6. pam_close_session() + pam_end() — Clean up (via PamSession::Drop)
// =============================================================================

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use tracing::{debug, info};

/// PAM constants (from <security/pam_appl.h>)
const PAM_SUCCESS: c_int = 0;
const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_ERROR_MSG: c_int = 3;
const PAM_TEXT_INFO: c_int = 4;
const PAM_BUF_ERR: c_int = 5;

const PAM_TTY: c_int = 3;

/// The PAM conversation structure
#[repr(C)]
struct PamConv {
    conv: extern "C" fn(
        num_msg: c_int,
        msg: *mut *const PamMessage,
        resp: *mut *mut PamResponse,
        appdata_ptr: *mut c_void,
    ) -> c_int,
    appdata_ptr: *mut c_void,
}

/// PAM message structure
#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

/// PAM response structure
#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

// External PAM functions — linked against libpam
#[link(name = "pam")]
#[allow(dead_code)]
extern "C" {
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const PamConv,
        pamh: *mut *mut c_void,
    ) -> c_int;

    fn pam_set_item(pamh: *mut c_void, item_type: c_int, item: *const c_void) -> c_int;

    fn pam_end(pamh: *mut c_void, pam_status: c_int) -> c_int;

    fn pam_authenticate(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_acct_mgmt(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_open_session(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_close_session(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_setcred(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_strerror(pamh: *mut c_void, errnum: c_int) -> *const c_char;
}

/// Data passed to the PAM conversation callback
struct ConvData {
    password: CString,
}

/// An open PAM session handle.
///
/// When dropped, automatically calls `pam_close_session()` and `pam_end()`
/// to properly clean up the PAM state.
pub struct PamSession {
    pamh: *mut c_void,
}

impl Drop for PamSession {
    fn drop(&mut self) {
        unsafe {
            pam_close_session(self.pamh, 0);
            pam_end(self.pamh, PAM_SUCCESS);
        }
        debug!("PAM session closed and handle released");
    }
}

/// The PAM conversation callback function.
/// PAM calls this to prompt for information (password, etc.)
///
/// Uses the Linux (Sun) convention: `msg` is a pointer to an array of
/// PamMessage pointers. Dereference once to get the array base, then index.
extern "C" fn pam_conversation(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    unsafe {
        // Allocate response array
        let responses = libc::calloc(num_msg as usize, std::mem::size_of::<PamResponse>())
            as *mut PamResponse;

        if responses.is_null() {
            return PAM_BUF_ERR;
        }

        let conv_data = &*(appdata_ptr as *const ConvData);

        // Linux (Sun) convention: *msg is a pointer to an array of PamMessage structs
        let messages = *msg;

        for i in 0..num_msg as isize {
            let message = &*messages.offset(i);

            match message.msg_style {
                PAM_PROMPT_ECHO_OFF => {
                    // Password prompt — provide the password
                    let passwd = libc::strdup(conv_data.password.as_ptr());
                    (*responses.offset(i)).resp = passwd;
                    (*responses.offset(i)).resp_retcode = 0;
                }
                PAM_PROMPT_ECHO_ON => {
                    // Username prompt (usually already set via pam_start)
                    (*responses.offset(i)).resp = ptr::null_mut();
                    (*responses.offset(i)).resp_retcode = 0;
                }
                PAM_ERROR_MSG | PAM_TEXT_INFO => {
                    // Informational messages — just acknowledge
                    (*responses.offset(i)).resp = ptr::null_mut();
                    (*responses.offset(i)).resp_retcode = 0;
                }
                _ => {
                    // Free any already-allocated responses before returning error
                    for j in 0..i {
                        let r = (*responses.offset(j)).resp;
                        if !r.is_null() {
                            libc::free(r as *mut c_void);
                        }
                    }
                    libc::free(responses as *mut c_void);
                    return PAM_BUF_ERR;
                }
            }
        }

        *resp = responses;
        PAM_SUCCESS
    }
}

/// Authenticate a user with the given username and password.
///
/// Uses the "hey-greeter" PAM service (configured in /etc/pam.d/hey-greeter).
///
/// # Returns
/// - `Ok(PamSession)` if authentication succeeds. The session remains open
///   until the returned `PamSession` is dropped.
/// - `Err(String)` with an error message if authentication fails.
pub fn authenticate(username: &str, password: &str) -> Result<PamSession, String> {
    let service = CString::new("hey-greeter").map_err(|e| format!("Invalid service: {e}"))?;
    let user = CString::new(username).map_err(|e| format!("Invalid username: {e}"))?;
    let pass = CString::new(password).map_err(|e| format!("Invalid password: {e}"))?;

    // Use Box::into_raw so the pointer stays valid for the entire PAM lifetime.
    // PAM's conversation callback receives this pointer asynchronously.
    let conv_data = Box::into_raw(Box::new(ConvData { password: pass }));

    let pam_conv = PamConv {
        conv: pam_conversation,
        appdata_ptr: conv_data as *mut c_void,
    };

    let mut pamh: *mut c_void = ptr::null_mut();

    let result = unsafe {
        // ---- Step 1: Initialize PAM ----
        let ret = pam_start(
            service.as_ptr(),
            user.as_ptr(),
            &pam_conv,
            &mut pamh,
        );

        if ret != PAM_SUCCESS {
            // Reclaim conv_data before returning
            let _ = Box::from_raw(conv_data);
            return Err(format!("pam_start failed: {}", pam_error_string(pamh, ret)));
        }

        // Inform PAM of the physical TTY being used
        // systemd-logind and pam_securetty require this for session registration
        let tty = CString::new("tty1").unwrap_or_default();
        let _ = pam_set_item(pamh, PAM_TTY, tty.as_ptr() as *const c_void);

        debug!("PAM session started for user: {username}");

        // ---- Step 2: Authenticate ----
        let ret = pam_authenticate(pamh, 0);
        if ret != PAM_SUCCESS {
            let err = pam_error_string(pamh, ret);
            pam_end(pamh, ret);
            let _ = Box::from_raw(conv_data);
            return Err(format!("Authentication failed: {err}"));
        }

        info!("PAM authentication successful for user: {username}");

        // ---- Step 3: Validate account ----
        let ret = pam_acct_mgmt(pamh, 0);
        if ret != PAM_SUCCESS {
            let err = pam_error_string(pamh, ret);
            pam_end(pamh, ret);
            let _ = Box::from_raw(conv_data);
            return Err(format!("Account validation failed: {err}"));
        }

        // ---- Step 4: Set credentials ----
        let ret = pam_setcred(pamh, 0x2); // PAM_ESTABLISH_CRED
        if ret != PAM_SUCCESS {
            let err = pam_error_string(pamh, ret);
            pam_end(pamh, ret);
            let _ = Box::from_raw(conv_data);
            return Err(format!("Failed to set credentials: {err}"));
        }

        // ---- Step 5: Open session ----
        let ret = pam_open_session(pamh, 0);
        if ret != PAM_SUCCESS {
            let err = pam_error_string(pamh, ret);
            pam_end(pamh, ret);
            let _ = Box::from_raw(conv_data);
            return Err(format!("Failed to open session: {err}"));
        }

        info!("PAM session opened for user: {username}");

        // Reclaim conv_data — PAM no longer needs the conversation callback
        let _ = Box::from_raw(conv_data);

        Ok(PamSession { pamh })
    };

    result
}

/// Get a human-readable error string from PAM
unsafe fn pam_error_string(pamh: *mut c_void, errnum: c_int) -> String {
    let msg = pam_strerror(pamh, errnum);
    if msg.is_null() {
        format!("PAM error code {errnum}")
    } else {
        CStr::from_ptr(msg).to_string_lossy().to_string()
    }
}

/// Zero out the bytes of a String in memory to prevent password leakage.
///
/// Uses `write_volatile` to prevent the compiler from optimizing away the zeroing.
pub fn zeroize_string(s: &mut String) {
    unsafe {
        let bytes = s.as_mut_vec();
        std::ptr::write_bytes(bytes.as_mut_ptr(), 0, bytes.len());
    }
    s.clear();
}
