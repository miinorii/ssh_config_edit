#[derive(Debug, PartialEq)]
pub struct Field {
    key: String,
    value: String
}

pub struct HostFields {
    fields: Vec<Field>
}

impl HostFields {
    pub fn new() -> Self {
        return HostFields { fields: Vec::new() }
    }

    /// Add and dedupe fields the same way that `ssh -G` does
    pub fn add_field(&mut self, key: &str, value: &str) {
        // keys are case insensitive
        let key = key.to_lowercase();

        if !self.contains_key(&key) {
            self.fields.push(Field { key: key, value: value.into() });
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        return self.fields
            .iter()
            .filter(|f| f.key == key)
            .count() != 0;
    }

    /// Access a singular field value corresponding to a case-insensitive key
    pub fn get_one(&self, key: &str) -> Option<&String> {
        return self.fields
            .iter()
            .find(|f| f.key == key.to_lowercase())
            .map(|f| &f.value);
    }

    /// Access multiple fields values corresponding to a case-insensitive key
    pub fn get_multiple(&self, key: &str) -> Vec<&String> {
        let fields: Vec<&String> = self.fields
            .iter()
            .filter(|f| f.key == key.to_lowercase()).map(|f| &f.value)
            .collect();
        return fields;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.fields.len() == 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.fields.len();
    }
}
