use strum_macros::Display;

#[derive(Display)]
pub enum BitFileType {
    Dir,
    Regular,
}

impl BitFileType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dir => "tree",
            Self::Regular => "blob",
        }
    }
}
