pub mod delimited;
pub mod vcf;

use std::path::Path;

use crate::data::types::Dataset;
use crate::error::Result;

pub fn load_dataset(path: &Path) -> Result<Dataset> {
    let path_str = path.to_string_lossy();

    if path_str.ends_with(".vcf") || path_str.ends_with(".vcf.gz") {
        vcf::load_dataset(path)
    } else {
        delimited::load_dataset(path)
    }
}
