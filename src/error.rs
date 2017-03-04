use libc::{ strerror, c_int };
use std::ffi::CStr;

/// A enum representing some of the errors that may be encountered during
/// operations with CFile.
///
/// The Errno varient is special: it contains the errno value given by libc,
/// meaning if you want to do something special for certain values of errno,
/// you must destruct Errno(errno) yourself and match the error yourself.
#[derive(Debug)]
pub enum Error {
    /// errno
    Errno(u64),
    BadPath,
    /// bytes_written
    EndOfFile(usize),
    /// bytes written, errno
    WriteError(usize, u64)
}

impl Error {
    /// Converts an error to a human readable form. This returns a CStr which is formed from a
    /// raw *const c_char. This value should not be stored for very long at all, as it is subject to
    /// change.
    ///
    /// This function is probably a lot more dangerous than I reckon.
    pub fn to_cstr(&self) -> &CStr {
        unsafe {
            match self {
                &Error::Errno(x) => {
                    CStr::from_ptr(strerror(x as c_int))
                },
                &Error::WriteError(_, x) => {
                    CStr::from_ptr(strerror(x as c_int))
                },
                &Error::BadPath => {
                    CStr::from_ptr("The path supplied is invalid\0".as_ptr() as *const i8)
                },
                &Error::EndOfFile(_) => {
                    CStr::from_ptr("The end of the file was reached\0".as_ptr() as *const i8)
                }
            }
        }
    }

    /// Returns the errno value equivelent to the Error contained in self.
    /// It is important to note that BadPath and EndOfFile aren't errno errors, but are considered
    /// errors by the CFile struct.
    pub fn errno(&self) -> u64 {
        match *self {
            Error::Errno(err) => err,
            Error::WriteError(_, err) => err,
            _ => 0
        }
    }
}
