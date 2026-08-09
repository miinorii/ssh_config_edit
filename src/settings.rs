use crate::error::{Error, Result};
use crate::field_keys::FieldKey;

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub key: FieldKey,
    pub value: String,
}

pub struct HostSettings {
    host: String,
    fields: Vec<Field>,
}

impl HostSettings {
    pub fn new(host: &str) -> Self {
        HostSettings {
            host: host.into(),
            fields: Vec::new(),
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn contains_key(&self, key: &FieldKey) -> bool {
        self.fields.iter().any(|f| f.key == *key)
    }

    /// Returns a singular field value corresponding to a case-insensitive `key`.
    pub fn get_one(&self, key: &FieldKey) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.key == *key)
            .map(|f| f.value.as_str())
    }

    /// Returns a mutable singular field value corresponding to a case-insensitive `key`.
    pub fn get_one_mut(&mut self, key: &FieldKey) -> Option<&mut String> {
        self.fields
            .iter_mut()
            .find(|f| f.key == *key)
            .map(|f| &mut f.value)
    }

    /// Returns all fields values corresponding to a case-insensitive key.
    pub fn get_all(&self, key: &FieldKey) -> impl Iterator<Item = &str> {
        self.fields
            .iter()
            .filter(|f| f.key == *key)
            .map(|f| f.value.as_str())
    }

    /// Construct and push a new `Field` to `HostSettings`.
    ///
    /// Validation is done to ensure `key`
    /// not a selector and does not already exist.
    pub fn push(&mut self, key: FieldKey, value: &str) -> Result<()> {
        if key.is_selector() {
            return Err(Error::UnexpectedSelector(key.to_string()));
        }

        if self.contains_key(&key) && !key.is_cumulative() {
            return Err(Error::NotCumulative(key.to_string()));
        }
        self.fields.push(Field {
            key,
            value: value.to_string(),
        });
        Ok(())
    }

    /// Remove all occurence of `key` then push `value`.
    pub fn replace(&mut self, key: FieldKey, value: &str) -> Result<()> {
        self.remove_all(&key);
        self.push(key, value)
    }

    /// Remove all occurence of `key`.
    pub fn remove_all(&mut self, key: &FieldKey) -> bool {
        let count = self.field_count();
        self.fields.retain(|f| f.key != *key);
        count != self.field_count()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Construct and push a new `Field` while ensuring fields
    /// are deduped the same way that `ssh -G` does
    pub(crate) fn push_dedup(&mut self, key: FieldKey, value: &str) {
        if !self.contains_key(&key) || key.is_cumulative() {
            self.fields.push(Field {
                key,
                value: value.into(),
            });
        }
    }
}
