mod raw;
use std::fmt;
use std::result;
use std::os::raw::c_int;
use std::ffi::CStr;
use std::path::Path;
use std::ptr;
use std::mem;
use std::os::raw::c_char;
use std::ffi::CString;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Error {
    _code: i32,
    message: String,
    _class: i32
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        // Displaying an `Error` simply displays the message from libgit2
        self.message.fmt(f)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Error {
        Error {_code: -1, message, _class: 0}
    }
}

// NulError is what `CString::new` returns if a string 
// has embedded zero bytes
impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        Error {_code: -1, message: e.to_string(), _class: 0}
    }
}

fn check(code: c_int) -> Result<c_int, Error> {
    if code >= 0 {
        return Ok(code)
    }

    unsafe {
        let error = raw::giterr_last();

        // libgit2 ensures that (*error).message is always non null and null
        let message = CStr::from_ptr((*error).message).to_string_lossy().into_owned();

        Err(Error {
            _code: code as i32,
            message,
            _class: (*error).klass as i32
        })
    }
}

// A git repository
pub struct Repository {
    // This must always be a pointer to a live `git_repo`
    // No other `repo` may point to it
    raw: *mut raw::git_repository
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repository, Error> {
        ensure_initialized();

        let path = path_to_cstring(path.as_ref())?;

        let mut repo = ptr::null_mut();
        unsafe {
            check(raw::git_repository_open(&mut repo, path.as_ptr()))?;
        }
        Ok(Repository{ raw: repo})
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        unsafe {
            raw::git_repository_free(self.raw);
        }
    }
}

impl Repository {
    pub fn reference_name_to_id(&self, name: &str) -> Result<Oid, Error> {
        let name = CString::new(name)?;
        unsafe {
            let mut oid = mem::MaybeUninit::uninit();
            check(raw::git_reference_name_to_id(oid.as_mut_ptr(), self.raw, name.as_ptr() as *const c_char))?;
            return Ok(Oid{raw: oid.assume_init()})
        }
    }
}

impl Repository {
    pub fn find_commit(&self, oid: &Oid) -> Result<Commit, Error> {
        let mut commit = ptr::null_mut();
        unsafe {
            check(raw::git_commit_lookup(&mut commit, self.raw, &oid.raw))?;
        }
        Ok(Commit {raw: commit, _marker: PhantomData})
    }
}

fn ensure_initialized() {
    static ONCE: std::sync::Once = std::sync::Once::new();

    ONCE.call_once(|| {
        unsafe {
            check(raw::git_libgit2_init())
                .expect("init libgit2 failed");
            assert_eq!(libc::atexit(shutdown), 0);
        }
    });
}

extern fn shutdown() {
    unsafe {
        if let Err(e) = check(raw::git_libgit2_shutdown()) {
            eprintln!("shutting down libgit2 failed: {}", e);
            std::process::abort();
        }
    }
}

#[cfg(unix)]
fn path_to_cstring(path: &Path) -> Result<CString,Error> {
    use std::os::unix::ffi::OsStrExt;

    Ok(CString::new(path.as_os_str().as_bytes())?)
}

#[cfg(windows)]
fn path_to_cstring(path: &Path) -> Result<CString> {
    match path.to_str() {
        Some(s) => Ok(CString::new(s)?),
        None => {
            let message = format!("Couldn't convert path to '{}' to UTF-8", path.display());
            Err(message.into())
        }
    }

}

pub struct Oid {
    pub raw: raw::git_oid
}


pub struct Commit<'repo> {
    raw: *mut raw::git_commit,
    _marker: PhantomData<&'repo Repository>
}

impl<'repo> Drop for Commit <'repo> {
    fn drop(&mut self) {
        unsafe {
            raw::git_commit_free(self.raw);
        }
    }
}

impl<'repo> Commit<'repo> {
    pub fn author(&self) -> Signature {
        unsafe {
            Signature {
                raw: raw::git_commit_author(self.raw),
                _marker: PhantomData
            }
        }
    }

    pub fn message(&self) -> Option<&str> {
        unsafe {
            let message = raw::git_commit_message(self.raw);
            char_ptr_to_str(self, message)
        }
    }
}

pub struct Signature<'text> {
    raw: *const raw::git_signature,
    _marker: PhantomData<&'text str>
}

impl <'text> Signature<'text> {
    pub fn name(&self) -> Option<&str> {
        unsafe{
            char_ptr_to_str(self, (*self.raw).name)
        }
    }

    pub fn email(&self) -> Option<&str> {
        unsafe{
            char_ptr_to_str(self, (*self.raw).email)
        }
    }
}

unsafe fn char_ptr_to_str<T>(_owner: &T, ptr: *const c_char) -> Option<&str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}