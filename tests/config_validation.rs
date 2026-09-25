//! Validation errors: every failure mode maps to a distinct ConfigError variant.

use ansiblek8s_rs::config::{load_from_str, ConfigError};

const VALID: &str = "
name: test-cluster
k3s_version: v1.30.2+k3s2
hosts:
  - address: 192.168.30.38
    role: server
  - address: 192.168.30.41
    role: agent
ssh:
  user: vagrant
  port: 22
network:
  cni_pod_cidr: 10.42.0.0/16
  cni_service_cidr: 10.43.0.0/16
";

#[test]
fn invalid_cidr_is_typed_error() {
    let yaml = VALID.replace("10.42.0.0/16", "10.42.0.0/99");
    let err = load_from_str(&yaml).expect_err("prefix > 32 must be rejected");
    match err {
        ConfigError::InvalidCidr(value) => assert_eq!(value, "10.42.0.0/99"),
        other => panic!("expected InvalidCidr, got {other:?}"),
    }
}

#[test]
fn duplicate_host_is_typed_error() {
    // Third entry repeats 192.168.30.38 — inserted inside the hosts list.
    let yaml = VALID.replace(
        "ssh:",
        "  - address: 192.168.30.38\n    role: agent\nssh:",
    );
    let err = load_from_str(&yaml).expect_err("duplicate address must be rejected");
    match err {
        ConfigError::DuplicateHost(address) => assert_eq!(address, "192.168.30.38"),
        other => panic!("expected DuplicateHost, got {other:?}"),
    }
}

#[test]
fn unknown_role_is_typed_error() {
    let yaml = VALID.replace("role: server", "role: kontrol");
    let err = load_from_str(&yaml).expect_err("unknown role must be rejected");
    match err {
        ConfigError::UnknownRole(role) => assert_eq!(role, "kontrol"),
        other => panic!("expected UnknownRole, got {other:?}"),
    }
}

#[test]
fn missing_required_field_is_typed_error() {
    let yaml = VALID.replace("name: test-cluster\n", "");
    let err = load_from_str(&yaml).expect_err("missing name must be rejected");
    match err {
        ConfigError::MissingField(field) => assert_eq!(field, "name"),
        other => panic!("expected MissingField, got {other:?}"),
    }
}
