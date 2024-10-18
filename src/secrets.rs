use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref SECRETS: HashMap<String, String> = {
        let contents = std::fs::read_to_string("Secrets.toml").unwrap();
        let data: toml::Value = contents.parse().unwrap();
        let mut secrets = HashMap::new();

        secrets.insert("DB_NAME".to_string(), data["DB_NAME"].as_str().unwrap().to_string());
        secrets.insert("DB_PW".to_string(), data["DB_PW"].as_str().unwrap().to_string());
        secrets.insert("DB_PORT".to_string(), data["DB_PORT"].as_str().unwrap().to_string());

        secrets
    };
}
