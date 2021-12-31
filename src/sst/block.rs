use leb128;
use crate::sst::lsm_error::DataStoreError;
use crate::filesystem::Path;
/// Block Size defined for 128 KB.
/// TODO: need to make it configurable 
/// and experiment to set optimal size
static BLOCK_SIZE: usize = 128 * 1024;
/// Block has 'data_' which is contigous block
/// of bytes consisting of contiguous entry.
/// Format of an entry is concatenation of:
///  key_size     : leb128 encoding of key size
///  value_size   : leb128 encoding of value size
///  key bytes    : char[key_size]
///  value bytes  : char[value_size]
struct Block {
    /// Block data. Check the comment at struct Block
    /// level for the detail about format. It would be
    /// written to the Disk in the same format.
    pub data_: Vec<u8>,
    /// Length of data written to 'data_'.
    pub current_pos_: usize,
    /// Whether Block is closed or not.
    pub finished_: bool
}

impl Block {
    fn new() -> Block {
        Block {
            data_ : vec![0; BLOCK_SIZE],
            current_pos_ : 0,
            finished_ : false
        }
    }

    fn add(&mut self, key: &[u8], value: &[u8])
        -> Result<(), DataStoreError> {
        debug_assert!(!self.finished_, "Block is already closed");
        let key_len = key.len();
        let value_len = value.len();
        let mut writable = &mut self.data_[self.current_pos_..];
        let key_len_bytes =
            leb128::write::unsigned(&mut writable,
                key_len.try_into().unwrap())
            .unwrap();
        self.current_pos_ += key_len_bytes;
        writable = &mut self.data_[self.current_pos_..];
        let value_len_bytes =
            leb128::write::unsigned(&mut writable,
                value_len.try_into().unwrap())
            .unwrap();
        self.current_pos_ += value_len_bytes;
        self.data_[self.current_pos_..self.current_pos_ + key_len]
            .copy_from_slice(key);
        self.current_pos_ += key_len;
        self.data_[self.current_pos_..self.current_pos_ + value_len]
            .copy_from_slice(value);
        self.current_pos_ += value_len;
        return Ok(());
    }

    fn finish(&mut self) -> &[u8] {
        self.finished_ = true;
        return &self.data_[0..self.current_pos_];
    }

    fn write_to_sst(&self, path: &dyn Path) -> Result<(), DataStoreError> {
        assert!(self.finished_, "Write cannot be called
            before closing the block");
        assert!(self.current_pos_ > 0, "Empty block cannot be written");
        path.get_file_system().append(path, &self.data_[0..self.current_pos_])?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::sst::block::Block;
    use tempfile::TempDir;
    use crate::filesystem::{Path, LocalPath};
    #[test]
    fn insert_one_key_value() {
        let mut b = Block::new();
        assert!(b.add(b"key", b"value").is_ok());
        b.finish();
        assert_eq!(b.current_pos_, 10);
        let expected_bytes =  [&[0x03 as u8, 0x05 as u8] as &[u8],
            b"keyvalue"].concat();
        let actual_bytes: &[u8] = &(b.data_)[0..b.current_pos_];
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    fn insert_longer_keys_and_values() {
        let mut b = Block::new();
        let test_str_129 = ["c"; 129].join("");
        // insert more than 129 characters i.e., > 128
        // so that its length takes 2 bytes when encoded
        // using leb128.
        let value = test_str_129.as_bytes(); 
        assert!(b.add(b"key", value).is_ok());
        let mut expected_len = 3 /*key chars*/ + 129 /*value chars*/
            + 1 /*keylen in leb128*/ + 2 /*valuelen in leb128*/;
        assert_eq!(b.current_pos_, expected_len);
        let mut expected_bytes = [&[0x03 as u8, 0x81 as u8, 0x01 as u8],
            b"key", value].concat();
        let mut actual_bytes = &(b.data_)[0..b.current_pos_];
        assert_eq!(expected_bytes, actual_bytes);
        // insert 129 characters key now.
        let key = test_str_129.as_bytes();
        assert!(b.add(key, value).is_ok());
        expected_len += 129 /*key chars*/ + 129 /*value chars*/
        + 2 + 2 /*both valuelen and keylen in leb128*/;
        assert_eq!(b.current_pos_, expected_len);
        expected_bytes = [&expected_bytes[0..],
        &[0x81 as u8, 0x01 as u8, 0x81 as u8, 0x01 as u8],
        key, value].concat();
        actual_bytes = &(b.data_)[0..b.current_pos_];
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    fn test_write_to_sst() {
        let mut b = Block::new();
        assert!(b.add(b"key", b"value").is_ok());
        b.finish();
        // create sst file
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test.sst");
        let local_path = LocalPath::from_std_path(file_path.as_path());
        let filesystem = local_path.get_file_system();
        assert!(filesystem.create(&local_path).is_ok());
        assert!(b.write_to_sst(&local_path).is_ok());
        let mut buf: [u8; 20] = [0; 20];
        let expected_bytes =  [&[0x03 as u8, 0x05 as u8] as &[u8],
            b"keyvalue"].concat();
        let expected_len = expected_bytes.len();
        let actual_len = filesystem.read(&local_path, &mut buf).unwrap();
        assert_eq!(actual_len, expected_len,
            "Expected len is {}, but actual len got is {}", actual_len, expected_len);
        assert_eq!(buf[0..expected_len], expected_bytes,
            "Bytes written to SST file differs than expected");
    }
}