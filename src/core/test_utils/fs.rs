use std::path::PathBuf;

pub fn create_tmp_dir() -> PathBuf {
    let random_project_name = format!("hibernacli-tests-{}", uuid::Uuid::new_v4());
    let tmp_path = std::env::temp_dir().join(random_project_name);
    std::fs::create_dir_all(&tmp_path).unwrap();
    tmp_path
}
