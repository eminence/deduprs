extern crate walkdir;
use walkdir::WalkDir;

extern crate twox_hash;
use twox_hash::XxHash;

use std::collections::HashMap;
use std::convert::From;
use std::env;
use std::fs;
use std::fs::{File, Metadata};
use std::hash::Hasher;
use std::io::Read;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Instead of looking up the metadata of a file multiple times, cache it!
#[derive(Debug)]
struct MDPath {
    pub p: PathBuf,
    pub md: fs::Metadata,
}
impl MDPath {
    fn from(p: PathBuf) -> Result<MDPath, std::io::Error> {
        let md = fs::metadata(&p)?;
        Ok(MDPath { p, md })
    }
}
impl std::convert::AsRef<Path> for MDPath {
    fn as_ref(&self) -> &Path {
        self.p.as_ref()
    }
}

impl std::convert::AsRef<Metadata> for MDPath {
    fn as_ref(&self) -> &Metadata {
        &self.md
    }
}

/// Returns (hash of file, mtime, filesize)
///
/// The memtable param is a memorization table, indexed by ino
fn hash_file(path: &MDPath, do_hash: bool, memtable: &mut HashMap<u64, u64>) -> (u64, u64, u64) {
    //let builder = RandomXxHashBuilder::default();
    let mut xxhash = XxHash::with_seed(0); //builder.build_hasher();

    let mut file = File::open(path).unwrap();
    let md = file.metadata().unwrap();

    let file_hash: u64 = if do_hash {
        *memtable.entry(md.st_ino()).or_insert_with(|| {
            let mut buf = [0; 4096];
            loop {
                match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(nbytes) => {
                        xxhash.write(&buf[..nbytes]);
                    }
                    Err(e) => panic!(e),
                }
            }
            xxhash.finish()
        })
    } else {
        0
    };

    let mtime = md
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    (file_hash, mtime, md.len())
}

fn atomic_link<A, B>(src: A, dst: B)
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let tmp_dst = dst.as_ref().with_extension("temporary_hardlink");
    fs::hard_link(src, &tmp_dst).expect("Creating temp hardlink");
    fs::rename(tmp_dst, dst).expect("Renaming");
}

/// these files are identical and should be deduplicated
fn hardlink(paths: &[MDPath]) {
    // find the file with the most number of hardlinks, and use that file as the "master"
    // any file that isn't already linked to this master is converted into a link

    let master = paths
        .iter()
        .max_by(|a, b| a.md.st_nlink().cmp(&b.md.st_nlink()))
        .unwrap();

    //println!("master is {:?}", master.p);

    for p in paths {
        if p.md.st_ino() != master.md.st_ino() {
            println!("Must link {} to {}", p.p.display(), master.p.display());
            atomic_link(master, p);
        }
    }
}

/// Hard-link these if they are identical
fn check(paths: Vec<MDPath>) {
    if paths.len() < 2 {
        return;
    } // we can't hardlink a file to itself

    // as a short-circuit, if these all have the same inode, they are already linked and we don't
    // have to do any more checking
    if paths
        .iter()
        .all(|mdp| mdp.md.st_ino() == paths[0].md.st_ino())
    {
        return;
    }

    let mut hash_memtable = HashMap::new();

    // partition these paths by sameness
    let mut table = HashMap::<_, Vec<MDPath>>::new();
    for path in paths {
        // first hash on filesize and mtime only
        table
            .entry(hash_file(&path, false, &mut hash_memtable))
            .or_insert_with(Vec::new)
            .push(path);
    }

    let mut keys_to_rehash = Vec::new();

    for (k, v) in &table {
        let &(_hash, _mtime, size) = k;
        if v.len() > 1 && size > 0 {
            // these files have the same size and mtime, let's hash them to see if they really are
            // the same
            keys_to_rehash.push(*k);
        }
    }

    for key in keys_to_rehash {
        for path in table.remove(&key).unwrap() {
            table
                .entry(hash_file(&path, true, &mut hash_memtable))
                .or_insert_with(Vec::new)
                .push(path);
        }
    }

    // now we can link files
    for (k, v) in table {
        let (_hash, _mtime, size) = k;
        if v.len() > 1 && size > 0 {
            hardlink(&v);
        }
    }
}

/// walk down the first path element, checking to see if each file in the first element exists in
/// the other paths, and if so, attempt to dedup
fn dedup<P>(paths: &[P])
where
    P: AsRef<Path>,
{
    if paths.len() < 2 {
        println!("Must specify more than 2 paths");
        return;
    }

    let prefix = &paths[0].as_ref();

    for entry in WalkDir::new(&paths[0]).follow_links(false) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }

        if let Ok(path) = entry.path().strip_prefix(prefix) {
            //println!("{}", path.display());

            // check to see if this file exists in the other paths
            check(
                paths
                    .iter()
                    .filter_map(|p| {
                        MDPath::from(p.as_ref().join(path)).ok().and_then(|p| {
                            if p.md.is_file() {
                                Some(p)
                            } else {
                                None
                            }
                        })
                    }).collect(),
            );
        }
    }
}

fn main() {
    let dirs: Vec<PathBuf> = env::args()
        .skip(1)
        .filter_map(|arg| PathBuf::from(arg).canonicalize().ok())
        .collect();
    dedup(&dirs);
}
