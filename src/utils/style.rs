use std::collections::HashMap;

use web_sys::Element;

pub struct Style {
    pub properties: HashMap<String, String>,
}

impl Style {
    pub fn new(value: &str) -> Style {
        let mut properties = HashMap::new();
        for item in value.split(";").map(|s| {
            if !s.is_empty() {
                let split: Vec<&str> = s.split(":").collect();
                Some((split[0].to_string(), split[1].to_string()))
            } else {
                None
            }
        }) {
            if item.is_some() {
                let (key, value) = item.unwrap();
                properties.insert(key, value);
            }
        }
        Style { properties }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.properties
            .insert(String::from(key), String::from(value));
    }

    pub fn remove(&mut self, key: &str) {
        self.properties.remove(key);
    }

    pub fn to_string(&self) -> String {
        let mut style = String::new();
        for (key, value) in &self.properties {
            style.push_str(format!("{}:{};", key, value).as_str());
        }
        style
    }
}

pub trait StyleExt {
    fn set_style(&self, key: &str, value: &str);
    fn remove_style(&self, key: &str);
    fn get_style(&self, key: &str) -> Option<String>;
}

impl StyleExt for Element {
    fn set_style(&self, key: &str, value: &str) {
        let mut style = Style::new(&self.get_attribute("style").unwrap_or(String::new()));
        style.set(key, value);
        self.set_attribute("style", &style.to_string()).unwrap();
    }

    fn remove_style(&self, key: &str) {
        let mut style = Style::new(&self.get_attribute("style").unwrap_or(String::new()));
        style.remove(key);
        self.set_attribute("style", &style.to_string()).unwrap();
    }

    fn get_style(&self, key: &str) -> Option<String> {
        let style = Style::new(&self.get_attribute("style").unwrap_or(String::new()));
        style.properties.get(key).map(|s| s.to_string())
    }
}
