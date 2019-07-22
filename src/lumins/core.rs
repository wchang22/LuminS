//! Contains core copy, remove, synchronize functions

use std::io;

use rayon::prelude::*;

use crate::lumins::{file_ops, file_ops::Dir, parse::Flag};
use crate::progress::{self, PROGRESS_BAR};

/// Synchronizes all files, directories, and symlinks in `dest` with `src`
///
/// # Arguments
/// * `src`: Source directory
/// * `dest`: Destination directory
/// * `flags`: set for Flag's
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * `src` is an invalid directory
/// * `dest` is an invalid directory
pub fn synchronize(src: &str, dest: &str, flags: Flag) -> Result<(), io::Error> {
    // Retrieve data from src directory about files, dirs, symlinks
    let src_file_sets = file_ops::get_all_files(&src)?;
    let src_files = src_file_sets.files();
    let src_dirs = src_file_sets.dirs();
    let src_symlinks = src_file_sets.symlinks();

    // Retrieve data from dest directory about files, dirs, symlinks
    let dest_file_sets = file_ops::get_all_files(&dest)?;
    let dest_files = dest_file_sets.files();
    let dest_dirs = dest_file_sets.dirs();
    let dest_symlinks = dest_file_sets.symlinks();

    // Initialize progress bar
    progress::progress_init(
        (src_files.len()
            + src_dirs.len()
            + src_symlinks.len()
            + dest_files.len()
            + dest_dirs.len()
            + dest_symlinks.len()) as u64,
    );

    // Determine whether or not to delete
    let delete = !flags.contains(Flag::NO_DELETE);

    // Delete files and symlinks
    if delete {
        let symlinks_to_delete = dest_symlinks.par_difference(&src_symlinks);
        let files_to_delete = dest_files.par_difference(&src_files);

        file_ops::delete_files(symlinks_to_delete, &dest);
        file_ops::delete_files(files_to_delete, &dest);
    }

    let dirs_to_copy = src_dirs.par_difference(&dest_dirs);
    let symlinks_to_copy = src_symlinks.par_difference(&dest_symlinks);
    let files_to_copy = src_files.par_difference(&dest_files);
    let files_to_compare = src_files.par_intersection(&dest_files);

    file_ops::copy_files(dirs_to_copy, &src, &dest);
    file_ops::copy_files(symlinks_to_copy, &src, &dest);
    file_ops::copy_files(files_to_copy, &src, &dest);
    file_ops::compare_and_copy_files(files_to_compare, &src, &dest, flags);

    // Delete dirs in the correct order
    if delete {
        let dirs_to_delete = dest_dirs.par_difference(&src_dirs);
        let dirs_to_delete: Vec<&file_ops::Dir> = file_ops::sort_files(dirs_to_delete);
        file_ops::delete_files_sequential(dirs_to_delete, &dest);
    }

    Ok(())
}

/// Copies all files, directories, and symlinks in `src` to `dest`
///
/// # Arguments
/// * `src`: Source directory
/// * `dest`: Destination directory
/// * `flags`: set for Flag's
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * `src` is an invalid directory
/// * `dest` is an invalid directory
pub fn copy(src: &str, dest: &str, _flags: Flag) -> Result<(), io::Error> {
    // Retrieve data from src directory about files, dirs, symlinks
    let src_file_sets = file_ops::get_all_files(&src)?;
    let src_files = src_file_sets.files();
    let src_dirs = src_file_sets.dirs();
    let src_symlinks = src_file_sets.symlinks();

    // Initialize progress bar
    progress::progress_init((src_files.len() + src_dirs.len() + src_symlinks.len()) as u64);

    // Copy everything
    file_ops::copy_files(src_dirs.into_par_iter(), &src, &dest);
    file_ops::copy_files(src_files.into_par_iter(), &src, &dest);
    file_ops::copy_files(src_symlinks.into_par_iter(), &src, &dest);

    Ok(())
}

/// Deletes directory `target`
///
/// # Arguments
/// * `target`: Target directory
/// * `flags`: set for Flag's
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * `target` is an invalid directory
pub fn remove(target: &str, _flags: Flag) -> Result<(), io::Error> {
    // Retrieve data from target directory about files, dirs, symlinks
    let target_file_sets = file_ops::get_all_files(&target)?;
    let target_files = target_file_sets.files();
    let target_dirs = target_file_sets.dirs();
    let target_symlinks = target_file_sets.symlinks();

    // Initialize progress bar
    progress::progress_init(
        (target_files.len() + target_dirs.len() + target_symlinks.len()) as u64,
    );
    PROGRESS_BAR.enable_steady_tick(1);

    // Delete everything
    file_ops::delete_files(target_files.into_par_iter(), &target);
    file_ops::delete_files(target_symlinks.into_par_iter(), &target);

    // Directories must always be deleted sequentially so that they are deleted in the correct order
    let mut target_dirs: Vec<&file_ops::Dir> = file_ops::sort_files(target_dirs.into_par_iter());

    // Delete the target directory last
    let root_dir = Dir::from("");
    target_dirs.push(&root_dir);

    file_ops::delete_files_sequential(target_dirs.into_iter(), &target);

    Ok(())
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test_synchronize {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[cfg(debug_assertions)]
    const BUILD_DIR: &str = "target/debug";

    #[cfg(not(debug_assertions))]
    const BUILD_DIR: &str = "target/release";

    #[test]
    fn invalid_src() {
        assert_eq!(synchronize("/?", "src", Flag::empty()).is_err(), true);
    }

    #[test]
    fn invalid_dest() {
        assert_eq!(synchronize("src", "/?", Flag::empty()).is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir_1() {
        const TEST_DIR: &str = "test_synchronize_dir1";
        fs::create_dir_all(TEST_DIR).unwrap();

        assert_eq!(synchronize("src", TEST_DIR, Flag::empty()).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", "src", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir_2() {
        const TEST_DIR: &str = "test_synchronize_dir2";
        fs::create_dir_all(TEST_DIR).unwrap();

        assert_eq!(
            synchronize(BUILD_DIR, TEST_DIR, Flag::empty()).is_ok(),
            true
        );

        let diff = Command::new("diff")
            .args(&["-r", BUILD_DIR, TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::File::create([BUILD_DIR, "file.txt"].join("/")).unwrap();
        fs::remove_dir_all([BUILD_DIR, "build"].join("/")).unwrap();

        let diff = Command::new("diff")
            .args(&["-r", BUILD_DIR, TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), false);

        assert_eq!(
            synchronize(BUILD_DIR, TEST_DIR, Flag::empty()).is_ok(),
            true
        );

        let diff = Command::new("diff")
            .args(&["-r", BUILD_DIR, TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn change_symlink() {
        use std::os::unix::fs::symlink;

        const TEST_SRC: &str = "test_synchronize_change_symlink_src";
        const TEST_DEST: &str = "test_synchronize_change_symlink_dest";
        fs::create_dir_all(TEST_SRC).unwrap();
        fs::create_dir_all(TEST_DEST).unwrap();

        symlink("../Cargo.lock", [TEST_SRC, "file"].join("/")).unwrap();
        symlink("../Cargo.toml", [TEST_DEST, "file"].join("/")).unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SRC, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), false);

        assert_eq!(
            synchronize(TEST_SRC, TEST_DEST, Flag::empty()).is_ok(),
            true
        );

        let diff = Command::new("diff")
            .args(&["-r", TEST_SRC, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
        fs::remove_dir_all(TEST_SRC).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn flags() {
        const TEST_DIR: &str = "test_synchronize_flags";
        const TEST_DIR_OUT: &str = "test_synchronize_flags_out";
        const TEST_DIR_EXPECTED: &str = "test_synchronize_flags_expected";
        const TEST_FILES: [&str; 2] = ["file1.txt", "file2.txt"];

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_OUT).unwrap();
        fs::create_dir_all(TEST_DIR_EXPECTED).unwrap();

        fs::File::create([TEST_DIR, TEST_FILES[0]].join("/")).unwrap();
        fs::File::create([TEST_DIR_EXPECTED, TEST_FILES[0]].join("/")).unwrap();
        fs::File::create([TEST_DIR_EXPECTED, TEST_FILES[1]].join("/")).unwrap();

        assert_eq!(
            synchronize(TEST_DIR, TEST_DIR_OUT, Flag::empty()).is_ok(),
            true
        );

        fs::File::create([TEST_DIR, TEST_FILES[1]].join("/")).unwrap();

        let mut flags = Flag::empty();
        flags.insert(Flag::VERBOSE);
        flags.insert(Flag::NO_DELETE);
        flags.insert(Flag::SECURE);
        flags.insert(Flag::SEQUENTIAL);

        assert_eq!(synchronize(TEST_DIR, TEST_DIR_OUT, flags).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", TEST_DIR_OUT, TEST_DIR_EXPECTED])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
        fs::remove_dir_all(TEST_DIR_EXPECTED).unwrap();
    }
}

#[cfg(test)]
mod test_copy {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn invalid_src() {
        assert_eq!(copy("/?", "src", Flag::empty()).is_err(), true);
    }

    #[test]
    fn invalid_dest() {
        const TEST_DIR: &str = "test_copy_invalid_dest";
        assert_eq!(copy("src", TEST_DIR, Flag::empty()).is_ok(), true);
        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir1() {
        const TEST_DIR: &str = "test_copy_dir1";
        fs::create_dir_all(TEST_DIR).unwrap();

        assert_eq!(copy("src", TEST_DIR, Flag::empty()).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", "src", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn flags() {
        const TEST_DIR: &str = "test_copy_flags";
        fs::create_dir_all(TEST_DIR).unwrap();

        let mut flags = Flag::empty();
        flags.insert(Flag::SEQUENTIAL);

        assert_eq!(copy("src", TEST_DIR, flags).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", "src", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DIR).unwrap();
    }
}

#[cfg(test)]
mod test_remove {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[cfg(debug_assertions)]
    const BUILD_DIR: &str = "target/debug";

    #[cfg(not(debug_assertions))]
    const BUILD_DIR: &str = "target/release";

    #[test]
    fn invalid_target() {
        assert_eq!(remove("/?", Flag::empty()).is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir1() {
        const TEST_DIR: &str = "test_remove_dir1";
        fs::create_dir_all(TEST_DIR).unwrap();

        Command::new("cp")
            .args(&["-r", BUILD_DIR, TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(remove(TEST_DIR, Flag::empty()).is_ok(), true);

        assert_eq!(fs::read_dir(TEST_DIR).is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn flags() {
        const TEST_DIR: &str = "test_remove_flags";
        fs::create_dir_all(TEST_DIR).unwrap();

        let mut flags = Flag::empty();
        flags.insert(Flag::SEQUENTIAL);

        Command::new("cp")
            .args(&["-r", "src", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(remove(TEST_DIR, flags).is_ok(), true);

        assert_eq!(fs::read_dir(TEST_DIR).is_err(), true);
    }
}
