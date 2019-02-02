mod error;

pub use error::MdckError;

use {
    pulldown_cmark::{
        Event::{self, Start},
        Parser,
        Tag::Link,
    },
    std::{
        fmt::{self, Display},
        fs::{self, File, Metadata},
        io::{self, Read},
        path::{Path, PathBuf},
        str::FromStr,
    },
    structopt::StructOpt,
    walkdir::{DirEntry, WalkDir},
};

enum Source {
    Stdin,
    File(PathBuf),
    Directory(PathBuf),
}

impl Source {
    fn is_stdin(&self) -> bool {
        match self {
            Source::Stdin => true,
            _ => false,
        }
    }
}

impl FromStr for Source {
    type Err = MdckError;

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        if arg == "-" {
            Ok(Source::Stdin)
        } else {
            let path = Path::new(&arg);
            let info = Config::metadata(path)?;
            let constructor = if info.is_dir() {
                Source::Directory
            } else {
                Source::File
            };
            Ok(constructor(path.to_path_buf()))
        }
    }
}

#[derive(StructOpt)]
#[structopt()]
/// Mdck checks markdown files for link destinations that point to
/// non-existent files.  Future versions may do other consistency
/// checks, but that particular check is potentially useful and was
/// easy to implement, so I wrote mdck to get some more practice
/// programming in Rust.
///
/// The source command line arguments can either be a a single hyphen
/// ('-'), a file name or a directory name.  A hyphen represents
/// standard input, which is also the default source when no command
/// line arguments are supplied.  Directories are recursively searched
/// and files with the ".md" extension are checked.  Files named
/// explicitly as a command line argument do not need to have the
/// ".md" extension.
///
/// Currently, the only check is that if a link destination is a file,
/// then there must be a file or directory for that destination.  Mdck
/// does not attempt to open the destination or check permissions.
/// Since the contents of the destination file are not inspected, mdck
/// prints a warning when a destination contains a fragment.  That
/// warning is to remind you that mdck is not looking for the
/// fragment.  It is not suggesting that the fragment is not there.
pub struct Config {
    /// '-' for standard input. Otherwise, file or directory names
    sources: Vec<Source>,
}

impl Config {
    pub fn new() -> Result<Self, MdckError> {
        let mut config = Config::from_iter_safe(std::env::args())?;
        let sources = &mut config.sources;

        if sources.is_empty() {
            sources.push(Source::Stdin)
        } else {
            Self::multiple_stdin(sources)?
        }

        Ok(config)
    }

    // We have our own metadata function because it's a call that can
    // easily fail (i.e., when there is no file), so we want to be
    // sure our error message has enough context.
    //
    // OTOH, that may be suggestive of us not having enough context in
    // errors in general, because if we get an error from something
    // that rarely fails and don't print enough context it will be
    // even harder to figure out.
    fn metadata(path: &Path) -> Result<Metadata, MdckError> {
        match fs::metadata(path) {
            Ok(result) => Ok(result),
            Err(e) => Err(MdckError::from(&format!("{:?}: {}", path, e)[..])),
        }
    }

    fn multiple_stdin(sources: &[Source]) -> Result<(), MdckError> {
        let mut count = 0;

        for filename in sources {
            if filename.is_stdin() {
                count += 1;
                if count > 1 {
                    return Err(MdckError::from("You may only use stdin once"));
                }
            }
        }
        Ok(())
    }
}

pub fn ck_sources(config: &Config) -> Result<(), MdckError> {
    for source in &config.sources {
        match source {
            Source::Stdin => show_stdin_broken_links()?,
            Source::File(path) => show_file_broken_links(path)?,
            Source::Directory(path) => show_broken_links(path)?,
        };
    }
    Ok(())
}

fn show_broken_links(path: &Path) -> Result<(), MdckError> {
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok) // TODO: stop silently discarding errors
        .filter(is_md)
    {
        show_broken_direntry_links(&entry)?
    }

    Ok(())
}

fn show_broken_readable(f: &mut dyn Read, parent: &Path, label: &str) -> Result<(), MdckError> {
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let contents = String::from_utf8(buffer)?;

    show_broken_links_for(&contents, &parent, label)
}

fn show_stdin_broken_links() -> Result<(), MdckError> {
    let stdin = io::stdin();
    let mut f = stdin.lock();

    show_broken_readable(&mut f, &Path::new("."), "STDIN")
}

fn is_md(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".md"))
        .unwrap_or(false)
}

// TODO: if there is a fragment, see if that fragment actually exists.
//
// This is slightly complicated by the fact that GitHub creates the
// anchor as part of their "special sauce" (see
// https://github.com/github/markup#github-markup and look at step 4).
//
// It turns out Comrak (https://github.com/kivikakk/comrak) has code
// that generates anchors, but it's not 100% compatible (e.g., it
// allows question marks through the anchorization process) and it
// also isn't available outside the markdown -> HTML conversion code
// that calls it.  I've already written a tentative PR that makes it
// accessible, but I haven't submitted it in part due to the
// compatibility issue.

fn show_broken_links_for(contents: &str, parent: &Path, label: &str) -> Result<(), MdckError> {
    for broken_link in Parser::new(contents)
        .with_offset()
        .filter_map(link_files_from_events)
        //        .inspect(|link_file| {
        //            if let Some(fragment) = &link_file.fragment {
        //                let line = line_from_offset(contents, link_file.offset);
        //                eprintln!(
        //                    "Warning: link with fragment \"{}\" found on line {} of {:?}",
        //                    fragment, line, label
        //                );
        //            }
        //        })
        .filter(|uri| broken_link_file(uri, parent))
    {
        let line = line_from_offset(contents, broken_link.offset);
        println!(
            "{:?} at line {} contains the broken link: {}",
            label, line, broken_link
        );
    }

    Ok(())
}

fn line_from_offset(contents: &str, offset: usize) -> usize {
    let span = &contents[0..offset];
    bytecount::count(span.as_bytes(), b'\n') + 1
}

fn show_broken_direntry_links(entry: &DirEntry) -> Result<(), MdckError> {
    let path = entry.path();
    let parent = path.parent().unwrap_or_else(|| Path::new("/"));
    let contents = fs::read_to_string(path)?;
    show_broken_links_for(&contents, &parent, path.to_str().unwrap())
}

fn show_file_broken_links(path: &PathBuf) -> Result<(), MdckError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("/"));

    show_broken_readable(&mut File::open(path)?, &parent, path.to_str().unwrap())
}

#[derive(Debug)]
struct LinkFile {
    is_relative: bool,
    path: String,
    fragment: Option<String>,
    offset: usize,
}

impl LinkFile {
    fn new(offset: usize, uri: &str) -> Self {
        let is_relative = !uri.starts_with('/');
        match uri.find('#') {
            Some(i) => Self {
                offset,
                is_relative,
                path: uri[..i].to_string(),
                fragment: Some(uri[i + 1..].to_string()),
            },

            None => Self {
                offset,
                is_relative,
                path: uri.to_string(),
                fragment: None,
            },
        }
    }
}

impl Display for LinkFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

fn link_files_from_events((offset, event): (usize, Event)) -> Option<LinkFile> {
    match event {
        Start(Link(string_uri, _)) => {
            if string_uri.find(':').is_none() {
                Some(LinkFile::new(offset, &string_uri))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn broken_link_file(link_file: &LinkFile, parent: &Path) -> bool {
    let path = if link_file.is_relative {
        parent.join(&link_file.path)
    } else {
        PathBuf::from(&link_file.path)
    };
    !path.exists()
}

struct WithOffsetIterator<'a> {
    iter: Parser<'a>,
}

impl<'a> Iterator for WithOffsetIterator<'a> {
    type Item = (usize, Event<'a>);

    fn next(&mut self) -> Option<(usize, Event<'a>)> {
        let offset = self.iter.get_offset(); // next advances, so we get offset first
        self.iter.next().map(|e| (offset, e))
    }
}

trait WithOffset<'a> {
    fn with_offset(self) -> WithOffsetIterator<'a>;
}

impl<'a> WithOffset<'a> for Parser<'a> {
    fn with_offset(self) -> WithOffsetIterator<'a> {
        WithOffsetIterator { iter: self }
    }
}
