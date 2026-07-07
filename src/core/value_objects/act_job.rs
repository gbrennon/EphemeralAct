#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActJob(String);

impl ActJob {
    /// Creates a new job from its name string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ActJob;
    /// let job = ActJob::new("test".into());
    /// assert_eq!(job.as_str(), "test");
    /// ```
    pub fn new(job: String) -> Self {
        Self(job)
    }

    /// Returns the job name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_job_name() {
        let job = ActJob::new("test".into());
        assert_eq!(job.as_str(), "test");
    }
}
