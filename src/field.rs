/// Fields parsed cumulatively (append accross matching key) rather than first-match-win.
/// 
/// Derived from `dump_cfg_strarray` and `dump_cfg_forwards` usage in OpenSSH's readconf.c 
/// https://github.com/openssh/openssh-portable/blob/c25c84074a47f700dd6534995b4af4b456927150/readconf.c
pub const CUMULATIVE_FIELDS: &[&str] = &[
    "IdentityFile", 
    "CertificateFile",
    "SendEnv",
    "SetEnv",
    "LocalForward",
    "RemoteForward",
    "DynamicForward"
];

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
        if !self.contains_key(&key) || Self::is_cumulative(key) {
            self.fields.push(Field { key: key.into(), value: value.into() });
        }
    }

    pub fn is_cumulative(key: &str) -> bool {
        return CUMULATIVE_FIELDS.iter().any(|f| key.eq_ignore_ascii_case(f));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        return self.fields
            .iter()
            .filter(|f| f.key.eq_ignore_ascii_case(key))
            .count() != 0;
    }

    /// Access a singular field value corresponding to a case-insensitive key
    pub fn get_one(&self, key: &str) -> Option<&String> {
        return self.fields
            .iter()
            .find(|f| f.key.eq_ignore_ascii_case(key))
            .map(|f| &f.value);
    }

    /// Access multiple fields values corresponding to a case-insensitive key
    pub fn get_multiple(&self, key: &str) -> Vec<&String> {
        let fields: Vec<&String> = self.fields
            .iter()
            .filter(|f| f.key.eq_ignore_ascii_case(key))
            .map(|f| &f.value)
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
