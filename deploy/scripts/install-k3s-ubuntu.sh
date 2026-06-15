#!/usr/bin/env bash
# Install k3s single-node and deploy prerequisites on Ubuntu (CI or remote host).
# Idempotent: safe to re-run.
set -euo pipefail

K3S_BIN="${K3S_BIN:-/usr/local/bin/k3s}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${HOME}/.kube/config}"

echo "=== install-k3s-ubuntu.sh ==="

if ! command -v k3s >/dev/null 2>&1; then
  echo "Installing k3s..."
  curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--write-kubeconfig-mode 644" sh -
else
  echo "k3s already installed"
fi

mkdir -p "$(dirname "${KUBECONFIG_PATH}")"
if [[ -f /etc/rancher/k3s/k3s.yaml ]]; then
  sudo cp /etc/rancher/k3s/k3s.yaml "${KUBECONFIG_PATH}"
  sudo chown "$(id -u)":"$(id -g)" "${KUBECONFIG_PATH}"
  chmod 600 "${KUBECONFIG_PATH}"
fi
export KUBECONFIG="${KUBECONFIG_PATH}"

if ! command -v kubectl >/dev/null 2>&1; then
  if [[ -x "${K3S_BIN}" ]]; then
    echo "kubectl via k3s"
    sudo ln -sf "${K3S_BIN}" /usr/local/bin/kubectl 2>/dev/null || true
  fi
fi

if ! command -v helm >/dev/null 2>&1; then
  echo "Installing helm..."
  curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
fi

echo "Installing qemu-utils..."
sudo apt-get update -qq
sudo apt-get install -y qemu-utils curl

if ! command -v podman >/dev/null 2>&1 && ! command -v docker >/dev/null 2>&1; then
  echo "Installing podman..."
  sudo apt-get install -y podman
fi

sudo mkdir -p /var/lib/zyvor/images
sudo chmod 1777 /var/lib/zyvor/images

echo "Waiting for k3s node..."
for _ in $(seq 1 60); do
  if kubectl get nodes >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
kubectl get nodes -o wide
echo "=== k3s ready (KUBECONFIG=${KUBECONFIG}) ==="
