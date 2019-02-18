use std::io;

use crate::lumins::file_ops;

/// Synchronizes all files, directories, and symlinks in `dest` with `src`
///
/// # Arguments
/// * `src`: Source directory
/// * `dest`: Destination directory
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * `src` is an invalid directory
/// * `dest` is an invalid directory
pub fn synchronize(src: &str, dest: &str) -> Result<(), io::Error> {
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

    // Figure out the differences between src and dest directories
    let dirs_to_delete = dest_dirs.par_difference(&src_dirs);
    let dirs_to_copy = src_dirs.par_difference(&dest_dirs);

    // Copy any new directories over
    file_ops::copy_files(dirs_to_copy, &src, &dest);

    // Figure out the differences between src and dest symlinks
    let symlinks_to_delete = dest_symlinks.par_difference(&src_symlinks);
    let symlinks_to_copy = src_symlinks.par_difference(&dest_symlinks);

    // Copy new and delete old symlinks
    file_ops::delete_files(symlinks_to_delete, &dest);
    file_ops::copy_files(symlinks_to_copy, &src, &dest);

    // Figure out the differences between src and dest files
    let files_to_delete = dest_files.par_difference(&src_files);
    let files_to_copy = src_files.par_difference(&dest_files);
    let files_to_compare = src_files.par_intersection(&dest_files);

    // Copy new and delete old files
    file_ops::delete_files(files_to_delete, &dest);
    file_ops::copy_files(files_to_copy, &src, &dest);
    file_ops::compare_and_copy_files(files_to_compare, &src, &dest);

    // Delete the old directories in the correct order to prevent conflicts
    let dirs_to_delete: Vec<&file_ops::Dir> = file_ops::sort_files(dirs_to_delete);
    file_ops::delete_files_sequential(dirs_to_delete, &dest);

    Ok(())
}

#[cfg(test)]
mod test_synchronize {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn invalid_src() {
        assert_eq!(synchronize("/?", "src").is_err(), true);
    }

    #[test]
    fn invalid_dest() {
        assert_eq!(synchronize("src", "/?").is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir_1() {
        const TEST_DIR: &str = "test_synchronize_dir1";
        fs::create_dir_all(TEST_DIR).unwrap();

        assert_eq!(synchronize("src", TEST_DIR).is_ok(), true);

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

        assert_eq!(synchronize("target/debug", TEST_DIR).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", "target/debug", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::File::create("target/debug/file.txt").unwrap();
        fs::remove_dir_all("target/debug/build").unwrap();

        let diff = Command::new("diff")
            .args(&["-r", "target/debug", TEST_DIR])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), false);

        assert_eq!(synchronize("target/debug", TEST_DIR).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", "target/debug", TEST_DIR])
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

        assert_eq!(synchronize(TEST_SRC, TEST_DEST).is_ok(), true);

        let diff = Command::new("diff")
            .args(&["-r", TEST_SRC, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
        fs::remove_dir_all(TEST_SRC).unwrap();
    }
}
