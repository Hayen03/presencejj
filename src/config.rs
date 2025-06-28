pub struct Config {
    pub out_dir: String, 
    pub working_dir: String,
    pub verbose: bool,
    pub excel: ExcelConfig,
    pub typst_working_dir: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            out_dir: "./out".into(),
            verbose: true,
            excel: ExcelConfig::default(),
            working_dir: ".".into(),
            typst_working_dir: "templates".into(),
        }
    }
}
pub struct ExcelConfig {
    pub ln_skip: usize,
    pub data_ln: usize,
}
impl Default for ExcelConfig {
    fn default() -> Self {
        Self {
            ln_skip: 6,
            data_ln: 5,
        }
    }
}