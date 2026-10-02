use std::ffi::c_void;

const CRED_TYPE_GENERIC: u32 = 1;

#[repr(C)]
struct CredentialW {
    flags: u32,
    type_: u32,
    target_name: *mut u16,
    comment: *mut u16,
    last_written: u64,
    credential_blob_size: u32,
    credential_blob: *mut u8,
    persist: u32,
    attribute_count: u32,
    attributes: *mut c_void,
    target_alias: *mut u16,
    user_name: *mut u16,
}

#[cfg(windows)]
#[link(name = "Advapi32")]
unsafe extern "system" {
    fn CredReadW(
        target_name: *const u16,
        type_: u32,
        reserved_flags: u32,
        credential: *mut *mut CredentialW,
    ) -> i32;
    fn CredEnumerateW(
        filter: *const u16,
        flags: u32,
        count: *mut u32,
        credentials: *mut *mut *mut CredentialW,
    ) -> i32;
    fn CredFree(buffer: *mut c_void);
}

/// A generic Windows credential whose secret decoded as text.
#[derive(Clone)]
pub struct StoredCredential {
    pub target: String,
    pub last_written: u64,
    pub secret: String,
}

#[cfg(windows)]
pub fn read_generic(target: &str) -> Option<String> {
    let target_wide = wide(target);
    let mut credential: *mut CredentialW = std::ptr::null_mut();
    let ok = unsafe { CredReadW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    if ok == 0 || credential.is_null() {
        return None;
    }

    let secret = unsafe { credential_secret(&*credential) };
    unsafe { CredFree(credential as *mut c_void) };
    secret
}

#[cfg(not(windows))]
pub fn read_generic(_target: &str) -> Option<String> {
    None
}

/// Enumerate all generic credentials whose target starts with `prefix`.
#[cfg(windows)]
pub fn enumerate_generic(prefix: &str) -> Vec<StoredCredential> {
    let filter = wide(&format!("{prefix}*"));
    let mut count = 0u32;
    let mut credentials: *mut *mut CredentialW = std::ptr::null_mut();
    let ok = unsafe { CredEnumerateW(filter.as_ptr(), 0, &mut count, &mut credentials) };
    if ok == 0 || credentials.is_null() {
        return Vec::new();
    }

    let mut found = Vec::new();
    unsafe {
        for &credential in std::slice::from_raw_parts(credentials, count as usize) {
            let Some(credential) = credential.as_ref() else {
                continue;
            };
            if credential.type_ != CRED_TYPE_GENERIC {
                continue;
            }
            let (Some(target), Some(secret)) = (
                wide_ptr_to_string(credential.target_name),
                credential_secret(credential),
            ) else {
                continue;
            };
            found.push(StoredCredential {
                target,
                last_written: credential.last_written,
                secret,
            });
        }
        CredFree(credentials as *mut c_void);
    }
    found
}

#[cfg(not(windows))]
pub fn enumerate_generic(_prefix: &str) -> Vec<StoredCredential> {
    Vec::new()
}

#[cfg(windows)]
unsafe fn credential_secret(credential: &CredentialW) -> Option<String> {
    if credential.credential_blob_size == 0 || credential.credential_blob.is_null() {
        return None;
    }
    let slice = unsafe {
        std::slice::from_raw_parts(
            credential.credential_blob,
            credential.credential_blob_size as usize,
        )
    };
    decode_secret(slice)
}

/// Secrets decode from UTF-8 or UTF-16LE.
pub fn decode_secret(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    if bytes.len() >= 2
        && bytes.len().is_multiple_of(2)
        && bytes.iter().skip(1).step_by(2).all(|byte| *byte == 0)
    {
        let units = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .ok()
            .map(|s| s.trim_matches('\0').to_string());
    }
    String::from_utf8(bytes.to_vec())
        .ok()
        .map(|s| s.trim_matches('\0').to_string())
}

#[cfg(windows)]
unsafe fn wide_ptr_to_string(value: *const u16) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let mut length = 0;
    while unsafe { *value.add(length) } != 0 {
        length += 1;
    }
    let slice = unsafe { std::slice::from_raw_parts(value, length) };
    String::from_utf16(slice).ok()
}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// TokenFuel account secret persistence in Windows Credential Manager.
pub fn save_account_secret(account_id: &str, secret: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("TokenFuel", account_id)
        .map_err(|e| format!("Credential Manager unavailable: {e}"))?;
    entry
        .set_password(secret)
        .map_err(|e| format!("Could not save secret: {e}"))?;
    Ok(())
}

pub fn read_account_secret(account_id: &str) -> Option<String> {
    let entry = keyring::Entry::new("TokenFuel", account_id).ok()?;
    entry.get_password().ok()
}

pub fn delete_account_secret(account_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new("TokenFuel", account_id)
        .map_err(|_| "Windows Credential Manager unavailable.")?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not delete the saved account secret from Credential Manager.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn isolated_windows_credential_roundtrip_and_deletion() {
        let id = format!("test-{}", uuid::Uuid::new_v4());
        save_account_secret(&id, "synthetic-session").unwrap();
        assert_eq!(
            read_account_secret(&id).as_deref(),
            Some("synthetic-session")
        );
        delete_account_secret(&id).unwrap();
        assert!(read_account_secret(&id).is_none());
        delete_account_secret(&id).unwrap();
    }

    #[test]
    fn secrets_decode_from_utf8_and_utf16() {
        assert_eq!(
            decode_secret(b"gho_test_token").as_deref(),
            Some("gho_test_token")
        );
        let utf16: Vec<u8> = "gho_test_token"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        assert_eq!(decode_secret(&utf16).as_deref(), Some("gho_test_token"));
        assert_eq!(decode_secret(b""), None);
    }
}
