#[cfg(test)]
mod local_filesystem_test {
    use crate::filesystem::LocalPath;
    use crate::filesystem::FileSystem;
    use tempfile::TempDir;

    #[test]
    fn test_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let dir_path = tmp_dir.path();
        let file_path = dir_path.join("test.txt");
        let local_path = LocalPath::from_std_path(file_path.as_path());
        assert!(local_path.filesystem.create(&local_path).is_ok());
        assert!(file_path.exists());
        // Test recreation fails
        let error = local_path.filesystem.create(&local_path).unwrap_err();        
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn test_append_and_read() {
        let tmp_dir = TempDir::new().unwrap();
        let dir_path = tmp_dir.path();
        let file_path = dir_path.join("test.txt");
        let local_path = LocalPath::from_std_path(file_path.as_path());
        let filesystem = &local_path.filesystem;
        assert!(local_path.filesystem.create(&local_path).is_ok());
        assert!(filesystem.append(&local_path, b"test string 1").is_ok());
        let mut buf: [u8; 1024] = [0x00; 1024];
        assert_eq!(filesystem.read(&local_path, &mut buf).unwrap(), 13);
        assert_eq!(&buf[0..13], b"test string 1");
        assert!(filesystem.append(&local_path, b" test string 2").is_ok());
        assert_eq!(filesystem.read(&local_path, &mut buf).unwrap(), 27);
        assert_eq!(&buf[0..27], b"test string 1 test string 2");
    }

    #[test]
    fn test_seek_read() {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test.txt");
        let local_path = LocalPath::from_std_path(file_path.as_path());
        let filesystem = &local_path.filesystem;
        assert!(local_path.filesystem.create(&local_path).is_ok());
        // Append first string
        assert!(filesystem.append(&local_path, b"test string 1").is_ok());
        // Append second string
        assert!(filesystem.append(&local_path, b" test string 2").is_ok());
        let mut buf: [u8; 1024] = [0x00; 1024];
        // Test seek skipping first string.
        assert_eq!(filesystem.seek_read(&local_path, 13, &mut buf).unwrap(), 14);
        assert_eq!(&buf[0..14], b" test string 2");
        // Test seek to EOF.
        assert_eq!(filesystem.seek_read(&local_path, 27, &mut buf).unwrap(), 0);
        // Test seeking beyond EOF. No errors thrown
        assert_eq!(filesystem.seek_read(&local_path, 200, &mut buf).unwrap(), 0);
    }
}