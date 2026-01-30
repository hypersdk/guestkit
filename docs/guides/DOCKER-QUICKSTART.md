# Docker Quick Start

One-page reference for running guestkit in Docker.

## Prerequisites

```bash
# Ensure Docker is running
systemctl is-active docker

# Load kernel modules (if not auto-loaded)
sudo modprobe nbd max_part=8
sudo modprobe loop
```

## Build

```bash
# Standard build
docker build -t guestkit:latest .

# With AI support
docker build --build-arg FEATURES="ai" -t guestkit:ai .
```

## Common Commands

### Inspect a Single VM

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  guestkit:latest inspect /vms/vm.qcow2
```

### Batch Process VMs

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  -v $(pwd)/output:/output \
  guestkit:latest inspect-batch /vms/*.qcow2 \
    --parallel 4 \
    --output json > output/results.json
```

### Security Profile

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  guestkit:latest profile security /vms/vm.qcow2
```

### Compare VMs

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  guestkit:latest diff \
    /vms/before.qcow2 \
    /vms/after.qcow2 \
    --output json
```

### Export Report

```bash
docker run --privileged \
  -v $(pwd)/vms:/vms:ro \
  -v $(pwd)/output:/output \
  guestkit:latest inspect /vms/vm.qcow2 \
    --export /output/report.json
```

## Docker Compose

### Create compose environment

```bash
# Create directories
mkdir -p vms output

# Copy your VM images to vms/
cp /path/to/*.qcow2 vms/
```

### Run commands

```bash
# Inspect
docker-compose run guestkit inspect /vms/vm.qcow2

# Batch process
docker-compose run guestkit inspect-batch /vms/*.qcow2 --parallel 4

# Security profile
docker-compose run guestkit profile security /vms/vm.qcow2
```

## Makefile Shortcuts

```bash
# Build image
make -f Makefile.docker docker-build

# Test setup
make -f Makefile.docker docker-test

# Inspect specific VM
make -f Makefile.docker docker-inspect VM_FILE=vm.qcow2 VM_PATH=./vms

# Batch process
make -f Makefile.docker docker-batch VM_PATH=./vms

# Open shell in container
make -f Makefile.docker docker-shell

# Clean up
make -f Makefile.docker docker-clean
```

## Permission Options

### With sudo (simple)

```bash
sudo docker run --privileged ...
```

### Add user to docker group (persistent)

```bash
sudo usermod -aG docker $USER
newgrp docker  # or logout/login
docker run --privileged ...
```

### Specific capabilities (more restrictive)

```bash
docker run \
  --cap-add=SYS_ADMIN \
  --cap-add=MKNOD \
  --cap-add=SYS_MODULE \
  --device=/dev/nbd0 \
  --device=/dev/loop0 \
  -v ./vms:/vms:ro \
  guestkit:latest inspect /vms/vm.qcow2
```

## Environment Variables

```bash
docker run --privileged \
  -e RUST_LOG=debug \
  -e OPENAI_API_KEY=sk-... \
  -v ./vms:/vms:ro \
  guestkit:latest inspect /vms/vm.qcow2 --ai-analyze
```

## Volume Mounts

| Host | Container | Purpose | Mode |
|------|-----------|---------|------|
| `./vms` | `/vms` | VM images | `ro` |
| `./output` | `/output` | Reports | `rw` |
| `/dev` | `/dev` | Devices | `rw` |

## Troubleshooting

**"Cannot load nbd module"**
```bash
sudo modprobe nbd max_part=8
```

**"Permission denied"**
```bash
# Add --privileged or use sudo
sudo docker run --privileged ...
```

**"No such device"**
```bash
# Ensure NBD devices exist
ls -la /dev/nbd*
```

**Image build slow/fails**
```bash
# Free up space
docker system prune -a

# Check available disk space
df -h
```

## CI/CD Example

```yaml
# .github/workflows/vm-scan.yml
name: VM Security Scan
on: [push]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build guestkit
        run: docker build -t guestkit:ci .

      - name: Scan VMs
        run: |
          docker run --privileged \
            -v ${{ github.workspace }}/vms:/vms:ro \
            guestkit:ci profile security /vms/*.qcow2 \
              --output json > scan-results.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: security-scan
          path: scan-results.json
```

## Links

- Full documentation: [DOCKER.md](DOCKER.md)
- Test results: [DOCKER-TEST-RESULTS.md](DOCKER-TEST-RESULTS.md)
- Main README: [README.md](README.md)
