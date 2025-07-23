use serde::{Serialize, Deserialize};
use toml::Value;
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KVData {
    data: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct KVStore {
    path: PathBuf,
    kv: KVData,
}

impl KVStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        let path = path.into();

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }
            fs::write(&path, "").expect("Failed to create file");
        }

        let kv = if let Ok(contents) = fs::read_to_string(&path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            KVData::default()
        };

        KVStore { path, kv }
    }

    pub fn save(&self) {
        let toml_str = toml::to_string_pretty(&self.kv).expect("Failed to serialize to TOML");
        fs::write(&self.path, toml_str).expect("Failed to write file");
    }

    pub fn set<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.kv.data.insert(key.to_string(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.kv.data.get(key)
    }

    pub fn get_as_str(&self, key: &str) -> Option<&str> {
        self.kv.data.get(key).and_then(Value::as_str)
    }

    pub fn get_as_integer(&self, key: &str) -> Option<i64> {
        self.kv.data.get(key).and_then(Value::as_integer)
    }

    pub fn get_as_bool(&self, key: &str) -> bool {
        self.get_as_bool_or_default(key, false)
    }

    pub fn get_as_bool_or_default(&self, key: &str, default: bool) -> bool {
        self.kv.data.get(key).and_then(Value::as_bool).unwrap_or(default)
    }

    pub fn get_bool_ref(&self, key: &str) -> Option<&bool> {
        self.kv.data.get(key).and_then(|v| {
            if let Value::Boolean(b) = v {
                Some(b)
            } else {
                None
            }
        })
    }

    pub fn get_bool_ref_mut(&mut self, key: &str) -> Option<&mut bool> {
        self.kv.data.get_mut(key).and_then(|v| {
            if let Value::Boolean(b) = v {
                Some(b)
            } else {
                None
            }
        })
    }

    pub fn get_bool_ref_mut_or_default(&mut self, key: &str, default: bool) -> &mut bool {
        if self.get_bool_ref(key).is_none() {
            self.set(key, default);
        }

        self.get_bool_ref_mut(key).expect("just inserted the default bool")
    }

    pub fn delete(&mut self, key: &str) {
        self.kv.data.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_kvstore_multitype() {
        let tmpfile = NamedTempFile::new().expect("Failed to create temp file");
        let path = tmpfile.path().to_path_buf();
        println!("path: {:?}", path);

        let mut store = KVStore::new(&path);
        store.set("username", "test_user");
        store.set("count", 42);
        store.set("active", true);
        store.set("langs", vec!["rust", "go", "js"]);
        store.save();

        let store2 = KVStore::new(&path);

        assert_eq!(store2.get("username").unwrap().as_str(), Some("test_user"));
        assert_eq!(store2.get("count").unwrap().as_integer(), Some(42));
        assert_eq!(store2.get("active").unwrap().as_bool(), Some(true));

        let langs = store2.get("langs").unwrap().as_array().unwrap();
        let langs_str: Vec<&str> = langs.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(langs_str, vec!["rust", "go", "js"]);
    }
}
