use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::mem;
use std::str;

use fatfs::{FsOptions, StdIoWrapper};
use fscommon::BufStream;

const FAT12_IMG: &str = "fat12.img";
const FAT16_IMG: &str = "fat16.img";
const FAT32_IMG: &str = "fat32.img";
const IMG_DIR: &str = "resources";
const TMP_DIR: &str = "tmp";
const TEST_STR: &str = "Hi there Rust programmer!\n";
const TEST_STR2: &str = "Rust is cool!\n";

type FileSystem = fatfs::FileSystem<StdIoWrapper<BufStream<fs::File>>>;

fn call_with_tmp_img<F: Fn(&str)>(f: F, filename: &str, test_seq: u32) {
    let _ = env_logger::builder().is_test(true).try_init();
    let img_path = format!("{}/{}", IMG_DIR, filename);
    let tmp_path = format!("{}/{}-{}", TMP_DIR, test_seq, filename);
    fs::create_dir(TMP_DIR).ok();
    fs::copy(img_path, &tmp_path).unwrap();
    f(tmp_path.as_str());
    fs::remove_file(tmp_path).unwrap();
}

fn open_filesystem_rw(tmp_path: &str) -> FileSystem {
    let file = fs::OpenOptions::new().read(true).write(true).open(tmp_path).unwrap();
    let buf_file = BufStream::new(file);
    let options = FsOptions::new().update_accessed_date(true);
    FileSystem::new(buf_file, options).unwrap()
}

fn call_with_fs<F: Fn(FileSystem)>(f: F, filename: &str, test_seq: u32) {
    let callback = |tmp_path: &str| {
        let fs = open_filesystem_rw(tmp_path);
        f(fs);
    };
    call_with_tmp_img(callback, filename, test_seq);
}

fn test_join_short_file(fs: FileSystem) {
    let root_dir = fs.root_dir();
    let mut file1 = root_dir.create_file("short1.txt").expect("open file");
    let mut file2 = root_dir.create_file("short2.txt").expect("open file");
    file1.truncate().unwrap();
    file1.write_all(TEST_STR.as_bytes()).unwrap();
    file2.write_all(TEST_STR2.as_bytes()).unwrap();
    file1.flush().unwrap();
    file2.flush().unwrap();
    root_dir.merge_files("short1.txt", "short2.txt").unwrap();
    let mut file1 = root_dir.create_file("short1.txt").expect("open file");
    let sector_size = fs.cluster_size();
    let mut sector_one = vec![0u8; sector_size as usize];
    let mut sector_two = vec![0u8; sector_size as usize];
    for (i, bytes) in TEST_STR.as_bytes().iter().enumerate() {
        sector_one[i] = *bytes;
    }
    for (i, bytes) in TEST_STR2.as_bytes().iter().enumerate() {
        sector_two[i] = *bytes;
    }
    sector_one.append(&mut sector_two);
    file1.seek(io::SeekFrom::Start(0)).unwrap();
    let mut buf = Vec::new();
    file1.read_to_end(&mut buf).unwrap();
    assert_eq!(&sector_one, &buf);
}

#[test]
fn test_join_file_fat12() {
    call_with_fs(test_join_short_file, FAT12_IMG, 1)
}

#[test]
fn test_join_file_fat16() {
    call_with_fs(test_join_short_file, FAT16_IMG, 1)
}

#[test]
fn test_join_file_fat32() {
    call_with_fs(test_join_short_file, FAT32_IMG, 1)
}
