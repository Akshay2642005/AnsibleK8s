//! Cluster configuration schema, loading, and validation.
//!
//! Two layers: a private `Raw*` layer that mirrors the YAML shape (strings),
//! and the public typed layer produced by [`validate`]. This is what turns
//! per-field parse failures into distinct, matchable [`ConfigError`] variants.

use std::collections::HashSet;
use std::net::IpAddr;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};

/// Errors raised while loading or validating a cluster configuration.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// The document is not valid YAML, or a value has the wrong shape.
    Parse(String),
    /// A required field is absent.
    MissingField(String),
    /// A CIDR value does not parse.
    InvalidCidr(String),
    /// Two hosts declare the same address.
    DuplicateHost(String),
    /// A host declares a role that is neither `server` nor `agent`.
    UnknownRole(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Parse(msg) => write!(f, "invalid cluster config: {msg}"),
            ConfigError::MissingField(field) => write!(f, "missing required field: {field}"),
            ConfigError::InvalidCidr(value) => write!(f, "invalid CIDR: {value}"),
            ConfigError::DuplicateHost(address) => write!(f, "duplicate host address: {address}"),
            ConfigError::UnknownRole(role) => {
                write!(f, "unknown role: {role} (expected server or agent)")
            }
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterConfig {
    pub name: String,
    pub k3s_version: String,
    pub hosts: Vec<Host>,
    pub ssh: Ssh,
    pub network: Network,
    pub load_balancer: LoadBalancer,
}

/// One machine in the cluster.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Host {
    pub address: IpAddr,
    pub role: Role,
}

/// The k3s role a host plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Server,
    Agent,
}

/// SSH connection options for target hosts.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ssh {
    pub user: String,
    pub port: u16,
}

/// Kubernetes network ranges.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Network {
    pub cni_pod_cidr: IpNet,
    pub cni_service_cidr: IpNet,
}

/// Load-balancer address pools. Both pools are optional and mutually
/// exclusive selection happens in a later epic (e03s05).
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoadBalancer {
    pub kube_vip_lb_ip_range: Option<String>,
    pub metal_lb_ip_range: Option<String>,
}

// --- raw layer (YAML shape, strings only) -----------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    name: Option<String>,
    k3s_version: Option<String>,
    hosts: Vec<RawHost>,
    ssh: Ssh,
    network: RawNetwork,
    #[serde(default)]
    load_balancer: LoadBalancer,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHost {
    address: String,
    role: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNetwork {
    cni_pod_cidr: String,
    cni_service_cidr: String,
}

// --- loading -----------------------------------------------------------------

/// Load a cluster configuration from YAML text.
///
/// This function is the YAML-backend seam: every raw-YAML access happens
/// here so the parser crate stays swappable (SLOPCHECK: serde_yml [SUS]).
pub fn load_from_str(yaml: &str) -> Result<ClusterConfig, ConfigError> {
    let raw: RawConfig = serde_yml::from_str(yaml)?;
    validate(raw)
}

/// Convert the raw YAML shape into the typed configuration, one distinct
/// error per failure mode.
fn validate(raw: RawConfig) -> Result<ClusterConfig, ConfigError> {
    let name = raw
        .name
        .ok_or_else(|| ConfigError::MissingField("name".to_string()))?;
    let k3s_version = raw
        .k3s_version
        .ok_or_else(|| ConfigError::MissingField("k3s_version".to_string()))?;

    let mut seen: HashSet<IpAddr> = HashSet::new();
    let mut hosts = Vec::with_capacity(raw.hosts.len());
    for host in raw.hosts {
        let address: IpAddr = host
            .address
            .parse()
            .map_err(|_| ConfigError::Parse(format!("invalid host address: {}", host.address)))?;
        if !seen.insert(address) {
            return Err(ConfigError::DuplicateHost(address.to_string()));
        }
        let role = match host.role.as_str() {
            "server" => Role::Server,
            "agent" => Role::Agent,
            other => return Err(ConfigError::UnknownRole(other.to_string())),
        };
        hosts.push(Host { address, role });
    }

    let network = Network {
        cni_pod_cidr: raw
            .network
            .cni_pod_cidr
            .parse()
            .map_err(|_| ConfigError::InvalidCidr(raw.network.cni_pod_cidr))?,
        cni_service_cidr: raw
            .network
            .cni_service_cidr
            .parse()
            .map_err(|_| ConfigError::InvalidCidr(raw.network.cni_service_cidr))?,
    };

    Ok(ClusterConfig {
        name,
        k3s_version,
        hosts,
        ssh: raw.ssh,
        network,
        load_balancer: raw.load_balancer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "
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
    fn rejects_unknown_fields() {
        let yaml = format!("{MINIMAL}bogus_field: 1\n");
        let err = load_from_str(&yaml).expect_err("unknown top-level key must be rejected");
        assert!(
            err.to_string().contains("bogus_field"),
            "error must name the offending key, got: {err}"
        );
    }

    #[test]
    fn loads_minimal_cluster_yaml() {
        let cfg = load_from_str(MINIMAL).expect("minimal config must load");
        assert_eq!(cfg.name, "test-cluster");
        assert_eq!(cfg.k3s_version, "v1.30.2+k3s2");
        assert_eq!(cfg.hosts.len(), 2);
        assert_eq!(cfg.hosts[0].role, Role::Server);
        assert_eq!(
            cfg.hosts[1].address,
            "192.168.30.41".parse::<std::net::IpAddr>().unwrap()
        );
        assert_eq!(cfg.ssh.user, "vagrant");
        assert_eq!(cfg.ssh.port, 22);
        // LB block is optional; minimal config carries none.
        assert!(cfg.load_balancer.kube_vip_lb_ip_range.is_none());
        assert!(cfg.load_balancer.metal_lb_ip_range.is_none());
    }
}
