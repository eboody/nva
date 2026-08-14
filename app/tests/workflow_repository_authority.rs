use app::workflow_repository::{OutcomeRepository, Repository, RuntimeCounters};

#[derive(Default)]
struct FakeRepository {
    counters: RuntimeCounters,
}

impl Repository for FakeRepository {
    fn runtime_counters(&self) -> RuntimeCounters {
        self.counters
    }
}

#[test]
fn workflow_repository_authority_is_owned_by_the_application_layer() {
    let repository = FakeRepository::default();

    assert_eq!(repository.runtime_counters(), RuntimeCounters::default());
}

#[test]
fn storage_adapter_implements_app_owned_outcome_repository_semantics() {
    let mut repository = storage::workflow_repository::InMemoryOutcomes::default();

    assert_eq!(repository.record("reviewed-outcome"), 1);
    assert_eq!(repository.outcomes(), &["reviewed-outcome"]);
}

#[test]
fn api_shell_contains_no_repository_traits_or_postgres_queries() {
    let http = include_str!("../../apps/api/src/http.rs");

    assert!(!http.contains("trait WorkflowRepository"));
    assert!(!http.contains("tokio_postgres::"));
    assert!(!http.contains("FROM source_quality_backlog"));
}
