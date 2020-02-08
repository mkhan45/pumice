use std;
use std::error::Error;
use std::fmt;

use lyon;

#[derive(Debug, Clone)]
pub enum PumiceError {
    LyonError(String)
}

impl fmt::Display for PumiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PumiceError::LyonError(ref s) => write!(f, "Lyon error: {}", s)
        }
    }
}

impl Error for PumiceError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl From<lyon::tessellation::TessellationError> for PumiceError {
    fn from(err: lyon::tessellation::TessellationError) -> PumiceError {
        let fmtd = format!("Tesselation error: {:?}", err);
        PumiceError::LyonError(fmtd)
    }
}
