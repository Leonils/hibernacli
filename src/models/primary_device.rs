pub trait PrimaryDevice {
    fn read_file(&self, file_name: &str) -> String;
    fn write_file(&self, file_name: &str, content: &str);
}
