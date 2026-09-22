use sdl3::video::WindowBuildError;
use sdl3::IntegerOrSdlError;
use std::error::Error;
use std::fmt;

/// A failure while creating the SDL platform and configuring its display.
#[derive(Debug)]
pub enum InitError {
    Sdl(sdl3::Error),
    Window(WindowBuildError),
    LogicalSize(IntegerOrSdlError),
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdl(error) => error.fmt(f),
            Self::Window(error) => error.fmt(f),
            Self::LogicalSize(error) => error.fmt(f),
        }
    }
}

impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sdl(error) => Some(error),
            Self::Window(error) => Some(error),
            Self::LogicalSize(error) => Some(error),
        }
    }
}

impl From<sdl3::Error> for InitError {
    fn from(error: sdl3::Error) -> Self {
        Self::Sdl(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_initialization_failure_retains_its_concrete_cause() {
        let error = InitError::Window(WindowBuildError::WidthOverflows(u32::MAX));
        let cause = error.source().expect("window error");

        assert!(matches!(
            cause.downcast_ref::<WindowBuildError>(),
            Some(WindowBuildError::WidthOverflows(u32::MAX))
        ));
        assert_eq!(error.to_string(), cause.to_string());
    }
}
