use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
struct User {
    id: u32,
    name: String,
    spending_limit_daily: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Project {
    user_ids: Vec<u32>,
    id: u32,
    name: String,
    spending_limit_daily: Option<f64>,
}
#[derive(Debug, Deserialize)]
struct ApiKey {
    id: u32,
    key: String,
    project_id: u32,
    user_id: Option<u32>,
    providers: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderId {
    Anthropic,
    Openai,
    Groq,
    GoogleVertex,
}

#[derive(Debug, Deserialize)]
struct ProviderConfig {
    name: String,
    provider_id: ProviderId,
    base_url: String,
    inject_cost: bool,
    credentials_env: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    projects: Vec<Project>,
    providers: Vec<ProviderConfig>,
    api_keys: Vec<ApiKey>,
    users: Vec<User>,
} // top level config for the entire gateway

impl Config {
    pub fn from_toml(s: &str) -> Config {
        toml::from_str(s).expect("bad config")
    }
    pub fn load(path: &str) -> Config {
        let s = std::fs::read_to_string(path).expect("bad file");
        Config::from_toml(&s)
    }
    pub fn find_key(&self, key: &str) -> Option<&ApiKey> {
        // for k in &self.api_keys {
        //     if key == k.key {
        //         return Some(k);
        //     }
        // }
        self.api_keys.iter().find(|k| k.key == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builds_by_hand() {
        /* build a Config with 1 of each; assert api_keys.len() == 1 */

        // Define Users
        let sri = User {
            id: 0,
            name: "sri".to_string(),
            spending_limit_daily: None,
        };
        let tracy = User {
            id: 1,
            name: "tracy".to_string(),
            spending_limit_daily: Some(1.0),
        };

        // Define Projects
        let cpb_project = Project {
            user_ids: vec![sri.id, tracy.id],
            id: 0,
            name: "cpb".to_string(),
            spending_limit_daily: None,
        };

        // Define Providers
        let anthropic_config = ProviderConfig {
            name: "anthropic".to_string(),
            provider_id: ProviderId::Anthropic,
            base_url: "https://helloworld.com".to_string(),
            inject_cost: false,
            credentials_env: "abc123".to_string(),
        };

        //Define API keys
        let cpb_api_key = ApiKey {
            id: 0,
            key: "abc123".to_string(),
            project_id: cpb_project.id,
            user_id: Some(0),
            providers: vec![anthropic_config.name.clone()],
        };
        let config = Config {
            users: vec![sri, tracy],
            projects: vec![cpb_project],
            providers: vec![anthropic_config],
            api_keys: vec![cpb_api_key],
        };
        assert_eq!(config.api_keys.len(), 1);
    }
    #[test]
    fn parses_toml() {
        let cfg = Config::load("config.example.toml");

        // one assert per section, e.g.
        assert_eq!(cfg.users[1].name, "tracy");
        assert_eq!(cfg.projects[0].name, "cpb");
        assert_eq!(cfg.providers[0].name, "anthropic");
        assert_eq!(cfg.api_keys[0].providers[0], "anthropic");
    }
    #[test]
    fn find_key() {
        let cfg = Config::load("config.example.toml");
        let a = cfg.find_key("sk-gw-test-abc123").unwrap();
        assert_eq!(a.id, 0);
        assert!(cfg.find_key("sk-gw-nope").is_none());
    }

    #[test]
    fn borrow_ends_before_push() {
        let mut cfg = Config::load("config.example.toml");
        let k = cfg.find_key("sk-gw-test-abc123").unwrap();
        assert_eq!(k.id, 0);
        let b_key = ApiKey {
            id: 5,
            key: "abc234".to_string(),
            project_id: 5,
            user_id: None,
            providers: vec!["anthropic".to_string()],
        };
        cfg.api_keys.push(b_key);
    }
}
