//! e01s02 t2: legacy group_vars → typed config fields via `convert::inventory`.

use std::fs;
use std::path::Path;

use ansiblek8s_rs::convert;

const SAMPLE_DIR: &str = "reference/inventory/sample";

#[test]
fn inventory_builds_complete_config_from_legacy_dirs() {
    // Case 1: the oracle sample maps fully into a ClusterConfig.
    let (cfg, warnings) = convert::inventory(Path::new(SAMPLE_DIR))
        .expect("sample inventory must convert");
    assert_eq!(cfg.hosts.len(), 5, "3 masters + 2 nodes survive the pipeline");
    assert_eq!(
        cfg.k3s_version, "v1.30.2+k3s2",
        "k3s_version maps from group_vars"
    );
    assert_eq!(
        cfg.network.cni_pod_cidr.to_string(),
        "10.52.0.0/16",
        "cluster_cidr maps to the pod CIDR"
    );
    assert_eq!(
        cfg.network.cni_service_cidr.to_string(),
        "10.43.0.0/16",
        "service CIDR uses the k3s default"
    );
    assert_eq!(
        cfg.name, "sample",
        "cluster name derives from the inventory directory"
    );
    assert!(
        !warnings.iter().any(|w| w.key.contains("apiserver_endpoint")),
        "derivable apiserver_endpoint must resolve without a warning: {warnings:?}"
    );

    // Case 2: documented `user@host:port` form survives the pipeline.
    let tmp = std::env::temp_dir().join("ansiblek8s_rs_inventory_userhost");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join("group_vars")).expect("tmp fixture");
    fs::write(tmp.join("hosts.ini"), "[master]\nvagrant@192.168.30.55:2222\n")
        .expect("tmp fixture");
    fs::write(tmp.join("group_vars/all.yaml"), "k3s_version: v1.30.2+k3s2\n")
        .expect("tmp fixture");
    let (cfg2, _) = convert::inventory(&tmp).expect("user@host fixture converts");
    assert_eq!(
        cfg2.hosts[0].address,
        "192.168.30.55".parse::<std::net::IpAddr>().unwrap()
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn no_legacy_key_is_silently_dropped() {
    let (cfg, warnings) = convert::inventory(Path::new(SAMPLE_DIR)).expect("converts");

    // Enumerate every key the oracle's group_vars actually defines.
    let gv_src = fs::read_to_string(Path::new(SAMPLE_DIR).join("group_vars/all.yaml"))
        .expect("oracle group_vars readable");
    let gv: serde_yml::Value = serde_yml::from_str(&gv_src).expect("group_vars is YAML");
    let keys: Vec<String> = gv
        .as_mapping()
        .expect("group_vars is a mapping")
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(keys.len() > 30, "sanity: oracle defines many keys, got {}", keys.len());

    // Accounted = mapped into a field, resolved to a computed default, or
    // warned about. Anything else was silently dropped.
    const MAPPED: [&str; 2] = ["k3s_version", "cluster_cidr"];
    const RESOLVED: [&str; 1] = ["apiserver_endpoint"];
    for key in &keys {
        if MAPPED.contains(&key.as_str()) || RESOLVED.contains(&key.as_str()) {
            continue;
        }
        assert!(
            warnings.iter().any(|w| &w.key == key && !w.reason.is_empty()),
            "group_vars key {key:?} was silently dropped"
        );
    }

    // Underivable metallb chain: placeholder in the config field, not silence.
    assert_eq!(
        cfg.load_balancer.metal_lb_ip_range.as_deref(),
        Some("PLACEHOLDER(metallb_ip_range)"),
        "underivable metallb_ip_range must emit a documented placeholder"
    );
    assert!(
        warnings.iter().any(|w| w.key == "k3s_node_ip"),
        "facts-dependent k3s_node_ip must be warned about"
    );
}
