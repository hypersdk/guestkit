#!/usr/bin/env bash
# Deploy Zyvor VM Services to a k3s host (build with podman, helm install).
# Run ON the remote host after guestkit sources are synced.
#
# Usage:
#   bash deploy/scripts/deploy-remote-k3s.sh
#   ROOT=/path/to/guestkit bash deploy/scripts/deploy-remote-k3s.sh
#   PULL_REGISTRY=ghcr.io/hypersdk bash deploy/scripts/deploy-remote-k3s.sh
set -euo pipefail

ROOT="${ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
NAMESPACE="${NAMESPACE:-zyvor}"
K3S_BIN="${K3S_BIN:-/usr/local/bin/k3s}"
BUILDER="${BUILDER:-podman}"
PULL_REGISTRY="${PULL_REGISTRY:-}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
NO_CACHE="${NO_CACHE:-}"

cd "${ROOT}"

if ! command -v "${BUILDER}" >/dev/null; then
  echo "ERROR: ${BUILDER} not found (install podman or set BUILDER=docker)"
  exit 1
fi
if ! command -v helm >/dev/null; then
  echo "ERROR: helm not found"
  exit 1
fi
if ! command -v kubectl >/dev/null; then
  echo "ERROR: kubectl not found"
  exit 1
fi

echo "=== Zyvor k3s deploy (root=${ROOT}) ==="

sudo mkdir -p /var/lib/zyvor/images
sudo chmod 1777 /var/lib/zyvor/images

build_and_import() {
  local name="$1"
  local dockerfile="$2"
  local context="$3"
  local tar="${HOME}/zyvor-${name//:/-}.tar"
  echo "Building ${name}..."
  local cache_flag=()
  if [[ -n "${NO_CACHE}" && "${name}" == zyvor-api ]]; then
    cache_flag=(--no-cache)
    echo "  (no-cache rebuild for zyvor-api)"
  fi
  "${BUILDER}" build --format docker "${cache_flag[@]}" -t "${name}:latest" -f "${dockerfile}" "${context}"
  echo "Exporting ${name}..."
  rm -f "${tar}"
  (cd /tmp && "${BUILDER}" save "${name}:latest" -o "${tar}")
  echo "Importing ${name} into k3s..."
  sudo "${K3S_BIN}" ctr -n k8s.io images import "${tar}"
  # k3s resolves unqualified image names to docker.io/library/<name>
  local base="${name%%:*}"
  local tag="${name#*:}"
  if [[ "${base}" == "${tag}" ]]; then
    tag="latest"
  fi
  local local_ref="localhost/${base}:${tag}"
  local k8s_ref="docker.io/library/${base}:${tag}"
  sudo "${K3S_BIN}" ctr -n k8s.io images tag "${local_ref}" "${k8s_ref}" 2>/dev/null \
    || sudo "${K3S_BIN}" ctr -n k8s.io images tag "${base}:${tag}" "${k8s_ref}" 2>/dev/null \
    || true
  rm -f "${tar}"
}

if [[ -n "${PULL_REGISTRY}" ]]; then
  WORKER_IMAGE="${PULL_REGISTRY}/guestkit-worker:${IMAGE_TAG}"
  API_IMAGE="${PULL_REGISTRY}/zyvor-api:${IMAGE_TAG}"
  UI_IMAGE="${PULL_REGISTRY}/zyvor-ui:${IMAGE_TAG}"
  echo "Pulling published images from ${PULL_REGISTRY}..."
  for img in "${WORKER_IMAGE}" "${API_IMAGE}" "${UI_IMAGE}"; do
    sudo "${K3S_BIN}" ctr -n k8s.io images pull "${img}"
  done
else
  WORKER_IMAGE="guestkit-worker:latest"
  API_IMAGE="zyvor-api:latest"
  UI_IMAGE="zyvor-ui:latest"
  build_and_import guestkit-worker "${ROOT}/crates/guestkit-worker/Dockerfile" "${ROOT}"
  build_and_import zyvor-api "${ROOT}/crates/zyvor-api/Dockerfile" "${ROOT}"
  build_and_import zyvor-ui "${ROOT}/deploy/ui/Dockerfile" "${ROOT}/deploy/ui"
fi

if [[ -f "${ROOT}/deploy/crd/zeus-vmtools.yaml" ]]; then
  echo "Applying Zeus VM Tools CRDs..."
  kubectl apply -f "${ROOT}/deploy/crd/zeus-vmtools.yaml"
fi

echo "Installing Helm chart..."
helm upgrade --install zyvor "${ROOT}/deploy/helm/zyvor" \
  -n "${NAMESPACE}" --create-namespace \
  -f "${ROOT}/deploy/helm/zyvor/values-k3s.yaml" \
  --set guestkitWorker.image="${WORKER_IMAGE}" \
  --set zyvorApi.image="${API_IMAGE}" \
  --set zyvorUi.image="${UI_IMAGE}"

echo "Waiting for rollouts..."
kubectl -n "${NAMESPACE}" rollout status deployment/postgresql --timeout=180s
kubectl -n "${NAMESPACE}" rollout status deployment/redis --timeout=180s
kubectl -n "${NAMESPACE}" rollout status deployment/zyvor-api --timeout=300s
kubectl -n "${NAMESPACE}" rollout status deployment/guestkit-worker --timeout=300s
kubectl -n "${NAMESPACE}" rollout status deployment/zyvor-ui --timeout=180s

NODE_IP="$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}')"
API_PORT="$(kubectl -n "${NAMESPACE}" get svc zyvor-api -o jsonpath='{.spec.ports[0].nodePort}')"
UI_PORT="$(kubectl -n "${NAMESPACE}" get svc zyvor-ui -o jsonpath='{.spec.ports[0].nodePort}')"

echo ""
echo "=== Zyvor deployed ==="
kubectl -n "${NAMESPACE}" get pods
echo ""
echo "API:  http://${NODE_IP}:${API_PORT}/api/v1/health"
echo "UI:   http://${NODE_IP}:${UI_PORT}/"
echo ""
echo "Smoke (empty image will fail doctor — use a real disk):"
echo "  API=http://${NODE_IP}:${API_PORT}/api/v1"
echo "  curl -sf \"\$API/health\""
