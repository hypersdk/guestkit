# k9s Guide for Guestkit Worker

**Tool Version:** k9s v0.50.18
**Last Updated:** 2026-01-31

---

## Overview

k9s is a terminal-based UI for Kubernetes that makes it easier to navigate, observe, and manage your Kubernetes clusters. This guide shows how to use k9s with guestkit-worker deployments.

---

## Installation

### Already Installed

k9s is installed at: `/usr/local/bin/k9s`

**Version:**
```bash
k9s version
# v0.50.18
```

### Manual Installation (for reference)

```bash
curl -L https://github.com/derailed/k9s/releases/latest/download/k9s_Linux_amd64.tar.gz -o k9s.tar.gz
tar -xvf k9s.tar.gz
sudo mv k9s /usr/local/bin/
rm k9s.tar.gz LICENSE README.md
```

---

## Quick Start

### Launch k9s

```bash
# Connect to current context
k9s

# Connect to specific context
k9s --context prod-cluster

# Connect to specific namespace
k9s -n guestkit-worker

# Read-only mode
k9s --readonly
```

---

## Essential Key Bindings

### Navigation

| Key | Action |
|-----|--------|
| `?` | Show all keyboard shortcuts |
| `:` | Command mode |
| `/` | Filter/search |
| `Esc` | Exit current view/filter |
| `Ctrl-c` | Quit k9s |

### Common Views

| Command | Description |
|---------|-------------|
| `:pods` or `:po` | View pods |
| `:deployments` or `:deploy` | View deployments |
| `:services` or `:svc` | View services |
| `:namespaces` or `:ns` | View namespaces |
| `:nodes` | View nodes |
| `:configmaps` or `:cm` | View config maps |
| `:secrets` | View secrets |
| `:pv` | View persistent volumes |
| `:pvc` | View persistent volume claims |
| `:jobs` | View jobs |

### Pod Operations

When viewing pods:

| Key | Action |
|-----|--------|
| `Enter` | Describe pod |
| `l` | View logs |
| `p` | Previous logs |
| `c` | View container logs (if multiple) |
| `s` | Shell into pod |
| `d` | Delete pod |
| `y` | View YAML |
| `e` | Edit resource |
| `t` | Port forward |
| `Shift-f` | Show port forwards |

### Log Viewing

| Key | Action |
|-----|--------|
| `0` | Show all logs |
| `1` | Show last 100 lines |
| `2` | Show last 200 lines |
| `3` | Show last 500 lines |
| `4` | Show last 1000 lines |
| `f` | Follow logs (auto-refresh) |
| `w` | Toggle line wrap |
| `/` | Filter logs |
| `Ctrl-s` | Save logs to file |

---

## Using k9s with Guestkit Worker

### 1. View Worker Pods

```bash
k9s -n guestkit-worker
```

In k9s:
```
:pods
# or just :po
```

**Filter by label:**
```
/app=guestkit-worker
```

### 2. Check Worker Logs

1. Navigate to pods (`:pods`)
2. Select a worker pod (arrow keys)
3. Press `l` for logs
4. Press `f` to follow logs in real-time
5. Press `/` to filter logs (e.g., `/error`, `/job`)

### 3. Monitor Worker Resources

```bash
# In k9s, press Shift-c to toggle CPU/Memory columns
:pods
```

Watch for:
- **CPU usage** - Should be reasonable for active workers
- **Memory** - Monitor for leaks
- **Restarts** - High restart count indicates issues

### 4. View Worker Configuration

1. Navigate to configmaps: `:cm`
2. Select worker config
3. Press `Enter` to view details
4. Press `y` to see YAML

### 5. Check Service Endpoints

```bash
:svc
# Find guestkit-worker-api service
# Press Enter to see endpoints
```

### 6. Port Forwarding to Worker API

1. Navigate to services: `:svc`
2. Select `guestkit-worker-api`
3. Press `Shift-f` (port forward)
4. Access API at `http://localhost:8080`

Or manually:
```bash
# In k9s, on the service
# Press 't' for port forward
# Enter local port: 8080
# Enter remote port: 8080
```

### 7. Exec into Worker Pod

1. Navigate to pods: `:po`
2. Select worker pod
3. Press `s` for shell
4. Choose container (if multiple)
5. Shell opens in bottom pane

**Useful commands in worker pod:**
```bash
# Check worker binary
/usr/local/bin/guestkit-worker --version

# Check metrics endpoint
curl localhost:9090/metrics

# Check API endpoint
curl localhost:8080/api/v1/health

# View config
cat /etc/guestkit-worker/config.yaml

# Check logs
ls -la /var/log/guestkit-worker/
```

---

## Common Workflows

### Debugging Worker Issues

**1. Check pod status:**
```
:pods
/guestkit-worker
```

**2. View recent events:**
```
# Select pod and press Enter
# Scroll to Events section
```

**3. Check logs for errors:**
```
# Press 'l' on pod
# Press '/' and type 'error'
```

**4. Compare with working pod:**
```
# View multiple pods side-by-side
# Press 'l' on first pod
# Open second terminal with k9s
# View logs of working pod
```

### Monitoring Deployments

**1. Watch deployment rollout:**
```
:deploy
# Select guestkit-worker
# Watch ReplicaSet changes
```

**2. Check rollout status:**
```
# In deployment view
# Press Enter for details
# Look at "Replicas" section
```

**3. Rollback if needed:**
```
# In deployment view
# Press 'r' for rollback
```

### Scaling Workers

**1. Quick scale:**
```
:deploy
# Select guestkit-worker
# Press 's' for scale
# Enter new replica count
```

**2. Watch scaling progress:**
```
:pods
/guestkit-worker
# Watch new pods come up
```

### Resource Monitoring

**1. Check node resources:**
```
:nodes
# Press Shift-c to show CPU/Memory
```

**2. Find resource-heavy pods:**
```
:pods
# Sort by CPU: Shift-c then 'c'
# Sort by Memory: Shift-c then 'm'
```

**3. Check PVC usage:**
```
:pvc
# See storage usage
```

---

## Advanced Features

### Custom Views

Create `~/.config/k9s/views.yaml`:

```yaml
k9s:
  views:
    v1/pods:
      columns:
        - NAME
        - READY
        - STATUS
        - RESTARTS
        - CPU
        - MEM
        - AGE
```

### Aliases

Create `~/.config/k9s/aliases.yaml`:

```yaml
aliases:
  aliases:
    gw: v1/pods -l app=guestkit-worker
    api: v1/services/guestkit-worker-api
    metrics: v1/services/guestkit-worker-metrics
```

Usage:
```
:gw      # View guestkit-worker pods
:api     # Jump to API service
:metrics # Jump to metrics service
```

### Plugins

Create `~/.config/k9s/plugins.yaml`:

```yaml
plugins:
  # View worker metrics
  worker-metrics:
    shortCut: Ctrl-m
    description: View Prometheus metrics
    scopes:
      - pods
    command: bash
    background: false
    args:
      - -c
      - kubectl exec -it $NAME -- curl -s localhost:9090/metrics | less

  # Test worker health
  worker-health:
    shortCut: Ctrl-h
    description: Check worker health
    scopes:
      - pods
    command: bash
    background: false
    args:
      - -c
      - kubectl exec -it $NAME -- curl -s localhost:8080/api/v1/health | jq
```

---

## Configuration

### k9s Config Location

```
~/.config/k9s/
├── config.yaml       # Main configuration
├── views.yaml        # Custom views
├── aliases.yaml      # Command aliases
├── plugins.yaml      # Custom plugins
└── skins/            # Color schemes
```

### Basic Config

`~/.config/k9s/config.yaml`:

```yaml
k9s:
  # Refresh rate (seconds)
  refreshRate: 2

  # Max logs lines
  maxConnRetry: 5

  # Read-only mode
  readOnly: false

  # Disable mouse support
  noIcons: false

  # Log level (debug, info, warn, error, fatal, panic)
  logger:
    tail: 100
    buffer: 5000
    sinceSeconds: -1
    fullScreenLogs: false
    textWrap: false
    showTime: false

  # Current context/namespace
  currentContext: guestkit-cluster
  currentCluster: guestkit-cluster

  # Screen dump directory
  screenDumpDir: /tmp/k9s-screen-dumps
```

---

## Troubleshooting

### k9s Won't Start

**Issue:** `error: You must be logged in to the server (Unauthorized)`

**Solution:**
```bash
# Check kubectl works
kubectl get pods

# Check current context
kubectl config current-context

# Set correct context
kubectl config use-context your-context

# Try k9s again
k9s
```

### Slow Performance

**Issue:** k9s is slow

**Solutions:**
1. Increase refresh rate in config:
   ```yaml
   refreshRate: 5  # Instead of 2
   ```

2. Filter to specific namespace:
   ```bash
   k9s -n guestkit-worker
   ```

3. Use read-only mode:
   ```bash
   k9s --readonly
   ```

### Can't Delete Resources

**Issue:** Delete operations fail

**Solutions:**
1. Check RBAC permissions:
   ```bash
   kubectl auth can-i delete pods
   ```

2. Use read-only mode if you shouldn't delete:
   ```bash
   k9s --readonly
   ```

### Logs Not Showing

**Issue:** Log view is empty

**Solutions:**
1. Check pod is running:
   ```
   :pods
   # STATUS should be Running
   ```

2. Try different container (if multi-container):
   ```
   # Press 'c' to switch containers
   ```

3. Check log level:
   ```
   # Press '0' to show all logs
   ```

---

## Guestkit-Worker Specific Tips

### Quick Health Check

```
# In k9s
:pods
/guestkit-worker
# Look for:
# - All pods Running
# - 0 Restarts (or very low)
# - Good CPU/Memory
```

### Monitor Job Processing

```
# In k9s, select worker pod
l  # View logs
f  # Follow logs
/job  # Filter for job-related logs
```

Look for:
```
Job submitted: job-01HX...
Processing job: job-01HX...
Job completed: job-01HX...
```

### Check API Availability

```
# In k9s
:svc
# Select guestkit-worker-api
# Press Enter
# Check Endpoints - should show pod IPs
```

### Monitor Metrics Collection

```
# In k9s
:svc
# Select guestkit-worker-metrics
# Press 't' for port-forward
# Port 9090 -> 9090
```

Then browse to: `http://localhost:9090/metrics`

---

## Tips & Tricks

### 1. Multi-View

Split terminal and run multiple k9s instances:
```bash
# Terminal 1: Watch pods
k9s -n guestkit-worker

# Terminal 2: Watch logs of specific pod
k9s -n guestkit-worker
# Navigate to pod, press 'l'
```

### 2. Quick Context Switching

```
# In k9s
:ctx
# Select context and press Enter
```

### 3. Filtering

```
# By label
/app=guestkit-worker

# By name
/worker-0

# By status
/Running
/Error

# Clear filter
Esc
```

### 4. Sorting

In any view:
- Press column header letter to sort
- Example in pods: `n` (name), `s` (status), `a` (age)

### 5. XRay View

Press `Ctrl-d` to toggle XRay mode - shows extended resource info

### 6. Pulse View

Shows real-time cluster activity and metrics

```
:pulse
```

---

## Resources

### Official Documentation
- **k9s GitHub**: https://github.com/derailed/k9s
- **k9s Docs**: https://k9scli.io

### Related Guestkit Docs
- [Kubernetes Deployment](K8S-DEPLOYMENT.md)
- [Worker Quickstart](quickstart.md)
- [CLI Guide](../CLI-GUIDE.md)

---

## Cheat Sheet

```
Navigation:
  ?          Show help
  :          Command mode
  /          Filter
  Esc        Back/clear

Common Views:
  :pods      Pods
  :deploy    Deployments
  :svc       Services
  :ns        Namespaces
  :nodes     Nodes

Pod Actions:
  l          Logs
  s          Shell
  d          Delete
  y          YAML
  Enter      Describe

Logs:
  0-4        Show last N lines
  f          Follow
  w          Wrap
  /          Filter

Management:
  Ctrl-a     Show all resources
  Ctrl-d     XRay mode
  Ctrl-c     Quit
```

---

**Happy k9s-ing with Guestkit Worker!** 🚀

For issues or questions, see [GitHub Issues](https://github.com/ssahani/guestkit/issues)
