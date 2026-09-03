use std::fmt;

use serde::{
    Deserializer,
    de::{Error, SeqAccess, Visitor},
};

/// Reads a job's `needs` entry, which a workflow may write either as a single
/// job id or as a sequence of job ids.
///
/// Both spellings deserialize into the same list, so a workflow author is free
/// to write `needs: build` or `needs: [build, lint]`.
pub struct JobNeedsVisitor;

impl JobNeedsVisitor {
    /// Deserializes either spelling into the list of job ids a job depends on.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JobNeedsVisitor)
    }
}

impl<'de> Visitor<'de> for JobNeedsVisitor {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a job id or a sequence of job ids")
    }

    fn visit_str<E>(self, job_id: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(vec![job_id.to_string()])
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut job_ids = Vec::new();
        while let Some(job_id) = sequence.next_element::<String>()? {
            job_ids.push(job_id);
        }
        Ok(job_ids)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::workflow::Job;

    #[test]
    fn a_single_job_id_becomes_a_one_element_dependency_list() {
        let job: Job = serde_yaml::from_str("needs: build\n").unwrap();

        assert_eq!(job.needs, vec!["build".to_string()]);
    }

    #[test]
    fn a_sequence_of_job_ids_keeps_every_dependency() {
        let job: Job = serde_yaml::from_str("needs: [build, lint]\n").unwrap();

        assert_eq!(job.needs, vec!["build".to_string(), "lint".to_string()]);
    }

    #[test]
    fn a_job_without_dependencies_has_an_empty_dependency_list() {
        let job: Job = serde_yaml::from_str("runs-on: ubuntu-latest\n").unwrap();

        assert_eq!(job.needs, Vec::<String>::new());
    }
}
