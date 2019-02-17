use std::io;

use crate::lumins::file_ops;

pub fn synchronize(src: &str, dest: &str) -> Result<(), io::Error> {
    let src_file_sets = file_ops::get_all_files(&src)?;
    let src_files = src_file_sets.files();
    let src_dirs = src_file_sets.dirs();

    let dest_file_sets = file_ops::get_all_files(&dest)?;
    let dest_files = dest_file_sets.files();
    let dest_dirs = dest_file_sets.dirs();

    let dirs_to_delete = dest_dirs.par_difference(&src_dirs);
    let dirs_to_copy = src_dirs.par_difference(&dest_dirs);

    file_ops::copy_files(dirs_to_copy, &src, &dest);

    let files_to_delete = dest_files.par_difference(&src_files);
    let files_to_copy = src_files.par_difference(&dest_files);
    let files_to_compare = src_files.par_intersection(&dest_files);

    file_ops::delete_files(files_to_delete, &dest);
    file_ops::copy_files(files_to_copy, &src, &dest);
    file_ops::compare_and_copy_files(files_to_compare, &src, &dest);

    let dirs_to_delete: Vec<&file_ops::Dir> = file_ops::sort_files(dirs_to_delete);

    file_ops::delete_files_sequential(dirs_to_delete, &dest);

    Ok(())
}
