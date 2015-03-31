#![feature(convert)]
#![feature(path_ext)]
#![feature(fs_time)]

use std::path::{Path, PathBuf};
use std::fs::{read_dir, DirEntry};
use std::fs::{File, PathExt};
use std::io::Read;
use std::old_io::fs::lstat;


fn contents_equal<P>(fileA: P, fileB: P) -> bool
where P: AsRef<Path> {

    let mut _fileA = File::open(fileA).unwrap();
    let mut _fileB = File::open(fileB).unwrap();

    let mut bufA : [u8; 1024] = [0; 1024];
    let mut bufB : [u8; 1024] = [0; 1024];
    loop {

        let read_fromA = _fileA.read(&mut bufA);
        let read_fromB = _fileB.read(&mut bufB);
        if read_fromA == read_fromB {
            if read_fromA.is_ok() {
                let len = read_fromA.unwrap();
                if len == 0 { return true; }
                if bufA[0..len] != bufB[0..len] {
                    return false;
                }
            } else {
                return true;
            }
        } else {
            return false;
        }
    

    }

    return false;
    
}

fn check_and_do_link(fileA: &Path, fileB: &Path) {
    let am = lstat(fileA).unwrap();
    let bm = lstat(fileB).unwrap();
   
    if am.unstable.inode == bm.unstable.inode {
        println!("{} and {} are already linked", fileA.display(), fileB.display())
    }

    if am.size != bm.size { return; }
    if am.modified != bm.modified { return; }

    if contents_equal(fileA, fileB) {
        println!("{} and {} are equal!", fileA.display(), fileB.display());

    } else {
        println!("{} and {} are NOT equal!", fileA.display(), fileB.display());
    }
    
}

fn dedup<P>(pathA: P, pathB: P)
where P: AsRef<Path> {

    for entry in read_dir(pathA).unwrap() {
        let entry: PathBuf = entry.unwrap().path();
        
        println!("{}", entry.display());
        if entry.is_file() {
            let other = pathB.as_ref().join(entry.file_name().unwrap());
            if other.exists() {
                println!("  {}", other.display());
                check_and_do_link(&entry, &other);
            }
        }
        //
        

    }
    return;

}


fn main() {

    dedup("testA", "testB");
    println!("Hello, world!");
}
