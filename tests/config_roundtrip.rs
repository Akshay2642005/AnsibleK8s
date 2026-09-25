//! Round-trip fidelity: load → serialize → load → equal.

use std::path::Path;

use ansiblek8s_rs::config::load_from_str;

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
load_balancer:
  kube_vip_lb_ip_range: 192.168.30.80-192.168.30.90
";

#[test]
fn roundtrips_through_yaml() {
    let original = load_from_str(VALID).expect("valid config loads");
    let yaml = serde_yml::to_string(&original).expect("config must serialize");
    let reloaded = load_from_str(&yaml).expect("serialized config must reload");
    assert_eq!(original, reloaded);
}

#[test]
fn serializes_roles_as_lowercase() {
    let original = load_from_str(VALID).expect("valid config loads");
    let yaml = serde_yml::to_string(&original).expect("config must serialize");
    assert!(yaml.contains("role: server"), "got:\n{yaml}");
    assert!(yaml.contains("role: agent"), "got:\n{yaml}");
}

#[test]
fn shipped_example_config_loads() {
    // Rot guard: the example we hand to users must always pass validation.
    let cfg = ansiblek8s_rs::config::load(Path::new("examples/cluster.yaml"))
        .expect("examples/cluster.yaml must load and validate");
    assert_eq!(cfg.hosts.len(), 5, "sample inventory has 3 servers + 2 agents");
    assert_eq!(
        cfg.hosts.iter().filter(|h| h.role == ansiblek8s_rs::config::Role::Server).count(),
        3
    );
}
