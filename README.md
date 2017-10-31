# Deduplication tool

This tool can deduplicate multiple directories that share the same directory structure.  Deduplication is done by hard-linking.
(While hard-linking is supported on Windows, this tool is only tested on Linux).


## How it works

Imagine you had the following directory structures:

```
./foo
|-- one/
|   |-- a.txt
|   `-- b.txt
`-- c.txt    


```

which was replicated in several different directories, for example `testA/`, `testB/` and `testC/`.  The directories pass in on the
command line are called "root directories" or "roots".  


If you run `./dedup test*`, the first directory will be used as the primary tree to be walked.  For each file in it,
dedup.rs will check to see if it exists in the other roots.  For example, does `testB/foo/one/a.txt` and `testC/foo/one/a.txt` exist?

Each file that exists in multiple roots will be checked for sameness.  Any files that are identical will be hard-linked together.  

To create the links, first a new link is created to a temporary file, and then then temporary file is moved on top of the real file.

## Important note

When using hardlinking for deduplication, it's important to remember that editing a file will change *every* path that links to that
file.  This can be very surprising in some situations.  Thus it is strongly recommended that once a folder is deduplicated, it be marked
as read-only, and never written to.  


## Details

When looking for files to deduplicate, they must exist in at least 2 of the roots.  They need not exist in every root.

Given a set of files with the same name, the set is first partitioned based on sameness.  As an example, if you had 5 files total, and files 1 and 2 were the same, and files 3 and 4 where the same, and file 5 was different from everything else, then files 1 and 2 would be hardlinked and files 3 and 4 would be hardlinked.  

As an optimiation, files with different mtimes and file sizes are never considered the same.  If they are, then the files are then
hashed with with [xxHash](https://github.com/Cyan4973/xxHash).  

Given a set of files that have all been confirmed to be the same, the file with the most number of hardlinks is considered to be
the "master".  All other files are then linked to point to this master file.
