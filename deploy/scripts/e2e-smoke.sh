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

echo "Inspect..."
INSPECT=$(curl -sf -X POST "${API}/vms/${VM_ID}/inspect")
INS_JOB=$(echo "${INSPECT}" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['job_id'])")
for i in $(seq 1 60); do
  JOB=$(curl -sf "${API}/jobs/${INS_JOB}" || true)
  STATUS=$(echo "${JOB}" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('data',{}).get('live_status',{}).get('status','pending'))" 2>/dev/null || echo pending)
  echo "  inspect poll ${i}: ${STATUS}"
  if [[ "${STATUS}" == "completed" ]]; then
    python3 -c "import sys,json; d=json.load(sys.stdin); r=d.get('data',{}).get('result',{}); assert r.get('inspect') or r.get('data',{}).get('inspect')" <<< "${JOB}"
    break
  fi
  sleep 5
done

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

echo "Config endpoint..."
curl -sf "${API}/config" | python3 -m json.tool | head -20

if curl -sf "${API}/kubevirt/vms" >/dev/null 2>&1; then
  echo "KubeVirt fleet..."
  curl -sf "${API}/kubevirt/vms" | python3 -m json.tool | head -40
  echo "KubeVirt namespaces..."
  curl -sf "${API}/kubevirt/namespaces" | python3 -m json.tool
fi

if curl -sf "${API}/vmtools/bundle" >/dev/null 2>&1; then
  echo "VM Tools bundle..."
  curl -sf "${API}/vmtools/bundle" | python3 -m json.tool | head -20
  echo "VM Tools coverage..."
  curl -sf "${API}/vmtools/coverage" | python3 -m json.tool
  echo "VM Tools policy..."
  curl -sf "${API}/vmtools/policy" | python3 -m json.tool
  echo "VM Tools reconcile..."
  RECON=$(curl -sf -X POST "${API}/vmtools/policy/reconcile" || true)
  echo "${RECON}" | python3 -m json.tool
  python3 -c "import sys,json; d=json.load(sys.stdin); r=d.get('data',{}); assert 'pending' in r and 'upgraded' in r" <<< "${RECON}"
fi

if curl -sf "${API}/kubevirt/vms" >/dev/null 2>&1; then
  echo "KubeVirt inspect job (first stopped Linux VM)..."
  STOPPED=$(curl -sf "${API}/kubevirt/vms" | python3 -c "
import sys,json
vms=json.load(sys.stdin).get('data',[])
for v in vms:
  phase=str(v.get('phase') or v.get('status') or '').lower()
  if not v.get('is_windows') and 'run' not in phase:
    print(v['namespace'], v['name'])
    break
" || true)
  if [[ -n "${STOPPED}" ]]; then
    NS=$(echo "${STOPPED}" | awk '{print $1}')
    NAME=$(echo "${STOPPED}" | awk '{print $2}')
    INS=$(curl -sf -X POST "${API}/kubevirt/vms/${NS}/${NAME}/inspect" || true)
    echo "${INS}" | python3 -m json.tool | head -10
  else
    echo "  (no stopped Linux VM — skip cluster inspect smoke)"
  fi
fi

if curl -sf "${API}/storage/roots" >/dev/null 2>&1; then
  echo "Storage roots..."
  curl -sf "${API}/storage/roots" | python3 -m json.tool | head -20
fi

echo "Smoke test complete."
