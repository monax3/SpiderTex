use std::collections::VecDeque;

use camino::{Utf8Path, Utf8PathBuf};

use crate::prelude::*;

pub fn walkdir(start: &Utf8Path) -> impl Iterator<Item = Utf8PathBuf> + 'static {
    fn make_reader(dir: &Utf8Path) -> impl Iterator<Item = camino::Utf8DirEntry> {
        dir.read_dir_utf8()
            .map(|i| i.filter_map(Result::ok))
            .into_iter()
            .flatten()
    }

    let mut dirs: VecDeque<Utf8PathBuf> = VecDeque::new();
    let mut read_dir = make_reader(start);

    std::iter::from_fn(move || {
        'start: loop {
            for entry in &mut read_dir {
                #[allow(clippy::filetype_is_file)]
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        dirs.push_back(entry.path().to_owned());
                        continue 'start;
                    } else if file_type.is_file() {
                        return Some(entry.path().to_owned());
                    }
                }
            }

            if let Some(next_dir) = dirs.pop_front() {
                read_dir = make_reader(&next_dir);
                continue 'start;
            }
            break None;
        }
    })
}

pub struct WalkArgs<'a> {
    args:    Box<dyn Iterator<Item = Utf8PathBuf> + 'a>,
    walkdir: Option<Box<dyn Iterator<Item = Utf8PathBuf> + 'a>>,
}

impl<'a> WalkArgs<'a> {
    #[inline]
    #[must_use]
    pub fn new(args: impl Iterator<Item = Utf8PathBuf> + 'a) -> Self {
        WalkArgs {
            args:    Box::new(args),
            walkdir: None,
        }
    }
}

impl<'a> Iterator for WalkArgs<'a> {
    type Item = Utf8PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(walkdir) = &mut self.walkdir {
            if let Some(next) = walkdir.next() {
                return Some(next);
            }
        }

        self.walkdir = None;

        if let Some(file) = self.args.next() {
            #[cfg(feature = "debug-inputs")]
            event!(TRACE, "Walking through {file} (is_dir: {})", file.is_dir());

            if file.is_dir() {
                self.walkdir = Some(Box::new(walkdir(&file)));
                self.next()
            } else {
                Some(file)
            }
        } else {
            None
        }
    }
}
