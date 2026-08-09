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

    /// Returns a singular [`Field`] value corresponding to a case-insensitive `key`.
    pub fn get_one(&self, key: &FieldKey) -> Option<&Field> {
        self.fields
            .iter()
            .find(|f| f.key == *key)
    }

    /// Returns a mutable singular [`Field`] value corresponding to a case-insensitive `key`.
    pub fn get_one_mut(&mut self, key: &FieldKey) -> Option<&mut String> {
        self.fields
            .iter_mut()
            .find(|f| f.key == *key)
            .map(|f| &mut f.value)
    }

    /// Returns all [`Field`] values corresponding to a case-insensitive `key`.
    pub fn get_all(&self, key: &FieldKey) -> impl Iterator<Item = &Field> {
        self.fields
            .iter()
            .filter(|f| f.key == *key)
    }

    /// Construct and push a new [`Field`] onto this [`HostSettings`].
    ///
    /// Validation ensures `key` is not a selector (see
    /// [`FieldKey::is_selector`]) and for non-cumulative keys if not
    /// already present.
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

    /// Replace every value for `key` with `values` in order.
    ///
    /// An empty `values` is equivalent to [`remove_all`](Self::remove_all).
    ///
    /// # Errors
    /// Returns [`Error::UnexpectedSelector`] if `key` is `Host` or `Match` and
    /// [`Error::NotCumulative`] if more than one value is given for a
    /// non-cumulative key. SSH honours only the first occurrence, so the rest
    /// would be silently unreachable.
    pub fn replace_all<I, S>(&mut self, key: FieldKey, values: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        if key.is_selector() {
            return Err(Error::UnexpectedSelector(key.to_string()));
        }

        // collected upfront so validation completes before anything is removed
        let values: Vec<String> = values.into_iter().map(|v| v.into()).collect();

        if values.len() > 1 && !key.is_cumulative() {
            return Err(Error::NotCumulative(key.to_string()));
        }

        self.fields.retain(|f| f.key != key);
        self.fields.extend(values.into_iter().map(|value| Field {
            key: key.clone(),
            value,
        }));

        Ok(())
    }

    /// Remove all occurrences of `key` via [`Self::remove_all`], then
    /// [`Self::push`] `value`.
    pub fn replace(&mut self, key: FieldKey, value: &str) -> Result<()> {
        self.replace_all(key, [value])
    }

    /// Remove all occurrences of `key`.
    pub fn remove_all(&mut self, key: &FieldKey) -> bool {
        let count = self.field_count();
        self.fields.retain(|f| f.key != *key);
        count != self.field_count()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Construct and push a new [`Field`] while ensuring fields
    /// are deduped the same way that `ssh -G` does.
    pub(crate) fn push_dedup(&mut self, key: FieldKey, value: &str) {
        if !self.contains_key(&key) || key.is_cumulative() {
            self.fields.push(Field {
                key,
                value: value.into(),
            });
        }
    }
}
