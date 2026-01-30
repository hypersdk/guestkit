#!/bin/bash
set -e

# Load kernel modules if not already loaded
if [ -w /dev ]; then
    # Check if nbd module is loaded
    if ! lsmod | grep -q nbd; then
        echo "Loading NBD kernel module..."
        modprobe nbd max_part=8 || echo "Warning: Could not load nbd module"
    fi

    # Check if loop module is loaded
    if ! lsmod | grep -q loop; then
        echo "Loading loop kernel module..."
        modprobe loop || echo "Warning: Could not load loop module"
    fi
fi

# Execute guestctl with provided arguments
exec guestctl "$@"
