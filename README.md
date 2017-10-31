# Deduplication tool

This tool can deduplicate multiple directories that share the same directory structure.  Deduplication is done by hard-linking.
(While hard-linking is supported on Windows, this tool is only tested on Linux).


## How it works

Imagine you had the following directdory structures:

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

