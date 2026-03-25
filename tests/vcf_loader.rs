use std::path::PathBuf;

#[test]
fn loads_vcf_fixed_fields_into_dataset() {
    let path = PathBuf::from("tests/fixtures/example.vcf");
    let dataset = ferrolens::data::loader::vcf::load_dataset(&path).unwrap();

    let names = dataset
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"CHROM"));
    assert!(names.contains(&"POS"));
    assert!(names.contains(&"REF"));
    assert!(names.contains(&"ALT"));
    assert_eq!(dataset.rows.len(), 1);
}

#[test]
fn exposes_info_fields_in_detail_metadata() {
    let path = PathBuf::from("tests/fixtures/example.vcf");
    let dataset = ferrolens::data::loader::vcf::load_dataset(&path).unwrap();

    assert!(dataset.metadata.contains_key("info_keys"));
}

#[test]
fn opens_gzipped_vcf() {
    let path = PathBuf::from("tests/fixtures/example.vcf.gz");
    let dataset = ferrolens::data::loader::vcf::load_dataset(&path).unwrap();

    assert_eq!(dataset.rows.len(), 1);
}

#[test]
fn marks_vcf_dataset_kind_for_detail_rendering() {
    let path = PathBuf::from("tests/fixtures/example.vcf");
    let dataset = ferrolens::data::loader::vcf::load_dataset(&path).unwrap();

    assert_eq!(
        dataset.metadata.get("dataset_kind"),
        Some(&"vcf".to_string())
    );
}

#[test]
fn loader_dispatches_vcfgz_files_by_extension() {
    let path = PathBuf::from("tests/fixtures/example.vcf.gz");
    let dataset = ferrolens::data::loader::load_dataset(&path).unwrap();

    assert_eq!(
        dataset.metadata.get("dataset_kind"),
        Some(&"vcf".to_string())
    );
}
