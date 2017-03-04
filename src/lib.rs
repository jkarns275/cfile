#![doc(html_root_url = "https://jkarns275.github.io/cfile/")]
#![feature(libc)]
extern crate libc;

pub mod error;
pub use error::*;

pub mod cfile;
pub use cfile::*;


#[cfg(test)]
mod tests {
    use std::str;
    use cfile;
    use cfile::*;
    use error::Error;
    use std::io::SeekFrom;

    #[test]
    fn file_flush() {
        let file = CFile::open("data.txt", TRUNCATAE_RANDOM_ACCESS_MODE).unwrap();
        match file.write_all("Howdy folks!".as_bytes()) {
            Ok(()) => println!("Successfully wrote to the file!"),
            Err(err) => {
                let error_str = err.to_cstr();
                let errno = err.errno();
                println!("Encountered error {}: {:?}", errno, error_str);
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
            Err(Error::EndOfFile(bytes_read)) => {
                // Oh no! There weren't enough bytes left to fill our buf! We did get some data though.
                let data = &buf[0..bytes_read];
                let str = str::from_utf8(data).unwrap();
                println!("{}", str);
            },
            _ => { /* Some other error happened 😢 */ }
        };
    }
}
