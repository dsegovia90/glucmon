use derive_more::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    // -- Externals
    #[from]
    Io(std::io::Error),
    #[from]
    Image(image::ImageError),
}

impl Error {
    pub fn custom(val: impl std::fmt::Display) -> Self {
        Self::Custom(val.to_string())
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Custom(val.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
