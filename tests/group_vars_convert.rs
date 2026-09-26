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
