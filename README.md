# KubeCluster: Automated High Availability Kubernetes Deployment

![KubeCluster Automated Deployment](https://img.youtube.com/vi/example-video-id/0.jpg)

This project provides an Ansible-based solution to deploy a fully automated HA Kubernetes cluster with integrated load balancing using `kube-vip` and `MetalLB`.

Built upon industry best practices for high availability Kubernetes, this implementation leverages `etcd` for distributed consensus and provides a straightforward approach to deploying production-ready clusters.

## 📋 Overview

Deploy a robust Kubernetes cluster with Ansible automation. Supports the following distributions:

- [x] Debian 11+
- [x] Ubuntu 20.04+ 
- [x] Rocky Linux 8+
- [x] Fedora 36+

Compatible with these processor architectures:

- [X] x64/amd64
- [X] arm64
- [X] armhf

## 🔧 System Requirements

- **Control Node**: Requires Ansible 2.12+ installed
- **Required Collections**: Install with `ansible-galaxy collection install -r ./collections/requirements.yml`
- **Python Dependencies**: `netaddr` package must be available to Ansible
- **Node Access**: Passwordless SSH access recommended between control node and all cluster nodes

## 🚀 Getting Started

### 📝 Initial Setup

1. Clone the repository and create your cluster configuration:

```bash
git clone https://github.com/Akshay2642005/kubecluster.git
cd kubecluster
cp -R inventory/sample inventory/my-cluster
```

2. Configure your hosts in `inventory/my-cluster/hosts.ini`:

```ini
[master]
192.168.18.101
192.168.18.102
192.168.18.103

[worker]
192.168.18.111
192.168.18.112
192.168.18.113

[k8s_cluster:children]
master
worker
```

> When multiple hosts are specified in the master group, the playbook automatically configures a high availability setup with embedded etcd.

3. Configure your deployment by customizing `inventory/my-cluster/group_vars/all.yml`

4. Copy the Ansible config and update as needed:

```bash
cp ansible.example.cfg ansible.cfg
```

### ☸️ Deploy Your Cluster

Run the deployment playbook:

```bash
ansible-playbook deploy.yml -i inventory/my-cluster/hosts.ini
```

The control plane will be accessible via the virtual IP address specified as `apiserver_endpoint` in your configuration.

### 🧹 Cleanup Resources

To completely remove the Kubernetes cluster:

```bash
ansible-playbook teardown.yml -i inventory/my-cluster/hosts.ini
```

> Note: A system reboot is recommended after teardown to ensure all components are properly cleaned up, especially the virtual IP.

## ⚙️ Accessing Your Cluster

Copy the kubeconfig file from a master node:

```bash
scp user@master_ip:/etc/kubernetes/admin.conf ~/.kube/config
```

If permission issues occur, temporarily adjust permissions on the master:
```bash
# On master node:
sudo chmod 644 /etc/kubernetes/admin.conf

# After copying, reset to secure permissions:
sudo chmod 600 /etc/kubernetes/admin.conf
```

Update the kubeconfig file to use your master IP or VIP:
```bash
sed -i 's/127.0.0.1/your_master_ip/g' ~/.kube/config
```

### 🔍 Verify Your Cluster

Useful commands to verify your cluster is working correctly:

```bash
# Check node status
kubectl get nodes -o wide

# Confirm pods are running
kubectl get pods -A

# Test load balancing
kubectl create deployment nginx --image=nginx
kubectl expose deployment nginx --port=80 --type=LoadBalancer
```

## 🔄 Configuration Variables

| Category | Variable | Type | Default | Required | Description |
|---|---|---|---|---|---|
| **General** | `kubernetes_version` | string | ❌ | Yes | Kubernetes version to install |
| **Networking** | `apiserver_endpoint` | string | ❌ | Yes | Virtual IP for control plane access |
| **Networking** | `pod_cidr` | string | `10.42.0.0/16` | No | CIDR range for pod networking |
| **Networking** | `service_cidr` | string | `10.43.0.0/16` | No | CIDR range for service networking |
| **LoadBalancing** | `metallb_ip_range` | string | ❌ | Yes | IP range for MetalLB load balancer |
| **LoadBalancing** | `metallb_mode` | string | `layer2` | No | MetalLB operating mode (layer2 or bgp) |
| **HighAvailability** | `kube_vip_tag` | string | `v0.5.0` | No | Version tag for kube-vip |
| **HighAvailability** | `kube_vip_interface` | string | ❌ | No | Network interface for kube-vip (auto-detected if not specified) |
| **Storage** | `enable_local_storage` | bool | `false` | No | Enable local path provisioner |
| **System** | `timezone` | string | `UTC` | No | Timezone for all nodes |

## 🔍 Troubleshooting

Common issues and solutions:

1. **Nodes Not Joining**: Check firewall settings and ensure ports 6443, 10250 are open
2. **Network Plugin Issues**: Verify CNI configuration and pod CIDR settings
3. **LoadBalancer Not Working**: Ensure MetalLB IP range is within your network subnet
4. **Certificate Errors**: Check system time sync across all nodes

## 🧪 Testing

This project includes molecule tests for validation. Run them with:

```bash
# Install dependencies
pip install molecule molecule-plugins[docker]

# Run tests
molecule test
```

## 🤝 Contributing

Contributions welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run pre-commit hooks: `pre-commit run --all-files`
5. Submit a pull request

## 📜 License

This project is licensed under the MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

Special thanks to:
- The Kubernetes community
- Contributors to kube-vip and MetalLB projects
- The Ansible community for automation tools and patterns
- @Techno-Tim for sharing the original repo and inspiration
