# Guestkit Worker - Kubernetes Deployment Guide

Complete guide for deploying the guestkit distributed worker system to Kubernetes.

---

## 📋 Overview

Deploy the production-ready guestkit worker as a DaemonSet on Kubernetes, enabling distributed VM inspection and security profiling across your cluster.

---

## 🎯 Prerequisites

- **Kubernetes cluster** (k3d, minikube, EKS, GKE, AKS)
- **kubectl** configured and connected
- **Docker** for building images
- **Node labels** for targeting worker nodes

---

## 🚀 Quick Deployment (k3d)

### 1. Build and Deploy

```bash
# Automated deployment script
./scripts/deploy-to-k3d.sh --build
```

This will:
- ✅ Build the Docker image
- ✅ Import to k3d cluster
- ✅ Label worker nodes
- ✅ Deploy all resources
- ✅ Wait for pods to be ready
- ✅ Show deployment status

###2. Verify Deployment

```bash
# Check pods
sudo kubectl get pods -n guestkit-workers

# Expected output:
# NAME                     READY   STATUS    RESTARTS   AGE
# guestkit-worker-xxxxx    1/1     Running   0          30s
# guestkit-worker-yyyyy    1/1     Running   0          30s
```

---

## 📦 Manual Deployment

### Step 1: Build Docker Image

```bash
# From repository root
sudo docker build -f crates/guestkit-worker/Dockerfile -t guestkit-worker:latest .
```

### Step 2: Import to k3d (if using k3d)

```bash
sudo k3d image import guestkit-worker:latest -c hyper2kvm-test
```

### Step 3: Label Worker Nodes

```bash
# Label nodes where workers should run
sudo kubectl label nodes k3d-hyper2kvm-test-agent-0 guestkit.io/worker-enabled=true
sudo kubectl label nodes k3d-hyper2kvm-test-agent-1 guestkit.io/worker-enabled=true

# Verify labels
sudo kubectl get nodes --show-labels | grep guestkit
```

### Step 4: Deploy Resources

```bash
# Deploy with Kustomize
sudo kubectl apply -k k8s/

# Or deploy individually
sudo kubectl apply -f k8s/namespace.yaml
sudo kubectl apply -f k8s/serviceaccount.yaml
sudo kubectl apply -f k8s/configmap.yaml
sudo kubectl apply -f k8s/daemonset.yaml
```

### Step 5: Verify

```bash
# Check all resources
sudo kubectl get all -n guestkit-workers

# Check logs
sudo kubectl logs -n guestkit-workers -l app=guestkit-worker --tail=20
```

---

## 🔧 Configuration

### Worker Configuration (ConfigMap)

Edit `k8s/configmap.yaml`:

```yaml
data:
  worker.conf: |
    worker_id: "auto"           # Auto-generate from hostname
    pool: "default"             # Worker pool name
    max_concurrent: 4           # Max concurrent jobs
    jobs_dir: /var/lib/guestkit/jobs
    results_dir: /var/lib/guestkit/results
    job_timeout_seconds: 7200
    log_level: info
```

Apply changes:
```bash
sudo kubectl apply -f k8s/configmap.yaml
sudo kubectl rollout restart daemonset/guestkit-worker -n guestkit-workers
```

### Resource Limits

Edit `k8s/daemonset.yaml`:

```yaml
resources:
  requests:
    cpu: 500m        # Minimum CPU
    memory: 512Mi    # Minimum memory
  limits:
    cpu: 2           # Maximum CPU
    memory: 2Gi      # Maximum memory
```

### Node Selection

Control which nodes run workers:

```yaml
nodeSelector:
  guestkit.io/worker-enabled: "true"
```

---

## 💼 Submitting Jobs

### Method 1: Direct Copy

```bash
# Get pod name
POD=$(sudo kubectl get pods -n guestkit-workers -l app=guestkit-worker -o name | head -1)

# Create job file
cat > inspect-job.json <<'EOF'
{
  "version": "1.0",
  "job_id": "inspect-001",
  "operation": "guestkit.inspect",
  "payload": {
    "type": "guestkit.inspect.v1",
    "data": {
      "image": {
        "path": "/data/vm.qcow2",
        "format": "qcow2"
      },
      "options": {
        "include_packages": true,
        "include_services": true,
        "include_security": true
      }
    }
  }
}
EOF

# Copy to worker
sudo kubectl cp inspect-job.json guestkit-workers/${POD#pod/}:/var/lib/guestkit/jobs/

# Check result
sleep 10
sudo kubectl exec -n guestkit-workers ${POD#pod/} -- ls -la /var/lib/guestkit/results/
```

### Method 2: PersistentVolume (Production)

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: guestkit-jobs
  namespace: guestkit-workers
spec:
  accessModes:
  - ReadWriteMany
  resources:
    requests:
      storage: 10Gi
```

Mount in DaemonSet and submit jobs via NFS/CephFS.

---

## 📊 Monitoring

### View Logs

```bash
# All workers
sudo kubectl logs -n guestkit-workers -l app=guestkit-worker -f

# Specific worker
sudo kubectl logs -n guestkit-workers guestkit-worker-xxxxx -f

# Last 50 lines
sudo kubectl logs -n guestkit-workers guestkit-worker-xxxxx --tail=50
```

### Check Status

```bash
# Pods
sudo kubectl get pods -n guestkit-workers -o wide

# DaemonSet
sudo kubectl get daemonset -n guestkit-workers

# Describe pod
sudo kubectl describe pod -n guestkit-workers guestkit-worker-xxxxx

# Events
sudo kubectl get events -n guestkit-workers --sort-by='.lastTimestamp'
```

### Metrics (if Prometheus installed)

```yaml
# Add annotations to DaemonSet
spec:
  template:
    metadata:
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
```

---

## 🔍 Troubleshooting

### Pods Not Starting

**Check node labels:**
```bash
sudo kubectl get nodes --show-labels | grep guestkit
```

**Check events:**
```bash
sudo kubectl describe pod -n guestkit-workers guestkit-worker-xxxxx
```

**Check logs:**
```bash
sudo kubectl logs -n guestkit-workers guestkit-worker-xxxxx
```

### Image Pull Errors

**For k3d:**
```bash
# Re-import image
sudo k3d image import guestkit-worker:latest -c hyper2kvm-test

# Verify
sudo docker exec k3d-hyper2kvm-test-server-0 crictl images | grep guestkit
```

**For production:**
```bash
# Push to registry
sudo docker tag guestkit-worker:latest your-registry.com/guestkit-worker:latest
sudo docker push your-registry.com/guestkit-worker:latest

# Update DaemonSet
kubectl set image daemonset/guestkit-worker worker=your-registry.com/guestkit-worker:latest -n guestkit-workers
```

### Worker Not Processing Jobs

**Check worker is running:**
```bash
sudo kubectl exec -n guestkit-workers guestkit-worker-xxxxx -- pgrep -f guestkit-worker
```

**Check directories:**
```bash
sudo kubectl exec -n guestkit-workers guestkit-worker-xxxxx -- ls -la /var/lib/guestkit/jobs/
sudo kubectl exec -n guestkit-workers guestkit-worker-xxxxx -- ls -la /var/lib/guestkit/results/
```

**Check permissions:**
```bash
sudo kubectl exec -n guestkit-workers guestkit-worker-xxxxx -- ls -ld /var/lib/guestkit/jobs
```

### Insufficient Resources

**Check node resources:**
```bash
sudo kubectl top nodes
sudo kubectl describe node <node-name>
```

**Adjust resource requests:**
Edit `k8s/daemonset.yaml` and reduce requests.

---

## 📈 Scaling

### Add More Workers

```bash
# Label additional nodes
sudo kubectl label nodes <node-name> guestkit.io/worker-enabled=true

# Workers will automatically deploy
sudo kubectl get pods -n guestkit-workers -w
```

### Remove Workers

```bash
# Remove label from node
sudo kubectl label nodes <node-name> guestkit.io/worker-enabled-

# Pod will be terminated
```

### Horizontal Pod Autoscaling

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: guestkit-worker
  namespace: guestkit-workers
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: DaemonSet
    name: guestkit-worker
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 🔒 Security

### RBAC

The ServiceAccount has minimal permissions:
- ✅ Get/list nodes
- ✅ Get/list pods
- ❌ No write access to cluster resources

### Privileged Containers

Workers run as privileged (required for NBD/loop devices):

```yaml
securityContext:
  privileged: true
  capabilities:
    add:
    - SYS_ADMIN
    - SYS_RESOURCE
```

**Production:** Consider using:
- RuntimeClass for gVisor/Kata Containers
- Pod Security Policies
- Network Policies

### Secrets Management

For sensitive data:

```bash
# Create secret
sudo kubectl create secret generic guestkit-secrets \
  --from-literal=api-key=your-api-key \
  -n guestkit-workers

# Mount in DaemonSet
volumeMounts:
- name: secrets
  mountPath: /etc/secrets
  readOnly: true
volumes:
- name: secrets
  secret:
    secretName: guestkit-secrets
```

---

## 🏭 Production Checklist

- [ ] Use PersistentVolumes for job/result storage
- [ ] Configure resource limits based on workload
- [ ] Set up monitoring (Prometheus/Grafana)
- [ ] Enable logging (Fluentd/Loki)
- [ ] Implement network policies
- [ ] Use image registry (not local images)
- [ ] Configure backup for results
- [ ] Set up alerting for failures
- [ ] Document runbooks
- [ ] Test disaster recovery

---

## 🧹 Cleanup

### Delete All Resources

```bash
# Delete namespace (removes everything)
sudo kubectl delete namespace guestkit-workers

# Or use Kustomize
sudo kubectl delete -k k8s/
```

### Remove Node Labels

```bash
# Remove labels
sudo kubectl label nodes --all guestkit.io/worker-enabled-
```

### Delete Docker Images

```bash
# Local
sudo docker rmi guestkit-worker:latest

# k3d
sudo docker exec k3d-hyper2kvm-test-server-0 crictl rmi guestkit-worker:latest
```

---

## 📚 Architecture

```
┌──────────────────────────────────────────────────────┐
│              Kubernetes Cluster                      │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  Namespace: guestkit-workers                   │  │
│  │                                                │  │
│  │  ┌──────────────────────────────────────────┐  │  │
│  │  │  DaemonSet: guestkit-worker             │  │  │
│  │  │                                          │  │  │
│  │  │  Node: agent-0         Node: agent-1    │  │  │
│  │  │  ┌──────────────┐      ┌──────────────┐  │  │  │
│  │  │  │ Worker Pod 1 │      │ Worker Pod 2 │  │  │  │
│  │  │  │              │      │              │  │  │  │
│  │  │  │ /jobs/       │      │ /jobs/       │  │  │  │
│  │  │  │ /results/    │      │ /results/    │  │  │  │
│  │  │  │              │      │              │  │  │  │
│  │  │  │ Guestfs API  │      │ Guestfs API  │  │  │  │
│  │  │  └──────┬───────┘      └──────┬───────┘  │  │  │
│  │  │         │                     │          │  │  │
│  │  │         ├─────────────────────┤          │  │  │
│  │  │         ↓                     ↓          │  │  │
│  │  │  ┌─────────────────────────────────┐    │  │  │
│  │  │  │  /data (HostPath or PV)         │    │  │  │
│  │  │  │  └─ VMs (QCOW2, VMDK, etc.)     │    │  │  │
│  │  │  └─────────────────────────────────┘    │  │  │
│  │  └──────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

---

## 🔗 See Also

- [k8s/README.md](k8s/README.md) - Detailed Kubernetes documentation
- [QUICKSTART-REAL-INTEGRATION.md](QUICKSTART-REAL-INTEGRATION.md) - Local quickstart
- [PHASE-3-COMPLETE.md](PHASE-3-COMPLETE.md) - Integration details
- [COMPLETE-SYSTEM-SUMMARY.md](COMPLETE-SYSTEM-SUMMARY.md) - System overview

---

**Status:** 📦 Ready for Deployment

**Version:** 1.0.0 with real guestkit integration

---

*Deploy with confidence! 🚀*
