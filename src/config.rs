//! Cluster configuration schema, loading, and validation.

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "
name: test-cluster
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
    fn loads_minimal_cluster_yaml() {
        let cfg = load_from_str(MINIMAL).expect("minimal config must load");
        assert_eq!(cfg.name, "test-cluster");
        assert_eq!(cfg.hosts.len(), 2);
        assert_eq!(cfg.hosts[0].role, Role::Server);
        assert_eq!(
            cfg.hosts[1].address,
            "192.168.30.41".parse::<std::net::IpAddr>().unwrap()
        );
        assert_eq!(cfg.ssh.user, "vagrant");
        assert_eq!(cfg.ssh.port, 22);
    }
}
