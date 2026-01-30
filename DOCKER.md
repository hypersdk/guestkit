# Docker Deployment Guide

This guide covers running guestkit in containers for automation and batch processing workflows.

## Quick Start

### Build the Image

```bash
docker build -t guestkit:latest .
```

### Run a Simple Inspection

```bash
docker run --privileged \
  -v /path/to/vms:/vms:ro \
  -v $(pwd)/output:/output \
  guestkit:latest inspect /vms/vm.qcow2
```

### Using Docker Compose

```bash
# Create a vms directory with your VM images
mkdir -p vms output

# Run with docker-compose
docker-compose run guestkit inspect /vms/vm.qcow2 --output json > output/report.json
```

## Why Privileged Mode?

Guestkit requires privileged access because it:

1. **Loads kernel modules** - `nbd` and `loop` drivers
2. **Creates device nodes** - `/dev/nbd*` and `/dev/loop*`
3. **Mounts filesystems** - Inspects disk partitions without booting

### Security Considerations

**Privileged mode grants extensive access.** For production use:

- Run in isolated environments
- Use read-only volume mounts for VM images
- Limit network access
- Run as non-root user where possible (see below)

### Alternative: Minimal Capabilities

Instead of `--privileged`, you can use specific capabilities:

```bash
docker run \
  --cap-add=SYS_ADMIN \
  --cap-add=MKNOD \
  --cap-add=SYS_MODULE \
  --device=/dev/nbd0 \
  --device=/dev/loop0 \
  -v /path/to/vms:/vms:ro \
  guestkit:latest inspect /vms/vm.qcow2
```

**Note:** This is more restrictive but may require pre-loaded kernel modules on the host.

## Common Use Cases

### 1. Batch Inspection with JSON Output

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  -v $(pwd)/output:/output \
  guestkit:latest inspect-batch /vms/*.qcow2 \
    --parallel 4 \
    --output json > output/inventory.json
```

### 2. Security Profiling

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  guestkit:latest profile security /vms/production-vm.qcow2 \
    --export /output/security-report.json
```

### 3. VM Comparison

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  guestkit:latest diff \
    /vms/vm-before.qcow2 \
    /vms/vm-after.qcow2 \
    --output json
```

### 4. CI/CD Pipeline Integration

```yaml
# .github/workflows/vm-security-scan.yml
name: VM Security Scan

on:
  push:
    paths:
      - 'vm-images/**'

jobs:
  security-scan:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Build guestkit
        run: docker build -t guestkit:ci .

      - name: Run security profile
        run: |
          docker run --privileged \
            -v ${{ github.workspace }}/vm-images:/vms:ro \
            guestkit:ci profile security /vms/*.qcow2 \
              --output json > security-report.json

      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: security-report.json
```

### 5. REST API Wrapper

Create a simple API service:

```dockerfile
# Dockerfile.api
FROM guestkit:latest

RUN apt-get update && apt-get install -y python3 python3-pip
RUN pip3 install fastapi uvicorn

COPY api.py /app/api.py

EXPOSE 8000
CMD ["uvicorn", "app.api:app", "--host", "0.0.0.0", "--port", "8000"]
```

```python
# api.py
from fastapi import FastAPI, UploadFile
import subprocess
import json

app = FastAPI()

@app.post("/inspect")
async def inspect_vm(file: UploadFile):
    # Save uploaded VM image
    vm_path = f"/tmp/{file.filename}"
    with open(vm_path, "wb") as f:
        f.write(await file.read())

    # Run guestkit
    result = subprocess.run(
        ["guestctl", "inspect", vm_path, "--output", "json"],
        capture_output=True,
        text=True
    )

    return json.loads(result.stdout)
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (error, warn, info, debug, trace) |
| `OPENAI_API_KEY` | - | OpenAI API key for AI diagnostics (optional) |
| `GUESTKIT_CACHE_DIR` | `/cache` | Cache directory for inspection results |
| `GUESTKIT_CONFIG_DIR` | `/config` | Configuration directory |

## Volume Mounts

| Host Path | Container Path | Purpose | Mode |
|-----------|----------------|---------|------|
| `./vms` | `/vms` | VM disk images | `ro` (read-only) |
| `./output` | `/output` | Export reports | `rw` |
| `/dev` | `/dev` | Device access | `rw` (required) |
| Named volume | `/cache` | Inspection cache | `rw` |
| Named volume | `/config` | TUI config | `rw` |

## Caching for Performance

Enable caching for repeated inspections:

```bash
docker-compose run guestkit inspect-batch /vms/*.qcow2 --cache --parallel 4
```

Cache is persisted in the `guestkit-cache` Docker volume.

## Kubernetes Deployment

For Kubernetes, you'll need a privileged pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: guestkit-job
spec:
  containers:
  - name: guestkit
    image: guestkit:latest
    securityContext:
      privileged: true
    volumeMounts:
    - name: vms
      mountPath: /vms
      readOnly: true
    - name: output
      mountPath: /output
    - name: dev
      mountPath: /dev
    command: ["guestctl", "inspect-batch", "/vms/*.qcow2", "--parallel", "4"]
  volumes:
  - name: vms
    hostPath:
      path: /path/to/vms
  - name: output
    emptyDir: {}
  - name: dev
    hostPath:
      path: /dev
  restartPolicy: Never
```

## Limitations

### Interactive Features Not Recommended

These features work better natively than in containers:

- **TUI Dashboard** - Terminal handling is complex in containers
- **Interactive Shell** - Better user experience natively
- **Fuzzy navigation** - Keyboard input may be problematic

### Host Kernel Dependencies

The container relies on the **host kernel** for:
- NBD module (`nbd.ko`)
- Loop device support

Ensure these are available on the host:

```bash
# On host machine
sudo modprobe nbd max_part=8
sudo modprobe loop
```

## Troubleshooting

### "Cannot load nbd module"

**Cause:** NBD module not available on host kernel

**Solution:**
```bash
# On host
sudo modprobe nbd max_part=8
```

### "Permission denied" accessing /dev

**Cause:** Container not running in privileged mode

**Solution:** Add `--privileged` flag or proper capabilities

### "No such device /dev/nbd0"

**Cause:** NBD devices not created

**Solution:**
```bash
# On host, create NBD devices
for i in {0..15}; do
  sudo mknod /dev/nbd$i b 43 $i
done
```

### Build fails with "disk quota exceeded"

**Cause:** Insufficient disk space

**Solution:**
```bash
# Clean up Docker resources
docker system prune -a
```

## Production Recommendations

1. **Use specific image tags** - Don't use `:latest` in production
2. **Enable read-only rootfs** - Add `--read-only` flag where possible
3. **Resource limits** - Set CPU and memory constraints
4. **Network isolation** - Use `--network none` if network isn't needed
5. **Security scanning** - Scan images with tools like Trivy
6. **Log aggregation** - Collect logs via Docker logging drivers

## Building with Features

### With AI Support

```dockerfile
# In Dockerfile, change the build command
RUN cargo build --release --features ai --bin guestctl
```

### With Python Bindings

```dockerfile
# In Dockerfile, change the build command
RUN cargo build --release --features python-bindings --bin guestctl
```

## Alternatives to Docker

### Podman

Guestkit works with Podman (rootless or rootful):

```bash
podman build -t guestkit:latest .
podman run --privileged -v ./vms:/vms:ro guestkit:latest inspect /vms/vm.qcow2
```

### Singularity/Apptainer

For HPC environments:

```bash
singularity build guestkit.sif docker://guestkit:latest
singularity run --writable-tmpfs guestkit.sif inspect /vms/vm.qcow2
```

## Further Reading

- [README.md](README.md) - Main documentation
- [SECURITY.md](SECURITY.md) - Security guidelines
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
