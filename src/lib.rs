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
#![feature(libc)]
extern crate libc;

pub mod cfile;
pub use cfile::*;

#[cfg(test)]
mod tests {
    use std::str;
    use cfile;
    use cfile::*;
    use std::io::SeekFrom;
    use std::io::Write;
    use std::io::Read;
    use std::io::Seek;
    #[test]
    fn file_flush() {
        let mut file = CFile::open("data.txt", TRUNCATAE_RANDOM_ACCESS_MODE).unwrap();
        match file.write_all("Howdy folks!".as_bytes()) {
            Ok(()) => println!("Successfully wrote to the file!"),
            Err(e) => {
                // darn
            }
        };
        let _ = file.flush();                       // Probably unnecessary
        let buf_size = 20;
        let mut buf = cfile::buffer(buf_size);      // 20 will be more than enough to store our data
        let _ = file.seek(SeekFrom::Start(0));      // Move to 1 byte after the beginning of the file
        let result = file.read_exact(&mut buf);     // Read exactly 20 bytes
        match result {
            Ok(()) => {                             // This won't happen since we only wrote 12 bytes,
                let data = &buf[0..buf_size];       // but if it did this is how we could print the data
                                                    // as a string.
                let str = str::from_utf8(data).unwrap();
                println!("{}", str);
            },
            Err(e) => {
                // Oh no!
            },
        };
    }
}
