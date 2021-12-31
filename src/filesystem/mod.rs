use std;
use std::fs::File;
use std::io::{BufWriter, BufReader, Error, Read, Seek, SeekFrom, Write};
use std::ffi::OsStr;
use std::fs::OpenOptions;

/// Names a file or directory in a FileSystem
pub trait Path {
    fn get_file_system(&self) -> &dyn FileSystem;
    fn as_os_str(&self) -> &OsStr;
    fn to_str(&self) -> Option<&str>;
}

/// Local path
pub struct LocalPath<'a> {
    local_path: &'a std::path::Path,
    filesystem: LocalFileSystem
}

impl LocalPath<'_> {
    pub fn from_std_path(path: &std::path::Path) -> LocalPath {
        LocalPath {
            local_path: path,
            filesystem: LocalFileSystem {}
        }
    }
}

impl Path for LocalPath<'_> {
    fn get_file_system(&self) -> &dyn FileSystem {
        &self.filesystem
    }

    fn as_os_str(&self) -> &OsStr {
        self.local_path.as_os_str()
    }

    fn to_str(&self) -> Option<&str> {
        self.local_path.to_str()
    }
}

/// Generic Filesystem
pub trait FileSystem {
    fn create(&self, path: &dyn Path) -> Result<(), Error>;
    fn append(&self, path: &dyn Path, buffer: &[u8])
        -> Result<(), Error>;
    fn read(&self, path: &dyn Path, buffer: &mut [u8]) -> Result<usize, Error>;
    fn seek_read(&self, path: &dyn Path, offset: u64,
        buffer: &mut [u8]) -> Result<usize, Error>;
    fn close(&self) -> Result<(), Error>;
}

pub struct LocalFileSystem {   
}

impl FileSystem for LocalFileSystem {
    fn create(&self, path: &dyn Path) -> Result<(), Error> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.to_str().unwrap())?;
        return Ok(());
    }

    fn append(&self, path: &dyn Path, buffer: &[u8])
        -> Result<(), Error> {
        let file = OpenOptions::new()
            .create(false)
            .append(true)
            .open(path.to_str().unwrap())?;
        let mut buf_file = BufWriter::new(file);
        buf_file.write_all(buffer)?;
        buf_file.flush()?;
        return Ok(());
    }

    fn read(&self, path: &dyn Path, buffer: &mut [u8]) -> Result<usize, Error> {
        let file = File::open(path.to_str().unwrap())?;
        let bytes = BufReader::new(file).read(buffer)?;
        return Ok(bytes);
    }

    fn seek_read(&self, path: &dyn Path, offset: u64,
        buffer: &mut [u8]) -> Result<usize, Error> {
        let mut file = File::open(path.to_str().unwrap())?;
        // move the cursor 'offset' bytes from the start of the file
        file.seek(SeekFrom::Start(offset))?;
        let bytes = BufReader::new(file).read(buffer)?;
        return Ok(bytes);
    }

    fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}
mod tests;