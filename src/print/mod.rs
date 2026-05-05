use ::typst as tp;

pub mod typst;

#[derive(Debug)]
pub enum PrintError {
	TempFileError,
	TypstLibError(typst_as_lib::TypstAsLibError),
	PDFError(tp::ecow::EcoVec<tp::diag::SourceDiagnostic>),
	IOError(std::io::Error),
}
impl std::fmt::Display for PrintError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PrintError::TempFileError => write!(f, "Failed to create temporary file"),
			PrintError::TypstLibError(e) => write!(f, "typst-as-lib error: {}", e),
			PrintError::PDFError(e) => write!(f, "PDF error: {:?}", e),
			PrintError::IOError(e) => write!(f, "I/O error: {}", e),
		}
	}
}
impl std::error::Error for PrintError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			PrintError::TempFileError => None,
			PrintError::TypstLibError(e) => Some(e),
			PrintError::PDFError(e) => None,
			PrintError::IOError(e) => Some(e),
		}
	}
}
impl From<typst_as_lib::TypstAsLibError> for PrintError {
	fn from(e: typst_as_lib::TypstAsLibError) -> Self {
		PrintError::TypstLibError(e)
	}
}
impl From<tp::ecow::EcoVec<tp::diag::SourceDiagnostic>> for PrintError {
	fn from(e: tp::ecow::EcoVec<tp::diag::SourceDiagnostic>) -> Self {
		PrintError::PDFError(e)
	}
}
impl From<std::io::Error> for PrintError {
	fn from(e: std::io::Error) -> Self {
		PrintError::IOError(e)
	}
}