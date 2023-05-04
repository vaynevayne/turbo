use std::{fs::OpenOptions, io, io::Read, path::Path};

use turbopath::{
    AbsoluteSystemPath, AbsoluteSystemPathBuf, AnchoredSystemPath, AnchoredSystemPathBuf,
};

use crate::{
    cache_archive::{restore::canonicalize_name, restore_directory::safe_mkdir_all},
    CacheError,
};

fn restore_regular(
    anchor: &AbsoluteSystemPath,
    header: &tar::Header,
    mut reader: impl Read,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    // Assuming this was a `turbo`-created input, we currently have an
    // AnchoredUnixPath. Assuming this is malicious input we don't really care
    // if we do the wrong thing.
    let processed_name = canonicalize_name(&header.path()?)?;

    // We need to traverse `processedName` from base to root split at
    // `os.Separator` to make sure we don't end up following a symlink
    // outside of the restore path.
    safe_mkdir_file(anchor, processed_name.as_anchored_path(), header.mode()?)?;

    let resolved_path = anchor.resolve(&processed_name);
    let mut open_options = OpenOptions::new();
    open_options.write(true).truncate(true).create(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        open_options.mode(header.mode()?);
    }

    let mut file = open_options.open(resolved_path.as_path())?;
    io::copy(&mut reader, &mut file)?;

    Ok(processed_name)
}

pub fn safe_mkdir_file(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPath,
    mode: u32,
) -> Result<(), CacheError> {
    let is_root_file = processed_name.as_path().parent() == Some(Path::new("."));
    if !is_root_file {
        let dir = processed_name.parent().unwrap();
        safe_mkdir_all(anchor, dir, 0o755)?;
    }

    Ok(())
}
