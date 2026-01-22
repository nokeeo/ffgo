use std::path::Path;

pub trait PathStrings {
    fn file_stem_str(&self) -> Option<&str>;
}

impl PathStrings for Path {
    fn file_stem_str(&self) -> Option<&str> {
        return self.file_stem()?.to_str();
    }
}
