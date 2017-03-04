/*
MIT License

Copyright (c) 2017 Joshua Karns

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
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
