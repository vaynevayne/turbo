use turbopath::{
    AbsoluteSystemPath, AbsoluteSystemPathBuf, AnchoredSystemPath, AnchoredSystemPathBuf,
};

use crate::{
    cache_archive::{restore::canonicalize_name, restore_regular::safe_mkdir_file},
    CacheError,
};

fn restore_symlink_with_missing_target(
    anchor: &AbsoluteSystemPath,
    header: &tar::Header,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    let processed_name = canonicalize_name(&header.path()?)?;

    actually_restore_symlink(anchor, processed_name.as_anchored_path(), header)
}

fn actually_restore_symlink(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPath,
    header: &tar::Header,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    safe_mkdir_file(anchor, &processed_name, header.mode()?)?;
    todo!()
}

pub fn canonicalize_linkname(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPathBuf,
    linkname: &str,
) -> String {
    todo!()
}
