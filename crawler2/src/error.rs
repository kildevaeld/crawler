use conveyor::ConveyorError;
use std::error::Error;
use std::fmt;
use std::result::Result;

pub type CrawlResult<T> = Result<T, CrawlError>;

#[derive(Debug)]
pub enum CrawlErrorKind {
    Unknown,
    Conveyor(String),
    Error(Box<dyn Error + Send + Sync>),
    NotFound(String),
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct CrawlError {
    kind: CrawlErrorKind,
}

impl CrawlError {
    pub fn new(kind: CrawlErrorKind) -> CrawlError {
        CrawlError { kind }
    }
}

impl fmt::Display for CrawlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CrawlError<")?;
        match &self.kind {
            CrawlErrorKind::Conveyor(s) => write!(f, "Conveyor({})", s),
            CrawlErrorKind::NotFound(s) => write!(f, "NotFound({})", s),
            _ => write!(f, "Unknown"),
        }?;
        write!(f, ">")
    }
}

impl Error for CrawlError {}

impl From<CrawlErrorKind> for CrawlError {
    fn from(error: CrawlErrorKind) -> CrawlError {
        CrawlError::new(error)
    }
}

impl From<ConveyorError> for CrawlError {
    fn from(error: ConveyorError) -> CrawlError {
        CrawlError::new(CrawlErrorKind::Conveyor(error.to_string()))
    }
}

impl From<std::io::Error> for CrawlError {
    fn from(error: std::io::Error) -> CrawlError {
        CrawlError::new(CrawlErrorKind::Io(error))
    }
}
