use std::{
    backtrace::Backtrace,
    fs,
    path::{Component, Components},
};

use tar::Header;
use turbopath::{
    AbsoluteSystemPath, AbsoluteSystemPathBuf, AnchoredSystemPath, AnchoredSystemPathBuf,
    RelativeSystemPathBuf,
};

use crate::{cache_archive::restore::canonicalize_name, CacheError};

pub fn restore_directory(
    anchor: &AbsoluteSystemPath,
    header: &Header,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    let processed_name = canonicalize_name(&header.path()?)?;

    safe_mkdir_all(anchor, processed_name.as_anchored_path(), header.mode()?)?;

    Ok(processed_name)
}

pub fn safe_mkdir_all(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPath,
    mode: u32,
) -> Result<(), CacheError> {
    // Iterate through path segments by os.Separator, appending them onto
    // current_path. Check to see if that path segment is a symlink
    // with a target outside of anchor.
    let mut current_path: AnchoredSystemPathBuf = RelativeSystemPathBuf::default().into();
    for component in processed_name.as_path().components() {
        check_path(anchor, current_path.as_anchored_path())?;
        current_path.push(component);
    }

    // If we have made it here we know that it is safe to call fs::create_dir_all
    // on the join of anchor and processed_name.
    //
    // This could _still_ error, but we don't care.
    let resolved_name = anchor.resolve(processed_name);
    fs::create_dir_all(&resolved_name)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(&resolved_name)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode);
        fs::set_permissions(&resolved_name, permissions)?;
    }

    Ok(())
}

fn check_path(
    anchor: &AbsoluteSystemPath,
    path: &AnchoredSystemPath,
) -> Result<AbsoluteSystemPathBuf, CacheError> {
    let resolved_path = anchor.resolve(path);
    // Getting an error here means we failed to stat the path.
    // Assume that means we're safe and continue.
    let Ok(file_info) = fs::symlink_metadata(resolved_path.as_path()) else {
            return Ok(resolved_path);
        };

    // If we don't have a symlink, it's safe
    if !file_info.is_symlink() {
        return Ok(resolved_path);
    }

    // Check to see if the symlink targets outside of the originalAnchor.
    // We don't do eval symlinks because we could find ourself in a totally
    // different place.

    // 1. Get the target.
    let link_target = fs::read_link(resolved_path.as_path())?;

    if link_target.is_absolute() {
        let absolute_link_target = AbsoluteSystemPathBuf::new(link_target.clone())?;
        if absolute_link_target.as_path().starts_with(&anchor) {
            return Ok(resolved_path);
        }
    } else {
        let relative_link_target = RelativeSystemPathBuf::new(link_target.clone())?;
        let computed_target = anchor.join_relative(&relative_link_target);
        if computed_target.as_path().starts_with(&anchor) {
            let anchored_link_target: AnchoredSystemPathBuf = relative_link_target.into();
            return check_path(anchor, anchored_link_target.as_anchored_path());
        }
    }

    Err(CacheError::LinkOutsideOfDirectory(
        link_target.to_string_lossy().to_string(),
        Backtrace::capture(),
    ))
}
