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
use std::net::IpAddr;

use crate::config::{Host, Role};

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
