use crate::field_keys::FieldKey;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub key: FieldKey,
    pub value: String,
}

pub struct HostSettings {
    pub host: String,
    pub fields: Vec<Field>,
}

impl HostSettings {
    pub fn new(host: &str) -> Self {
        HostSettings {
            host: host.into(),
            fields: Vec::new(),
        }
    }

    /// Add and dedupe fields the same way that `ssh -G` does
    pub fn add_field(&mut self, field: Field) {
        if !self.contains_key(&field.key) || field.key.is_cumulative() {
            self.fields.push(field);
        }
    }

    pub fn contains_key(&self, key: &FieldKey) -> bool {
        self.fields.iter().any(|f| f.key == *key)
    }

    /// Access a singular field value corresponding to a case-insensitive key
    pub fn get_one(&self, key: &FieldKey) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.key == *key)
            .map(|f| f.value.as_str())
    }

    /// Access multiple fields values corresponding to a case-insensitive key
    pub fn get_multiple(&self, key: &FieldKey) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|f| f.key == *key)
            .map(|f| f.value.as_str())
            .collect()
    }

    pub fn cumulative_fields(&self) -> HashMap<&FieldKey, Vec<&Field>> {
        let mut cumul_key_to_value: HashMap<&FieldKey, Vec<&Field>> = HashMap::new();
        for field in self.fields.iter().filter(|f| f.key.is_cumulative()) {
            cumul_key_to_value
                .entry(&field.key)
                .or_default()
                .push(field);
        }
        cumul_key_to_value
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fields.len()
    }
}
