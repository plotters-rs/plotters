use rusttype::Error;

#[derive(Debug)]
pub enum FontError {
    LockError,
    NoSuchFont,
    FontLoadError(Error),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        return match self {
            FontError::LockError => write!(fmt, "Could not lock mutex"),
            FontError::NoSuchFont => write!(fmt, "No such font"),
            FontError::FontLoadError(e) => write!(fmt, "Font loading error: {}", e),
        };
    }
}

impl std::error::Error for FontError {}

pub struct FontDataInternal;
