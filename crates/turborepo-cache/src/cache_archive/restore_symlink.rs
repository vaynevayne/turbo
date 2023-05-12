use std::{
    backtrace::Backtrace,
    path::{Path, PathBuf},
};

use path_clean::clean;
use turbopath::{
    AbsoluteSystemPath, AbsoluteSystemPathBuf, AnchoredSystemPath, AnchoredSystemPathBuf,
    RelativeSystemPathBuf,
};

use crate::{
    cache_archive::{restore::canonicalize_name, restore_regular::safe_mkdir_file},
    CacheError,
};

pub fn restore_symlink(
    anchor: &AbsoluteSystemPath,
    header: &tar::Header,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    let processed_name = canonicalize_name(&header.path()?)?;

    let processed_linkname = canonicalize_linkname(
        anchor,
        &processed_name,
        &header.link_name()?.expect("has linkname"),
    )?;
    if !processed_linkname.exists() {
        return Err(CacheError::LinkTargetDoesNotExist(
            processed_linkname.to_string_lossy().to_string(),
            Backtrace::capture(),
        ));
    }

    actually_restore_symlink(anchor, processed_name.as_anchored_path(), header)?;

    Ok(processed_name)
}

fn restore_symlink_with_missing_target(
    anchor: &AbsoluteSystemPath,
    header: &tar::Header,
) -> Result<AnchoredSystemPathBuf, CacheError> {
    let processed_name = canonicalize_name(&header.path()?)?;

    actually_restore_symlink(anchor, processed_name.as_anchored_path(), header)?;

    Ok(processed_name)
}

fn actually_restore_symlink<'a>(
    anchor: &AbsoluteSystemPath,
    processed_name: &'a AnchoredSystemPath,
    header: &tar::Header,
) -> Result<&'a AnchoredSystemPath, CacheError> {
    safe_mkdir_file(anchor, &processed_name)?;

    let symlink_from = anchor.resolve(processed_name);

    _ = symlink_from.remove();

    let symlink_to = header.link_name()?.expect("have linkname");

    if symlink_to.is_dir() {
        symlink_from.symlink_to_file(symlink_to)?;
    } else {
        symlink_from.symlink_to_dir(symlink_to)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = symlink_from.as_absolute_path().symlink_metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(header.mode()?);
    }

    Ok(processed_name)
}

// canonicalizeLinkname determines (lexically) what the resolved path on the
// system will be when linkname is restored verbatim.
pub fn canonicalize_linkname(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPathBuf,
    linkname: &Path,
) -> Result<PathBuf, CacheError> {
    // We don't know _anything_ about linkname. It could be any of:
    //
    // - Absolute Unix Path
    // - Absolute Windows Path
    // - Relative Unix Path
    // - Relative Windows Path
    //
    // We also can't _truly_ distinguish if the path is Unix or Windows.
    // Take for example: `/Users/turbobot/weird-filenames/\foo\/lol`
    // It is a valid file on Unix, but if we do slash conversion it breaks.
    // Or `i\am\a\normal\unix\file\but\super\nested\on\windows`.
    //
    // We also can't safely assume that paths in link targets on one platform
    // should be treated as targets for that platform. The author may be
    // generating an artifact that should work on Windows on a Unix device.
    //
    // Given all of that, our best option is to restore link targets _verbatim_.
    // No modification, no slash conversion.
    //
    // In order to DAG sort them, however, we do need to canonicalize them.
    // We canonicalize them as if we're restoring them verbatim.
    //
    let cleaned_linkname = clean(linkname);

    // 1. Check to see if the link target is absolute _on the current platform_.
    // If it is an absolute path it's canonical by rule.
    if cleaned_linkname.is_absolute() {
        return Ok(cleaned_linkname);
    }

    let cleaned_linkname = RelativeSystemPathBuf::new(cleaned_linkname)?;
    // Remaining options:
    // - Absolute (other platform) Path
    // - Relative Unix Path
    // - Relative Windows Path
    //
    // At this point we simply assume that it's a relative path—no matter
    // which separators appear in it and where they appear,  We can't do
    // anything else because the OS will also treat it like that when it is
    // a link target.
    //
    let source = anchor.resolve(processed_name);
    let canonicalized = source
        .parent()
        .unwrap_or(anchor)
        .join_relative(&cleaned_linkname);

    Ok(clean(canonicalized))
}
