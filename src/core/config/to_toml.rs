use crate::core::global_config::GlobalConfig;

trait ToToml {
    fn to_toml(&self) -> String;
}

impl ToToml for GlobalConfig {
    fn to_toml(&self) -> String {
        format!(r#""#,)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::global_config::GlobalConfig;
    use super::*;

    #[test]
    fn when_converting_empty_config_to_toml_it_shall_return_empty_string() {
        let global_config = GlobalConfig::new(vec![], vec![]);
        let toml = global_config.to_toml();
        assert_eq!(toml, r#""#.trim());
    }
}
