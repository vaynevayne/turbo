use path_clean::clean;
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

// canonicalizeLinkname determines (lexically) what the resolved path on the
// system will be when linkname is restored verbatim.
pub fn canonicalize_linkname(
    anchor: &AbsoluteSystemPath,
    processed_name: &AnchoredSystemPathBuf,
    linkname: &str,
) {
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
        return;
    }
}
