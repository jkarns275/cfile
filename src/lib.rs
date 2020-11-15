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

#![doc(html_root_url = "https://jkarns275.github.io/cfile/")]

use libc::FILE;

use std:: {
    ffi::CString,
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
    ptr::{null_mut, NonNull},
};

/// A utility function to pull the current value of errno and put it into an Error::Errno
fn get_error<T>() -> Result<T, Error> {
    Err(Error::last_os_error())
}

/// A utility function that creates a "buffer" of len bytes.
/// A Vec is used because it is memory safe and has a bunch of useful functionality (duh).
pub fn buffer(len: usize) -> Vec<u8> {
    vec![0u8; len]
}

/// A &'static str to be passed into the CFile::open method. It will open the file in a way that will allow
/// reading and writing, including overwriting old data. It will not create the file if it does not exist.
pub static RANDOM_ACCESS_MODE: &'static str = "rb+";
/// A &'static str to be passed into the CFile::open method. It will open the file in a way that will allow
/// reading and writing, including overwriting old data
pub static UPDATE: &'static str = "rb+";
/// A &'static str to be passed into the CFile::open method. It will only allow reading.
pub static READ_ONLY: &'static str = "r";
/// A &'static str to be passed into the CFile::open method. It will only allow writing.
pub static WRITE_ONLY: &'static str = "w";
/// A &'static str to be passed into the CFile::open method. It will only allow data to be appended to the
/// end of the file.
pub static APPEND_ONLY: &'static str = "a";
/// A &'static str to be passed into the CFile::open method. It will allow data to be appended to the end of
/// the file, and data to be read from the file. It will create the file if it doesn't exist.
pub static APPEND_READ: &'static str = "a+";
/// A &'static str to be passed into the CFile::open method. It will open the file in a way that will allow
/// reading and writing, including overwriting old data. It will create the file if it doesn't exist
pub static TRUNCATE_RANDOM_ACCESS_MODE: &'static str = "wb+";

/// A wrapper around C's file type.
/// Attempts to mimic the functionality if rust's std::fs::File while still allowing complete
/// control of all I/O operations.
pub struct CFile {
    file_ptr: NonNull<FILE>,
    pub path: CString,
}

impl CFile {
    /// Attempts to open a file in random access mode (i.e. rb+). However, unlike rb+, if the file
    /// doesn't exist, it will be created. To avoid createion, simply call CFile::open(path, "rb+"),
    /// which will return an error if the file doesn't exist.
    /// # Failure
    /// This function will return Err for a whole bunch of reasons, the errno id will be returned
    /// as an Error::Errno(u64). For more information on what that number actually means see
    pub fn open_random_access(path: &str) -> Result<CFile, Error> {
        let _ = Self::create_file(&path); // Ensure the file exists, create it if it doesn't
        Self::open(path, RANDOM_ACCESS_MODE)
    }

    /// Attempts to create a file, and then immedietly close it. If the file already exists, this
    /// function will not do anything. If the file does exist, then it will be created with no
    /// and nothing more (it will be empty).
    pub fn create_file(path: &str) -> Result<(), Error> {
        match Self::open(path, APPEND_READ) {
            Ok(file) => file.close(),
            Err(e) => Err(e),
        }
    }

    /// Attempt to open the file with path p.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, TRUNCATE_RANDOM_ACCESS_MODE};
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
    /// ```
    pub fn open(p: &str, mode: &str) -> Result<CFile, Error> {
        unsafe {
            if let Ok(path) = CString::new(p) {
                if let Ok(mode) = CString::new(mode) {
                    let file_ptr = libc::fopen(path.as_ptr(), mode.as_ptr());
                    if file_ptr.is_null() {
                        get_error()
                    } else {
                        Ok(CFile {
                            file_ptr: NonNull::new_unchecked(file_ptr),
                            path: path,
                        })
                    }
                } else {
                    get_error()
                }
            } else {
                get_error()
            }
        }
    }

    /// Deletes the file from the filesystem, and consumes the object.
    /// # Errors
    /// On error Error::Errno(errno) is returned.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, UPDATE};
    /// use std::io::Write;
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", UPDATE).unwrap();
    /// let _ = file.write_all("Howdy folks".as_bytes());   // Write some data!
    /// let _ = file.delete();                              // The file is gone!
    /// ```
    pub fn delete(self) -> Result<(), Error> {
        unsafe {
            let path = self.path.clone();
            drop(self);
            let result = libc::remove(path.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                get_error()
            }
        }
    }

    /// Attempts to close the file. Consumes the file as well
    /// # Errors
    /// On error Error::Errno(errno) is returned.
    pub fn close(mut self) -> Result<(), Error> {
        unsafe {
            if !(self.file_ptr.as_ptr()).is_null() {
                let res = libc::fclose(self.file_ptr.as_ptr());
                if res == 0 {
                    self.file_ptr = NonNull::new_unchecked(null_mut::<libc::FILE>());
                    Ok(())
                } else {
                    get_error()
                }
            } else {
                Ok(())
            }
        }
    }

    /// Returns the underlying file pointer as a reference. It is returned as a reference to, in theory,
    /// prevent it from being used after the file is closed.
    pub unsafe fn file<'a>(&'a mut self) -> &'a mut libc::FILE {
        &mut (*self.file_ptr.as_ptr())
    }

    /// Returns the current position in the file.
    /// # Errors
    /// On error Error::Errno(errno) is returned.
    pub fn current_pos(&self) -> Result<u64, Error> {
        unsafe {
            let pos = libc::ftell(self.file_ptr.as_ptr());
            if pos != -1 {
                Ok(pos as u64)
            } else {
                get_error()
            }
        }
    }

    /// A utility function to expand a vector without increasing its capacity more than it needs
    /// to be expanded.
    fn expand_buffer(buff: &mut Vec<u8>, by: usize) {
        if buff.capacity() < buff.len() + by {
            buff.reserve(by);
        }
        for _ in 0..by {
            buff.push(0u8);
        }
    }
}

impl Write for CFile {
    /// Attempts to write all of the bytes in buf to the file.
    /// # Errors
    /// If an error occurs during writing, Error::WriteError(bytes_written, errno) will be
    /// returned.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, TRUNCATE_RANDOM_ACCESS_MODE};
    /// use std::io::Write;
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
    /// let _ = file.write_all("Howdy folks".as_bytes());   // Write some data!
    ///
    /// ```
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        unsafe {
            let written_bytes = libc::fwrite(
                buf.as_ptr() as *const libc::c_void,
                1,
                buf.len(),
                self.file_ptr.as_ptr(),
            );
            if written_bytes != buf.len() {
                get_error()
            } else {
                Ok(())
            }
        }
    }

    /// Attempts to write all of the bytes in buf to the file.
    /// # Errors
    /// If an error occurs during writing, Error::WriteError(bytes_written, errno) will be
    /// returned.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, TRUNCATE_RANDOM_ACCESS_MODE};
    /// use std::io::Write;
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
    /// let _ = file.write("Howdy folks".as_bytes());   // Write some data!
    ///
    /// ```
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        unsafe {
            let written_bytes = libc::fwrite(
                buf.as_ptr() as *const libc::c_void,
                1,
                buf.len(),
                self.file_ptr.as_ptr(),
            );
            if written_bytes != buf.len() {
                get_error()
            } else {
                Ok(written_bytes)
            }
        }
    }

    /// Flushes the underlying output stream, meaning it actually writes everything to the
    /// filesystem.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, TRUNCATE_RANDOM_ACCESS_MODE};
    /// use std::io::Write;
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
    /// match file.write_all("Howdy folks!".as_bytes()) {
    ///     Ok(()) => println!("Successfully wrote to the file!"),
    ///     Err(err) => {
    ///         println!("Encountered error: {}", err);
    ///     }
    /// };
    /// let _ = file.flush();   // Upon this call, all data waiting in the output
    ///                         // stream will be written to the file
    /// ```
    fn flush(&mut self) -> Result<(), Error> {
        unsafe {
            let result = libc::fflush(self.file_ptr.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                get_error()
            }
        }
    }
}

impl Read for CFile {
    /// Reads the entire file starting from the current position, expanding buf as needed. On a successful
    /// read, this function will return Ok(bytes_read).
    /// # Errors
    /// If an error occurs during reading, some varient of error will be returned.
    /// # Examples
    /// ```
    /// use cfile_rs::{CFile, TRUNCATE_RANDOM_ACCESS_MODE};
    /// use std::io::{Seek, SeekFrom, Read, Write};
    /// use std::str::from_utf8;
    ///
    /// // Truncate random access mode will overwrite the old "data.txt" file if it exists.
    /// let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
    /// let _ = file.write_all("Howdy folks".as_bytes());   // Write some data!
    /// let _ = file.seek(SeekFrom::Start(0));              // Move back to the beginning of the file
    /// let mut buffer = cfile_rs::buffer(10);              // Create a buffer (a Vec<u8>) to read into
    /// match file.read_to_end(&mut buffer) {               // Read the entire file, expanding our buffer as needed
    ///     Ok(bytes_read) => {
    ///         // It is a bad idea to do this unless you know it is valid utf8
    ///         let as_str = from_utf8(&buffer[0..bytes_read]).unwrap();
    ///         println!("Read '{}' from the file.", as_str);
    ///     },
    ///     Err(err) => {
    ///         println!("Encountered error: {}", err);
    ///     }
    /// };
    /// ```
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        let pos = self.current_pos();
        let _ = self.seek(SeekFrom::End(0));
        let end = self.current_pos();
        match pos {
            Ok(cur_pos) => match end {
                Ok(end_pos) => {
                    if end_pos == cur_pos {
                        return Ok(0);
                    }
                    let to_read = (end_pos - cur_pos) as usize;
                    println!("to_read {}", to_read);
                    if buf.len() < to_read {
                        let to_reserve = to_read - buf.len();
                        Self::expand_buffer(buf, to_reserve);
                    }
                    let _ = self.seek(SeekFrom::Start(cur_pos as u64));
                    match self.read_exact(buf) {
                        Ok(()) => Ok(to_read),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    /// Reads exactly the number of bytes required to fill buf.
    /// # Errors
    /// If the end of the file is reached before buf is filled, Err(EndOfFile(bytes_read)) will be
    /// returned. The data that was read before that will still have been placed into buf.
    ///
    /// Upon some other error, Err(Errno(errno)) will be returned.
    /// # Examples
    /// ```
    ///
    /// ```
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        unsafe {
            let result = libc::fread(
                buf.as_ptr() as *mut libc::c_void,
                1,
                buf.len(),
                self.file_ptr.as_ptr(),
            );
            if result != buf.len() {
                match get_error::<u8>() {
                    Err(err) => {
                        if err.kind() == ErrorKind::UnexpectedEof {
                            Ok(result)
                        } else {
                            Err(err)
                        }
                    }
                    Ok(_) => panic!("This is impossible"),
                }
            } else {
                Ok(result)
            }
        }
    }

    /// Reads exactly the number of bytes required to fill buf.
    /// # Errors
    /// If the end of the file is reached before buf is filled, Err(EndOfFile(bytes_read)) will be
    /// returned. The data that was read before that will still have been placed into buf.
    ///
    /// Upon some other error, Err(Errno(errno)) will be returned.
    /// # Examples
    /// ```
    ///
    /// ```
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        unsafe {
            let result = libc::fread(
                buf.as_ptr() as *mut libc::c_void,
                1,
                buf.len(),
                self.file_ptr.as_ptr(),
            );
            if result == buf.len() {
                Ok(())
            } else {
                // Check if we hit the end of the file
                if libc::feof(self.file_ptr.as_ptr()) != 0 {
                    get_error()
                } else {
                    get_error()
                }
            }
        }
    }
}

impl Seek for CFile {
    /// Changes the current position in the file using the SeekFrom enum.
    ///
    /// To set relative to the beginning of the file (i.e. index is 0 + offset):
    /// ```ignore
    /// SeekFrom::Start(offset)
    /// ```
    /// To set relative to the end of the file (i.e. index is file_lenth - 1 - offset):
    /// ```ignore
    /// SeekFrom::End(offset)
    /// ```
    /// To set relative to the current position:
    /// ```ignore
    /// SeekFrom::End(offset)
    /// ```
    /// # Errors
    /// On error Error::Errno(errno) is returned.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        unsafe {
            let result = match pos {
                SeekFrom::Start(from) => {
                    libc::fseek(self.file_ptr.as_ptr(), from as libc::c_long, libc::SEEK_SET)
                }
                SeekFrom::End(from) => {
                    libc::fseek(self.file_ptr.as_ptr(), from as libc::c_long, libc::SEEK_END)
                }
                SeekFrom::Current(delta) => libc::fseek(
                    self.file_ptr.as_ptr(),
                    delta as libc::c_long,
                    libc::SEEK_CUR,
                ),
            };
            if result == 0 {
                self.current_pos()
            } else {
                get_error()
            }
        }
    }
}

impl Drop for CFile {
    /// Ensures the file stream is closed before abandoning the data.
    fn drop(&mut self) {
        let _ = unsafe {
            if !(self.file_ptr.as_ptr()).is_null() {
                let res = libc::fclose(self.file_ptr.as_ptr());
                if res == 0 {
                    self.file_ptr = NonNull::new_unchecked(null_mut::<libc::FILE>());
                    Ok(())
                } else {
                    get_error()
                }
            } else {
                Ok(())
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{buffer, CFile, Read, Seek, SeekFrom, Write, TRUNCATE_RANDOM_ACCESS_MODE};
    use std::str;

    #[test]
    fn file_flush() {
        let mut file = CFile::open("data.txt", TRUNCATE_RANDOM_ACCESS_MODE).unwrap();
        match file.write_all("Howdy folks!".as_bytes()) {
            Ok(()) => println!("Successfully wrote to the file!"),
            Err(_e) => {
                // darn
            }
        };
        let _ = file.flush(); // Probably unnecessary
        let buf_size = 20;
        let mut buf = buffer(buf_size); // 20 will be more than enough to store our data
        let _ = file.seek(SeekFrom::Start(0)); // Move to 1 byte after the beginning of the file
        let result = file.read_exact(&mut buf); // Read exactly 20 bytes
        match result {
            Ok(()) => {
                // This won't happen since we only wrote 12 bytes,
                let data = &buf[0..buf_size]; // but if it did this is how we could print the data
                                              // as a string.
                let str = str::from_utf8(data).unwrap();
                println!("{}", str);
            }
            Err(_e) => {
                // Oh no!
            }
        };
    }
}
