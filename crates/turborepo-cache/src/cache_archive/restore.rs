use std::{
    backtrace::Backtrace,
    fs,
    fs::{File, OpenOptions},
    io::Read,
    path::{Iter, Path, PathBuf},
};

use petgraph::graph::DiGraph;
use tar::{Entry, Header};
use turbopath::{
    AbsoluteSystemPath, AbsoluteSystemPathBuf, AnchoredSystemPathBuf, PathError,
    PathValidationError, RelativeSystemPathBuf,
};

use crate::{
    cache_archive::{
        restore_directory::restore_directory,
        restore_regular::restore_regular,
        restore_symlink::{canonicalize_linkname, restore_symlink},
    },
    CacheError,
};

struct CacheReader {
    path: AbsoluteSystemPathBuf,
    file: File,
    is_compressed: bool,
}

impl CacheReader {
    pub fn open(path: &AbsoluteSystemPathBuf) -> Result<Self, CacheError> {
        let mut options = OpenOptions::new();

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options.mode(0o777);
        }

        #[cfg(windows)]
        {
            use crate::cache_archive::create::FILE_FLAG_SEQUENTIAL_SCAN;
            options.custom_flags(FILE_FLAG_SEQUENTIAL_SCAN);
        }

        let file = options.read(true).open(path.as_path())?;

        Ok(CacheReader {
            path: path.clone(),
            file,
            is_compressed: path.as_path().ends_with(".zst"),
        })
    }

    pub fn restore(
        &self,
        anchor: &AbsoluteSystemPath,
    ) -> Result<Vec<AnchoredSystemPathBuf>, CacheError> {
        let mut restored = Vec::new();
        fs::create_dir_all(anchor.as_path())?;

        // We're going to make the following two assumptions here for "fast"
        // path restoration:
        // - All directories are enumerated in the `tar`.
        // - The contents of the tar are enumerated depth-first.
        //
        // This allows us to avoid:
        // - Attempts at recursive creation of directories.
        // - Repetitive `lstat` on restore of a file.
        //
        // Violating these assumptions won't cause things to break but we're
        // only going to maintain an `lstat` cache for the current tree.
        // If you violate these assumptions and the current cache does
        // not apply for your path, it will clobber and re-start from the common
        // shared prefix.

        if self.is_compressed {
            let zr = zstd::Decoder::new(&self.file)?;
            let mut tr = tar::Archive::new(zr);
            Self::restore_entries(&mut tr, &mut restored, anchor)?;
        } else {
            let mut tr = tar::Archive::new(&self.file);
            Self::restore_entries(&mut tr, &mut restored, anchor)?;
        };

        Ok(restored)
    }

    fn restore_entries<'a, T: Read>(
        tr: &'a mut tar::Archive<T>,
        restored: &mut Vec<AnchoredSystemPathBuf>,
        anchor: &AbsoluteSystemPath,
    ) -> Result<(), CacheError> {
        // On first attempt to restore it's possible that a link target doesn't exist.
        // Save them and topologically sort them.
        let mut symlinks = Vec::new();

        for entry in tr.entries()? {
            let mut entry = entry?;
            match restore_entry(anchor, &mut entry) {
                Err(CacheError::LinkTargetDoesNotExist(_, _)) => {
                    symlinks.push(entry);
                }
                Err(e) => return Err(e),
                Ok(restored_path) => restored.push(restored_path),
            }
        }

        let mut restored_symlinks = Self::topologically_restore_symlinks(anchor, &symlinks)?;
        restored.append(&mut restored_symlinks);
        Ok(())
    }

    fn topologically_restore_symlinks<'a, T: Read>(
        anchor: &AbsoluteSystemPath,
        symlinks: &[Entry<'a, T>],
    ) -> Result<Vec<AnchoredSystemPathBuf>, CacheError> {
        let mut graph = DiGraph::new();

        for entry in symlinks {
            let processed_name = canonicalize_name(&entry.header().path()?)?;
            let processed_sourcename =
                canonicalize_linkname(anchor, &processed_name, processed_name.as_path());
            // symlink must have a linkname
            let linkname = entry
                .header()
                .link_name()?
                .expect("symlink without linkname");

            let processed_linkname = canonicalize_linkname(anchor, &processed_name, &linkname);

            let source_name = graph.add_node(processed_sourcename);
            let link_name = graph.add_node(processed_linkname);
            graph.add_edge(source_name, link_name, ());
        }

        let nodes = petgraph::algo::toposort(&graph, None)
            .map_err(|cycle| CacheError::CycleDetected(Backtrace::capture()))?;

        for node in nodes {
            println!("{:?}", node);
        }

        todo!()
    }
}

fn restore_entry<T: Read>(
    anchor: &AbsoluteSystemPath,
    entry: &mut Entry<T>,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    let header = entry.header();

    match header.entry_type() {
        tar::EntryType::Directory => restore_directory(anchor, entry.header()),
        tar::EntryType::Regular => restore_regular(anchor, entry),
        tar::EntryType::Symlink => restore_symlink(anchor, entry.header()),
        ty => Err(CacheError::UnsupportedFileType(ty, Backtrace::capture())),
    }
}

pub fn canonicalize_name(name: &Path) -> Result<AnchoredSystemPathBuf, CacheError> {
    let PathValidation {
        well_formed,
        windows_safe,
    } = check_name(name);

    if !well_formed {
        return Err(CacheError::MalformedName(
            name.to_string_lossy().to_string(),
            Backtrace::capture(),
        ));
    }

    #[cfg(windows)]
    {
        if !windows_safe {
            return Err(CacheError::WindowsUnsafeName(
                name.to_string(),
                Backtrace::capture(),
            ));
        }
    }

    // There's no easier way to remove trailing slashes in Rust
    // because `OsString`s are really just `Vec<u8>`s.
    let no_trailing_slash: PathBuf = name.components().collect();

    Ok(RelativeSystemPathBuf::new(no_trailing_slash)?.into())
}

struct PathValidation {
    well_formed: bool,
    windows_safe: bool,
}

fn check_name(name: &Path) -> PathValidation {
    if name.as_os_str().is_empty() {
        return PathValidation {
            well_formed: false,
            windows_safe: false,
        };
    }

    let mut well_formed = true;
    let mut windows_safe = true;
    let name = name.to_string_lossy();
    // Name is:
    // - "."
    // - ".."
    if well_formed && name == "." || name == ".." {
        well_formed = false;
    }

    // Name starts with:
    // - `/`
    // - `./`
    // - `../`
    if well_formed && (name.starts_with("/") || name.starts_with("./") || name.starts_with("../")) {
        well_formed = false;
    }

    // Name ends in:
    // - `/.`
    // - `/..`
    if well_formed && (name.ends_with("/.") || name.ends_with("/..")) {
        well_formed = false;
    }

    // Name contains: `\`
    if name.contains('\\') {
        windows_safe = false;
    }

    PathValidation {
        well_formed,
        windows_safe,
    }
}
