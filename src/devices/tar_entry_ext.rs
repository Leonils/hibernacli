use std::{
    fs::{self, File},
    io::{self, Error, ErrorKind},
    path::{Component, Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::Entry;

pub trait UnpackFileIn {
    fn unpack_file_in(&mut self, dst: &Path) -> io::Result<bool>;
    fn ensure_dir_created(&self, dst: &Path, dir: &Path) -> io::Result<()>;
    fn validate_inside_dst(&self, dst: &Path, file_dst: &Path) -> io::Result<PathBuf>;
}

impl UnpackFileIn for Entry<'_, GzDecoder<File>> {
    fn unpack_file_in(&mut self, dst: &Path) -> io::Result<bool> {
        // Notes regarding bsdtar 2.8.3 / libarchive 2.8.3:
        // * Leading '/'s are trimmed. For example, `///test` is treated as
        //   `test`.
        // * If the filename contains '..', then the file is skipped when
        //   extracting the tarball.
        // * '//' within a filename is effectively skipped. An error is
        //   logged, but otherwise the effect is as if any two or more
        //   adjacent '/'s within the filename were consolidated into one
        //   '/'.
        //
        // Most of this is handled by the `path` module of the standard
        // library, but we specially handle a few cases here as well.

        let mut file_dst = dst.to_path_buf();
        {
            let path = self.path().unwrap();
            let mut starts_with_files = false;

            for part in path.components() {
                match part {
                    // Leading '/' characters, root paths, and '.'
                    // components are just ignored and treated as "empty
                    // components"
                    Component::Prefix(..) | Component::RootDir | Component::CurDir => continue,

                    // If any part of the filename is '..', then skip over
                    // unpacking the file to prevent directory traversal
                    // security issues.  See, e.g.: CVE-2001-1267,
                    // CVE-2002-0399, CVE-2005-1918, CVE-2007-4131
                    Component::ParentDir => return Ok(false),

                    Component::Normal(part) if part == ".files" => {
                        starts_with_files = true;
                        continue;
                    }
                    Component::Normal(part) => file_dst.push(part),
                }
            }

            // Skip entries outside of the .files directory
            if !starts_with_files {
                return Ok(false);
            }
        }

        // Skip cases where only slashes or '.' parts were seen, because
        // this is effectively an empty filename.
        if *dst == *file_dst {
            return Ok(true);
        }

        // Skip entries without a parent (i.e. outside of FS root)
        let parent = match file_dst.parent() {
            Some(p) => p,
            None => return Ok(false),
        };

        self.ensure_dir_created(&dst, parent).unwrap();

        self.unpack(&file_dst).unwrap();

        Ok(true)
    }

    fn ensure_dir_created(&self, dst: &Path, dir: &Path) -> io::Result<()> {
        let mut ancestor = dir;
        let mut dirs_to_create = Vec::new();
        while ancestor.symlink_metadata().is_err() {
            dirs_to_create.push(ancestor);
            if let Some(parent) = ancestor.parent() {
                ancestor = parent;
            } else {
                break;
            }
        }
        for ancestor in dirs_to_create.into_iter().rev() {
            if let Some(parent) = ancestor.parent() {
                self.validate_inside_dst(dst, parent)?;
            }
            fs::create_dir_all(ancestor)?;
        }
        Ok(())
    }

    fn validate_inside_dst(&self, dst: &Path, file_dst: &Path) -> io::Result<PathBuf> {
        // Abort if target (canonical) parent is outside of `dst`
        let canon_parent = file_dst.canonicalize().map_err(|err| {
            Error::new(
                err.kind(),
                format!("{} while canonicalizing {}", err, file_dst.display()),
            )
        })?;
        let canon_target = dst.canonicalize().map_err(|err| {
            Error::new(
                err.kind(),
                format!("{} while canonicalizing {}", err, dst.display()),
            )
        })?;
        if !canon_parent.starts_with(&canon_target) {
            let err = Error::new(ErrorKind::Other, "Invalid argument");
            return Err(err);
        }
        Ok(canon_target)
    }
}
