// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// External dependencies
extern crate app_dirs;
extern crate bloodhound;
extern crate fragment;
extern crate git2;
extern crate luthor;
extern crate mio;
extern crate pad;
extern crate regex;
extern crate scribe;
extern crate signal_hook;
extern crate syntect;
extern crate unicode_segmentation;
extern crate clipboard;
extern crate yaml_rust as yaml;
extern crate smallvec;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

// Private modules
mod commands;
mod errors;
mod util;
mod input;
mod models;
mod presenters;
mod view;

// External application API
pub use crate::models::Application;
pub use crate::errors::Error;

mod git {
    use std::path::PathBuf;
    use git2::{Repository, DiffOptions, IntoCString};
    use std::collections::hash_map::HashMap;

    pub struct FileData {
        status: FileStatus,
        line_map: HashMap<usize, LineStatus>
    }

    impl FileData {
        pub fn from(repo: &Option<Repository>, path: &Option<PathBuf>) -> Option<Self> {
            let repo = repo.as_ref()?;
            let path = path.as_ref()?;
            let rel_path = path.strip_prefix(repo.workdir()?).ok()?;

            let status = map_status(repo.status_file(rel_path).ok()?);

            let mut opts = DiffOptions::default();
            opts.context_lines(0);
            opts.pathspec(rel_path.to_str()?.into_c_string().ok()?);
            let diff = repo.diff_index_to_workdir(None, Some(&mut opts)).ok()?;

            let mut line_map = HashMap::new();
            // diff.foreach(&mut |_, _| true, None, Some(&mut |_, hunk| {
            //     let new_start = hunk.new_start() as usize;
            //     let new_end = new_start + hunk.new_lines() as usize;
            //     let old_start = hunk.old_start() as usize;
            //     let old_end = old_start + hunk.old_lines() as usize;

            //     for i in new_start..new_end {
            //         let status = if i < old_start || i > old_end {
            //             LineStatus::Added
            //         } else {
            //             LineStatus::Modified
            //         };
            //         line_map.insert(i, status);
            //     } 
            //     true
            // }), None).ok()?;

            diff.foreach(&mut |_, _| true, None, None, Some(&mut |_, _, line| {
                if let Some(line_no) = line.new_lineno() {
                    let status = match line.old_lineno() {
                        Some(_) => LineStatus::Added,
                        None => LineStatus::Modified
                    };
                    line_map.insert(line_no as usize, status);
                }
                true
            })).ok()?;


            Some(FileData {
                status,
                line_map
            })
        }

        pub fn status(&self) -> &FileStatus {
            &self.status
        }

        pub fn status_of_line(&self, line_no: usize) -> Option<&LineStatus> {
            self.line_map.get(&line_no)
        }
    }

    fn map_status(status: git2::Status) -> FileStatus {
        if status.contains(git2::Status::WT_NEW) {
            if status.contains(git2::Status::INDEX_NEW) {
                // Parts of the file are staged as new in the index.
                FileStatus::Partial
            } else {
                // The file has never been added to the repository.
                FileStatus::Untracked
            }
        } else if status.contains(git2::Status::INDEX_NEW) {
            // The complete file is staged as new in the index.
            FileStatus::Staged
        } else if status.contains(git2::Status::WT_MODIFIED) {
            if status.contains(git2::Status::INDEX_MODIFIED) {
                // The file has both staged and unstaged modifications.
                FileStatus::Partial
            } else {
                // The file has unstaged modifications.
                FileStatus::Modified
            }
        } else if status.contains(git2::Status::INDEX_MODIFIED) {
            // The file has staged modifications.
            FileStatus::Staged
        } else {
            // The file is tracked, but has no modifications.
            FileStatus::Ok
        }
    }

    pub enum FileStatus {
        Ok,
        Modified,
        Staged,
        Partial,
        Untracked
    }

    impl std::string::ToString for FileStatus {
        fn to_string(&self) -> String {
            match *self {
                FileStatus::Ok => "ok",
                FileStatus::Modified => "modified",
                FileStatus::Staged => "staged",
                FileStatus::Partial => "partially staged",
                FileStatus::Untracked => "untracked",
            }.into()
        }
    }

    pub enum LineStatus {
        Added,
        Modified
    }
}