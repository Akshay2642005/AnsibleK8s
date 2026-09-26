//! Legacy Ansible inventory conversion (ADR-002): hosts.ini + group_vars
//! produce a validated [`ClusterConfig`]. Conversion never writes into
//! `reference/`; it only reads.
//!
//! The parser is hand-written for exactly the documented forms (story
//! e01s02 §16). An INI library cannot do this job: Ansible inventory lines
//! are bare (`192.168.30.38`) or space-separated (`host k=v`), while every
//! INI parser requires `key = value` (rust-ini 0.21 slurped bare lines to
//! EOF and failed with `13:1 expecting [= or :]`).

use std::fmt;
use std::fs;
use std::net::IpAddr;
use std::path::Path;

use ipnet::IpNet;

use crate::config::{ClusterConfig, Host, LoadBalancer, Network, Role, Ssh};

/// Errors raised while converting a legacy inventory.
#[derive(Debug, PartialEq, Eq)]
pub enum ConvertError {
    /// The inventory source is malformed; the message names the source line.
    Parse(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::Parse(msg) => write!(f, "inventory conversion failed: {msg}"),
        }
    }
}

impl std::error::Error for ConvertError {}

/// Parse legacy hosts.ini content into hosts with roles.
///
/// Documented forms only: `[group]` headers, blank lines, `#`/`;`
/// comments, and one host per line as the first whitespace token in
/// plain-IP or `user@host:port` form. Trailing `key=value` tokens on a
/// host line are accepted and ignored (host variables are out of scope
/// for conversion).
///
/// Groups: `[master]` → [`Role::Server`], `[node]` → [`Role::Agent`].
/// When `[k3s_cluster:children]` lists group names, only listed groups
/// contribute hosts (a group present but not a cluster child is not a
/// cluster member); an absent or empty children list is permissive.
/// Other sections contribute no hosts.
pub fn parse_hosts(ini_content: &str) -> Result<Vec<Host>, ConvertError> {
    let children = cluster_children(ini_content);
    let mut hosts = Vec::new();
    let mut section = "";
    for (idx, line) in ini_content.lines().enumerate() {
        let line = line.trim();
        let line_no = idx + 1;
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            section = rest.strip_suffix(']').ok_or_else(|| {
                ConvertError::Parse(format!("line {line_no}: unterminated group header"))
            })?;
            continue;
        }
        let Some(role) = role_of(section, &children) else {
            continue; // not a cluster member group: no host entry
        };
        let spec = line.split_whitespace().next().unwrap_or_default();
        let address = parse_address(spec)
            .map_err(|msg| ConvertError::Parse(format!("line {line_no}: {msg}")))?;
        hosts.push(Host { address, role });
    }
    Ok(hosts)
}

/// Group names listed under `[k3s_cluster:children]`, or `None` when the
/// section is absent.
fn cluster_children(ini_content: &str) -> Option<Vec<String>> {
    let mut children: Option<Vec<String>> = None;
    let mut in_children = false;
    for line in ini_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            in_children = rest.strip_suffix(']').is_some_and(|name| name == "k3s_cluster:children");
            if in_children {
                children.get_or_insert_with(Vec::new);
            }
            continue;
        }
        if in_children
            && let Some(list) = &mut children
            && let Some(name) = line.split_whitespace().next()
        {
            list.push(name.to_string());
        }
    }
    children
}

/// Map a section name to a role, gated by `[k3s_cluster:children]`
/// membership when the list is non-empty.
fn role_of(section: &str, children: &Option<Vec<String>>) -> Option<Role> {
    let role = match section {
        "master" => Role::Server,
        "node" => Role::Agent,
        _ => return None, // k3s_cluster:children etc. are not host lists
    };
    if let Some(list) = children
        && !list.is_empty()
        && !list.iter().any(|name| name == section)
    {
        return None; // group exists but is not a cluster child
    }
    Some(role)
}

/// A legacy key that could not be converted as-is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// The legacy group_vars key (or dotted field path) it affects.
    pub key: String,
    /// Why the key could not be converted directly.
    pub reason: String,
}

/// Build a [`ClusterConfig`] from a legacy inventory directory
/// (`hosts.ini` + `group_vars/all.yaml`).
///
/// Reads only — conversion never writes (the CLI's `-o` target is the sole
/// write, in t3). Mapping policy: known keys map directly
/// (`k3s_version`, `cluster_cidr`); known-derivable keys resolve silently
/// (`apiserver_endpoint` from the first `[master]` host); required fields
/// no inventory dir can supply (`ssh.user`) get a placeholder **and** a
/// warning; every remaining unmapped key must be reported — the
/// no-silent-drop test in t2 asserts that completeness.
pub fn inventory(dir: &Path) -> Result<(ClusterConfig, Vec<Warning>), ConvertError> {
    let ini_path = dir.join("hosts.ini");
    let ini_src = fs::read_to_string(&ini_path)
        .map_err(|err| ConvertError::Parse(format!("{}: {err}", ini_path.display())))?;
    let hosts = parse_hosts(&ini_src)?;

    let gv_path = dir.join("group_vars").join("all.yaml");
    let gv_src = fs::read_to_string(&gv_path)
        .map_err(|err| ConvertError::Parse(format!("{}: {err}", gv_path.display())))?;
    let gv: serde_yml::Value = serde_yml::from_str(&gv_src)
        .map_err(|err| ConvertError::Parse(format!("{}: {err}", gv_path.display())))?;

    let k3s_version = gv
        .get("k3s_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConvertError::Parse("missing required key: k3s_version".to_string()))?
        .to_string();

    let cni_pod_cidr: IpNet = match gv.get("cluster_cidr").and_then(|v| v.as_str()) {
        Some(raw) => raw
            .parse()
            .map_err(|_| ConvertError::Parse(format!("invalid cluster_cidr: {raw}")))?,
        None => "10.42.0.0/16"
            .parse()
            .expect("static k3s default pod CIDR is valid"),
    };
    let cni_service_cidr: IpNet = "10.43.0.0/16"
        .parse()
        .expect("static k3s default service CIDR is valid");

    let mut warnings = Vec::new();

    // No-silent-drop sweep: every group_vars key is either mapped above,
    // resolved to a computed default, or warned about with a reason.
    const MAPPED: [&str; 2] = ["k3s_version", "cluster_cidr"];
    if let Some(mapping) = gv.as_mapping() {
        for (k, v) in mapping {
            let key = k.to_string();
            if MAPPED.contains(&key.as_str()) {
                continue;
            }
            if key == "apiserver_endpoint" {
                // Derivable from the first [master] host (the runtime computes
                // it per ADR-002): silent when masters exist, warned otherwise.
                if !hosts.iter().any(|h| h.role == Role::Server) {
                    warnings.push(Warning {
                        key,
                        reason: "underivable: no [master] hosts to derive from".to_string(),
                    });
                }
                continue;
            }
            let reason = if v.as_str().is_some_and(|s| s.contains("{{")) {
                "jinja expression needs live facts or runtime evaluation: \
                 underivable from the inventory alone"
                    .to_string()
            } else {
                "legacy-only setting with no cluster.yaml field".to_string()
            };
            warnings.push(Warning { key, reason });
        }
    }

    // ssh.user cannot come from an inventory dir (legacy remote_user lives in
    // ansible.cfg outside it): placeholder + warning, never silent.
    warnings.push(Warning {
        key: "ssh.user".to_string(),
        reason: "not derivable from the inventory dir (legacy remote_user lives \
                 outside it): PLACEHOLDER emitted"
            .to_string(),
    });

    // The metallb chain hangs off ansible_default_ipv4 (live facts): emit a
    // documented placeholder so the operator must fill it in explicitly.
    let metal_lb_ip_range = gv
        .get("metallb_ip_range")
        .map(|_| "PLACEHOLDER(metallb_ip_range)".to_string());

    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "converted-cluster".to_string());

    let cfg = ClusterConfig {
        name,
        k3s_version,
        hosts,
        ssh: Ssh {
            user: "PLACEHOLDER".to_string(),
            port: 22,
        },
        network: Network {
            cni_pod_cidr,
            cni_service_cidr,
        },
        load_balancer: LoadBalancer {
            kube_vip_lb_ip_range: None,
            metal_lb_ip_range,
        },
    };
    Ok((cfg, warnings))
}

/// Extract the IP from the documented forms: `192.168.30.38`,
/// `vagrant@192.168.30.45`, `192.168.30.45:2222`, `vagrant@host:port`.
fn parse_address(raw: &str) -> Result<IpAddr, String> {
    let host = raw.rsplit('@').next().unwrap_or(raw);
    let host = match host.rsplit_once(':') {
        Some((head, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => head,
        _ => host,
    };
    host.parse()
        .map_err(|_| format!("invalid host address: {raw}"))
}
