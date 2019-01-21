# mdck

Mdck reports broken links found within md files.  It has a `--help` command
line argument whose output looks like:

```
mdck 0.1.0
Clifford T. Matthews <ctm@clifford.t.matthews@gmail.com>
Mdck checks markdown files for link destinations that point to non-existent
files.  Future versions may do other consistency checks, but that particular
check is potentially useful and was easy to implement, so I wrote mdck to get
some more practice programming in Rust.

The source command line arguments can either be a a single hyphen ('-'), a file
name or a directory name.  A hyphen represents standard input, which is also the
default source when no command line arguments are supplied.  Directories are
recursively searched and files with the ".md" extension are checked.  Files
named explicitly as a command line argument do not need to have the ".md"
extension.

Currently, the only check is that if a link destination is a file, then there
must be a file or directory for that destination.  Mdck does not attempt to open
the destination or check permissions. Since the contents of the destination file
are not inspected, mdck prints a warning when a destination contains a fragment.
That warning is to remind you that mdck is not looking for the fragment.  It is
not suggesting that the fragment is not there.

USAGE:
    mdck [sources]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <sources>...    '-' for standard input. Otherwise, file or directory
                    names
```

## Public Domain

Mdck has been released into the public domain, per the [UNLICENSE](UNLICENSE).

### Building

1. [Install rust](https://www.rust-lang.org/learn/get-started)
2. `cargo build --release`
3. The binary will be in `target/release/mdck`

### Rust is fun

Mdck is a toy program I've written to help me learn Rust.
I've tried to use Rust idioms, but I'm sure I've overlooked some.  So,
if you're a seasoned Rust programmer and it's convenient, please
look over the source and make suggestions, preferably as issues.

#### Trivia

Mdck is the spiritual successor to
[ulf](https://archive.org/stream/bitsavers_decpdp11ulltrix11PgmrsManVol11984_33861595/AA-X344B-TC_Ultrix-11_PgmrsManVol1_1984_djvu.txt),
the "universal lineprinter filter".  They are both extremely limited
applications with tongue-in-cheek names that suggest additional utility.
Ulf supported exactly one non-generic printer (the [Diablo](https://en.wikipedia.org/wiki/Diablo_630)),
and mdck has only one consistency check.
