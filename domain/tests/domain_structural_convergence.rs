use domain::staff;

#[test]
fn staff_role_has_a_cycle_free_concept_owner_module() {
    let staff_root = include_str!("../src/staff.rs");
    let role_owner = include_str!("../src/staff/role.rs");
    let operations_labor = include_str!("../src/operations/labor.rs");

    assert!(staff_root.contains("pub mod role;"));
    assert!(staff_root.contains("pub use role::Role;"));
    assert!(!staff_root.contains("pub enum Role"));
    assert!(role_owner.contains("pub enum Role"));
    assert!(!operations_labor.contains("pub use staff::Role;"));

    let role = staff::Role::FrontDesk;
    assert_eq!(role, domain::operations::labor::Role::FrontDesk);
}
