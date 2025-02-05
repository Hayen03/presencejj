pub mod typst;

#[derive(Debug, Clone, Copy)]
pub enum PrintError {
	TempFileError,
}