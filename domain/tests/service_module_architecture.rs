use std::{fs, path::Path};

use domain::{boarding, daycare, entities, grooming, money, policy, retail, training};

const SERVICE_LINE_NAMES: [&str; 5] = ["boarding", "daycare", "grooming", "training", "retail"];

#[test]
fn petsuites_service_contracts_are_constructed_through_canonical_service_modules() {
    let contracts = domain::operations::service_core::ServiceContracts::builder()
        .location_id(entities::LocationId(uuid::Uuid::nil()))
        .boarding(boarding::Contract::standard_petsuites())
        .daycare(daycare::Contract::standard_petsuites())
        .grooming(grooming::Contract::standard_petsuites())
        .training(training::Contract::standard_petsuites())
        .retail(retail::Contract::standard_petsuites())
        .build();

    assert_eq!(contracts.core_services().len(), SERVICE_LINE_NAMES.len());
    assert!(contracts.boarding.requires_deposit_collection());
    assert!(contracts.daycare.requires_staff_review_before_group_play());
    assert!(contracts.grooming.requires_deposit_after_no_show());
    assert!(contracts.training.requires_named_trainer());
    assert!(contracts.retail.should_reorder());
}

#[test]
fn service_domain_module_paths_remain_the_owned_home_for_line_specific_policy_types() {
    let boarding_decision = boarding::deposit::Policy::new(
        boarding::DepositRule::Required {
            amount: money::Money::new(
                money::MinorUnits::try_new(2_500).unwrap(),
                money::Currency::Usd,
            ),
        },
        boarding::PaymentTiming::DueAtBooking,
    )
    .readiness_for_confirmation(None);

    assert!(matches!(
        boarding_decision,
        boarding::deposit::ConfirmationReadiness::Blocked {
            blocker: boarding::deposit::Blocker::DepositRequired,
            review_gate: policy::ReviewGate::RefundOrDepositException,
        }
    ));

    let daycare_decision = daycare::coverage::Policy.evaluate(
        &daycare::coverage::RosterSnapshot::new(
            daycare::StaffCount::try_new(1).unwrap(),
            daycare::PetCount::try_new(13).unwrap(),
        ),
        daycare::StaffPetRatio::new(
            daycare::StaffCount::try_new(1).unwrap(),
            daycare::PetCount::try_new(12).unwrap(),
        ),
    );

    assert_eq!(
        daycare_decision,
        daycare::coverage::Decision::Insufficient {
            reason: daycare::coverage::InsufficiencyReason::RatioExceeded,
            gate: policy::ReviewGate::ManagerApproval,
        }
    );
}

#[test]
fn non_compatibility_tests_do_not_import_service_lines_through_operations_shims() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("domain crate lives under workspace root");
    let roots = [
        repo.join("domain/tests"),
        repo.join("app/tests"),
        repo.join("storage/tests"),
        repo.join("integrations/gingr/tests"),
    ];

    let mut stale_paths = Vec::new();
    for root in roots {
        collect_stale_service_shim_imports(&root, &mut stale_paths);
    }

    assert!(
        stale_paths.is_empty(),
        "service-line tests must use domain::<line> paths; stale compatibility imports: {stale_paths:#?}"
    );
}

#[test]
fn stale_service_shim_scanner_catches_grouped_and_aliased_import_forms() {
    let source = [
        "use ",
        "domain",
        "::{",
        "operations",
        "::{boarding as boarding_ops,self as ops}};ops::training::Contract::standard_petsuites();",
    ]
    .concat();
    let compact_source = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    assert!(grouped_operation_import_contains(
        &compact_source,
        "boarding"
    ));
    assert!(operation_module_aliases(&compact_source).contains(&"ops".to_owned()));
    assert!(
        grouped_module_import_contains(&compact_source, "ops::{", "training")
            || compact_source.contains("ops::training")
    );

    let grouped_subpath_source = [
        "use ",
        "domain",
        "::",
        "operations",
        "::{boarding::Contract,training::{Contract as TrainingContract}};",
    ]
    .concat();
    let compact_grouped_subpath_source = grouped_subpath_source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    assert!(grouped_operation_import_contains(
        &compact_grouped_subpath_source,
        "boarding"
    ));
    assert!(grouped_operation_import_contains(
        &compact_grouped_subpath_source,
        "training"
    ));
}

fn collect_stale_service_shim_imports(root: &Path, stale_paths: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_stale_service_shim_imports(&path, stale_paths);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("operations_compatibility.rs") {
            continue;
        }

        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let compact_source = source
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        let operation_aliases = operation_module_aliases(&compact_source);
        for service_line in SERVICE_LINE_NAMES {
            let stale_patterns = [
                format!("domain::operations::{service_line}"),
                format!("operations::{service_line}"),
            ];
            for stale in stale_patterns {
                if compact_source.contains(&stale) {
                    stale_paths.push(format!("{} contains {stale}", path.display()));
                }
            }
            if grouped_operation_import_contains(&compact_source, service_line) {
                stale_paths.push(format!(
                    "{} imports {service_line} from domain::operations grouped imports",
                    path.display()
                ));
            }
            for alias in &operation_aliases {
                let stale = format!("{alias}::{service_line}");
                if compact_source.contains(&stale)
                    || grouped_module_import_contains(
                        &compact_source,
                        &format!("{alias}::{{"),
                        service_line,
                    )
                {
                    stale_paths.push(format!(
                        "{} contains domain::operations alias {alias} for {service_line}",
                        path.display()
                    ));
                }
            }
        }
    }
}

fn grouped_operation_import_contains(compact_source: &str, service_line: &str) -> bool {
    grouped_module_import_contains(compact_source, "domain::operations::{", service_line)
        || grouped_module_import_contains(compact_source, "operations::{", service_line)
}

fn grouped_module_import_contains(compact_source: &str, prefix: &str, service_line: &str) -> bool {
    let mut remaining = compact_source;
    while let Some(start) = remaining.find(prefix) {
        let group_start = start + prefix.len();
        if let Some(group_end) = remaining[group_start..].find('}') {
            let group = &remaining[group_start..group_start + group_end];
            if group.split(',').any(|member| {
                member == service_line
                    || member.starts_with(&format!("{service_line}as"))
                    || member.starts_with(&format!("{service_line}::"))
            }) {
                return true;
            }
            remaining = &remaining[group_start + group_end + 1..];
        } else {
            return false;
        }
    }
    false
}

fn operation_module_aliases(compact_source: &str) -> Vec<String> {
    compact_source
        .split(';')
        .flat_map(|statement| {
            let mut aliases = vec![
                operation_aliases_after(statement, "usedomain::operationsas"),
                operation_aliases_after(statement, "usedomain::{operationsas"),
                operation_aliases_after(statement, "usedomain::operations::{selfas"),
                operation_aliases_after(statement, "usedomain::{operations::{selfas"),
            ];
            if statement.contains("operations::{") {
                aliases.push(operation_aliases_after(statement, "selfas"));
            }
            aliases
        })
        .flatten()
        .collect()
}

fn operation_aliases_after(statement: &str, prefix: &str) -> Vec<String> {
    let mut aliases = Vec::new();
    let mut remaining = statement;
    while let Some(start) = remaining.find(prefix) {
        let alias_start = start + prefix.len();
        let alias = remaining[alias_start..]
            .split([',', '}'])
            .next()
            .unwrap_or_default();
        if !alias.is_empty()
            && alias
                .chars()
                .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        {
            aliases.push(alias.to_owned());
        }
        remaining = &remaining[alias_start..];
    }
    aliases
}
