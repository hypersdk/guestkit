#!/usr/bin/env bash
# E2E smoke test for Zyvor API workflow
set -euo pipefail

API="${API:-http://localhost:8080/api/v1}"
IMAGE="${IMAGE:-}"

if [[ -z "${IMAGE}" ]]; then
  echo "Usage: IMAGE=./disk.qcow2 API=http://api.zyvor.local/api/v1 $0"
  exit 1
fi

echo "Importing ${IMAGE}..."
IMPORT=$(curl -sf -F "file=@${IMAGE}" "${API}/vms/import")
echo "${IMPORT}" | head -c 500
VM_ID=$(echo "${IMPORT}" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['id'])")
echo ""
echo "VM ID: ${VM_ID}"

echo "Doctor..."
DOCTOR=$(curl -sf -X POST "${API}/vms/${VM_ID}/doctor?target=kubevirt&explain=true")
JOB_ID=$(echo "${DOCTOR}" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['job_id'])")
echo "Job ID: ${JOB_ID}"

for i in $(seq 1 60); do
  JOB=$(curl -sf "${API}/jobs/${JOB_ID}" || true)
  STATUS=$(echo "${JOB}" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('live_status',{}).get('status','pending'))" 2>/dev/null || echo pending)
  echo "  poll ${i}: ${STATUS}"
  if [[ "${STATUS}" == "completed" ]]; then
    echo "${JOB}" | python3 -m json.tool | head -80
    break
  fi
  sleep 5
done

echo "Migration plan..."
curl -sf -X POST "${API}/vms/${VM_ID}/migration-plan?target=kubevirt" | python3 -m json.tool

echo "Provision YAML..."
curl -sf -X POST "${API}/vms/${VM_ID}/provision" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['yaml'][:2000])"

echo "Smoke test complete."
