//! e01s02 t1: legacy hosts.ini groups → hosts with roles.

use std::fs;
use std::path::Path;

use ansiblek8s_rs::config::Role;
use ansiblek8s_rs::convert::parse_hosts;

const SAMPLE_INI: &str = "reference/inventory/sample/hosts.ini";

#[test]
fn sample_inventory_maps_groups_to_roles() {
    let content =
        fs::read_to_string(Path::new(SAMPLE_INI)).expect("oracle sample must be readable");
    let hosts = parse_hosts(&content).expect("sample hosts.ini must parse");

    assert_eq!(hosts.len(), 5, "sample has 3 masters + 2 nodes");

    let servers: Vec<String> = hosts
        .iter()
        .filter(|h| h.role == Role::Server)
        .map(|h| h.address.to_string())
        .collect();
    let agents: Vec<String> = hosts
        .iter()
        .filter(|h| h.role == Role::Agent)
        .map(|h| h.address.to_string())
        .collect();
    assert_eq!(
        servers,
        ["192.168.30.38", "192.168.30.39", "192.168.30.40"],
        "[master] maps to role: server"
    );
    assert_eq!(
        agents,
        ["192.168.30.41", "192.168.30.42"],
        "[node] maps to role: agent"
    );
}

#[test]
fn k3s_cluster_children_gate_membership() {
    // [node] exists but is NOT listed under [k3s_cluster:children],
    // so its host is not a cluster member and must not convert.
    let ini = "\
[master]
192.168.30.38
[node]
192.168.30.41
[k3s_cluster:children]
master
";
    let hosts = parse_hosts(ini).expect("parses");
    assert_eq!(
        hosts.len(),
        1,
        "[node] is not a k3s_cluster child: its host must be excluded"
    );
    assert_eq!(hosts[0].address, "192.168.30.38".parse::<std::net::IpAddr>().unwrap());
}
