//! Cluster configuration schema, loading, and validation.

use std::net::IpAddr;

use ipnet::IpNet;
use serde::Deserialize;

/// Errors raised while loading a cluster configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The document is not valid YAML, or a value has the wrong shape.
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Parse(msg) => write!(f, "invalid cluster config: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<serde_yml::Error> for ConfigError {
    fn from(err: serde_yml::Error) -> Self {
        ConfigError::Parse(err.to_string())
    }
}

/// Fully validated cluster configuration.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub hosts: Vec<Host>,
    pub ssh: Ssh,
    pub network: Network,
}

/// One machine in the cluster.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Host {
    pub address: IpAddr,
    pub role: Role,
}

/// The k3s role a host plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Server,
    Agent,
}

/// SSH connection options for target hosts.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Ssh {
    pub user: String,
    pub port: u16,
}

/// Kubernetes network ranges.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Network {
    pub cni_pod_cidr: IpNet,
    pub cni_service_cidr: IpNet,
}

/// Load a cluster configuration from YAML text.
///
/// This function is the YAML-backend seam: every raw-YAML access happens
/// here so the parser crate stays swappable (SLOPCHECK: serde_yml [SUS]).
pub fn load_from_str(yaml: &str) -> Result<ClusterConfig, ConfigError> {
    Ok(serde_yml::from_str(yaml)?)
}

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
