use std::fs::File;
use std::io::Read;
use super::FetchData;

pub struct FileSource<'a> {
    filename: &'a str,
}

impl<'a> FileSource<'a> {
    pub fn new(filename: &'a str) -> FileSource {
        FileSource {
            filename: filename,
        }
    }
}

impl<'a> FetchData for FileSource<'a> {
    fn data(&self) -> String {
        let mut f = File::open(self.filename).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).expect("Cannot read file");
        buf
    }
}
