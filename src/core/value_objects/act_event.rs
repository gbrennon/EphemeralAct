#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActEvent(String);

impl ActEvent {
    /// Creates a new event from its name string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ActEvent;
    /// let event = ActEvent::new("push".into());
    /// assert_eq!(event.as_str(), "push");
    /// ```
    pub fn new(event: String) -> Self {
        Self(event)
    }

    /// Returns the event name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_event_name() {
        let event = ActEvent::new("push".into());
        assert_eq!(event.as_str(), "push");
    }
}
