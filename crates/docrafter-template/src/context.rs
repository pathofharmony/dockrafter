//! Key/value context for `{{placeholder}}` substitution.

use std::collections::HashMap;

/// Variables available when rendering templates.
#[derive(Debug, Clone, Default)]
pub struct Context {
    vars: HashMap<String, String>,
}

impl Context {
    /// Empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a string variable.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Fluent: one variable.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.set(key, value);
        self
    }

    /// Lookup a variable (missing keys stay as `{{key}}` in output).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str)
    }

    /// Iterator over all bindings.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
