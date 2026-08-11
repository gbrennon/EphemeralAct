use std::collections::{HashMap, HashSet, VecDeque};

use super::{plan::Plan, plan_error::PlanError, run::Run, stage::Stage};
use crate::core::workflow::Workflow;

/// Plans the execution order of workflow jobs.
///
/// Builds a DAG from job `needs` dependencies and topologically sorts
/// into stages where independent jobs run in parallel.
pub struct Planner;

impl Planner {
    /// Creates a new planner.
    pub fn new() -> Self {
        Self
    }

    /// Plans the execution of a single workflow.
    ///
    /// Jobs with no `needs` go in the first stage. Jobs that depend on
    /// other jobs go in later stages. The planner detects cycles and
    /// returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use ephemeral_act::core::planner::Planner;
    /// use ephemeral_act::core::workflow::Workflow;
    ///
    /// let yaml = r#"
    /// on: push
    /// jobs:
    ///   build:
    ///     runs-on: ubuntu-latest
    ///     steps: [{run: make}]
    ///   test:
    ///     runs-on: ubuntu-latest
    ///     needs: [build]
    ///     steps: [{run: make test}]
    /// "#;
    /// let wf: Workflow = serde_yaml::from_str(yaml).unwrap();
    /// let plan = Planner::new().plan(&wf).unwrap();
    /// assert_eq!(plan.stages.len(), 2);
    /// ```
    pub fn plan(&self, workflow: &Workflow) -> Result<Plan, PlanError> {
        let job_ids: Vec<&String> = workflow.jobs.keys().collect();

        // Build adjacency: job_id -> set of jobs it depends on
        let mut dependencies: HashMap<&str, Vec<&str>> = HashMap::new();
        for id in &job_ids {
            let job = &workflow.jobs[*id];
            let deps: Vec<&str> = job.needs.iter().map(|n| n.as_str()).collect();
            dependencies.insert(id.as_str(), deps);
        }

        // Detect cycles via DFS
        self.detect_cycles(&dependencies)?;

        // Topological sort into stages
        let stages = self.topological_sort(&dependencies, workflow)?;

        Ok(Plan { stages })
    }

    /// Detects cycles in the dependency graph.
    fn detect_cycles(&self, deps: &HashMap<&str, Vec<&str>>) -> Result<(), PlanError> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for &node in deps.keys() {
            if !visited.contains(node) {
                self.dfs_cycle(node, deps, &mut visited, &mut in_stack)?;
            }
        }
        Ok(())
    }

    fn dfs_cycle<'a>(
        &self,
        node: &'a str,
        deps: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        in_stack: &mut HashSet<&'a str>,
    ) -> Result<(), PlanError> {
        visited.insert(node);
        in_stack.insert(node);

        if let Some(neighbors) = deps.get(node) {
            for &neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle(neighbor, deps, visited, in_stack)?;
                } else if in_stack.contains(neighbor) {
                    return Err(PlanError::CycleDetected {
                        job: node.to_owned(),
                        dependency: neighbor.to_owned(),
                    });
                }
            }
        }

        in_stack.remove(node);
        Ok(())
    }

    /// Topologically sorts jobs into stages.
    ///
    /// Stage 0: jobs with no dependencies.
    /// Stage N: jobs whose dependencies are all in stages < N.
    fn topological_sort(
        &self,
        deps: &HashMap<&str, Vec<&str>>,
        workflow: &Workflow,
    ) -> Result<Vec<Stage>, PlanError> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        for (&job_id, job_deps) in deps {
            in_degree.entry(job_id).or_insert(0);
            for &dep in job_deps {
                if !deps.contains_key(dep) {
                    return Err(PlanError::MissingDependency {
                        job: job_id.to_owned(),
                        dependency: dep.to_owned(),
                    });
                }
                *in_degree.entry(job_id).or_insert(0) += 1;
                dependents.entry(dep).or_default().push(job_id);
            }
        }

        // Queue jobs with no dependencies
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut stages: Vec<Stage> = Vec::new();
        let mut processed = 0usize;
        let total = deps.len();

        while !queue.is_empty() {
            // All jobs currently in the queue form one stage
            let stage_jobs: Vec<&str> = queue.drain(..).collect();
            let runs: Vec<Run> = stage_jobs
                .iter()
                .map(|&id| {
                    let job = workflow.jobs[id].clone();
                    Run {
                        workflow_name: workflow.name.clone(),
                        job_id: id.to_owned(),
                        job,
                        matrix_values: None,
                    }
                })
                .collect();

            stages.push(Stage { runs });
            processed += stage_jobs.len();

            // Decrease in-degree of dependents
            for &job_id in &stage_jobs {
                if let Some(deps) = dependents.get(job_id) {
                    for &dep_id in deps {
                        if let Some(deg) = in_degree.get_mut(dep_id) {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push_back(dep_id);
                            }
                        }
                    }
                }
            }
        }

        if processed != total {
            return Err(PlanError::UnresolvedDependencies);
        }

        Ok(stages)
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_workflow(jobs_yaml: &str) -> Workflow {
        let yaml = format!("on: push\njobs:\n{}", jobs_yaml);
        serde_yaml::from_str(&yaml).unwrap()
    }

    #[test]
    fn plan_single_job() {
        let wf = make_workflow("  build:\n    runs-on: ubuntu-latest\n    steps: [{run: echo}]");
        let plan = Planner::new().plan(&wf).unwrap();
        assert_eq!(plan.stages.len(), 1);
        assert_eq!(plan.stages[0].runs.len(), 1);
        assert_eq!(plan.stages[0].runs[0].job_id, "build");
    }

    #[test]
    fn plan_independent_jobs_same_stage() {
        let wf = make_workflow(
            "  build:\n    runs-on: ubuntu-latest\n    steps: [{run: make}]\n  lint:\n    runs-on: ubuntu-latest\n    steps: [{run: cargo clippy}]",
        );
        let plan = Planner::new().plan(&wf).unwrap();
        assert_eq!(plan.stages.len(), 1);
        assert_eq!(plan.stages[0].runs.len(), 2);
    }

    #[test]
    fn plan_sequential_jobs() {
        let wf = make_workflow(
            "  build:\n    runs-on: ubuntu-latest\n    steps: [{run: make}]\n  test:\n    runs-on: ubuntu-latest\n    needs: [build]\n    steps: [{run: make test}]",
        );
        let plan = Planner::new().plan(&wf).unwrap();
        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0].runs[0].job_id, "build");
        assert_eq!(plan.stages[1].runs[0].job_id, "test");
    }

    #[test]
    fn plan_diamond_dependency() {
        let wf = make_workflow(
            "  build:\n    runs-on: ubuntu-latest\n    steps: [{run: make}]\n  test:\n    runs-on: ubuntu-latest\n    needs: [build]\n    steps: [{run: make test}]\n  lint:\n    runs-on: ubuntu-latest\n    needs: [build]\n    steps: [{run: cargo clippy}]\n  deploy:\n    runs-on: ubuntu-latest\n    needs: [test, lint]\n    steps: [{run: deploy}]",
        );
        let plan = Planner::new().plan(&wf).unwrap();
        assert_eq!(plan.stages.len(), 3);
        // Stage 0: build
        assert_eq!(plan.stages[0].runs.len(), 1);
        // Stage 1: test, lint (both depend on build)
        assert_eq!(plan.stages[1].runs.len(), 2);
        // Stage 2: deploy (depends on test and lint)
        assert_eq!(plan.stages[2].runs.len(), 1);
    }

    #[test]
    fn plan_detects_cycle() {
        let wf = make_workflow(
            "  a:\n    runs-on: ubuntu-latest\n    needs: [b]\n    steps: [{run: echo}]\n  b:\n    runs-on: ubuntu-latest\n    needs: [a]\n    steps: [{run: echo}]",
        );
        let result = Planner::new().plan(&wf);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlanError::CycleDetected { .. }
        ));
    }

    #[test]
    fn plan_detects_missing_dependency() {
        let wf = make_workflow(
            "  build:\n    runs-on: ubuntu-latest\n    needs: [nonexistent]\n    steps: [{run: echo}]",
        );
        let result = Planner::new().plan(&wf);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PlanError::MissingDependency { .. }
        ));
    }

    #[test]
    fn default_creates_planner() {
        let _planner = Planner;
    }
}
