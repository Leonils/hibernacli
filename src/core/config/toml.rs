use toml::Table;

pub trait TryRead<'a, T> {
    fn try_read(&'a self, key: &'a str) -> Result<T, String>;
}

impl<'a> TryRead<'a, &'a str> for &'a Table {
    fn try_read(&'a self, key: &'a str) -> Result<&'a str, String> {
        self.get(key)
            .ok_or_else(|| format!("Missing {} field", key))?
            .as_str()
            .ok_or_else(|| format!("Invalid string for {}", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn test_try_read_str() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::String("value".to_string()));
        let table = &table;
        let v: &str = table.try_read("key").unwrap();
        assert_eq!(v, "value");
    }

    #[test]
    fn test_try_read_str_missing() {
        let table = &Table::new();
        let v: Result<&str, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Missing key field");
    }

    #[test]
    fn test_try_read_str_invalid() {
        let mut table = Table::new();
        table.insert("key".to_string(), Value::Integer(42));
        let table = &table;
        let v: Result<&str, _> = table.try_read("key");
        assert_eq!(v.unwrap_err(), "Invalid string for key");
    }
}
