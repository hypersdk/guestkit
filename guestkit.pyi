"""Type stubs for guestkit Python bindings

This file provides type hints for better IDE support and type checking.
"""

from typing import List, Dict, Any, Optional
from types import TracebackType

__version__: str

class Guestfs:
    """Main class for VM disk image inspection and manipulation"""

    def __init__(self) -> None:
        """Create a new Guestfs handle"""
        ...

    def __enter__(self) -> 'Guestfs':
        """Enter context manager"""
        ...

    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_value: Optional[BaseException],
        traceback: Optional[TracebackType]
    ) -> bool:
        """Exit context manager and cleanup"""
        ...

    # Drive operations
    def add_drive(self, filename: str) -> None:
        """Add a disk image (read-write)"""
        ...

    def add_drive_ro(self, filename: str) -> None:
        """Add a disk image (read-only)"""
        ...

    def launch(self) -> None:
        """Launch the appliance"""
        ...

    def shutdown(self) -> None:
        """Shutdown the appliance"""
        ...

    def set_verbose(self, verbose: bool) -> None:
        """Enable/disable verbose output"""
        ...

    # Inspection API
    def inspect_os(self) -> List[str]:
        """Inspect operating systems in the disk image"""
        ...

    def inspect_get_type(self, root: str) -> str:
        """Get OS type (e.g., 'linux', 'windows')"""
        ...

    def inspect_get_distro(self, root: str) -> str:
        """Get distribution name (e.g., 'ubuntu', 'fedora')"""
        ...

    def inspect_get_major_version(self, root: str) -> int:
        """Get major version number"""
        ...

    def inspect_get_minor_version(self, root: str) -> int:
        """Get minor version number"""
        ...

    def inspect_get_hostname(self, root: str) -> str:
        """Get hostname"""
        ...

    def inspect_get_arch(self, root: str) -> str:
        """Get architecture (e.g., 'x86_64', 'aarch64')"""
        ...

    def inspect_get_product_name(self, root: str) -> str:
        """Get product name"""
        ...

    def inspect_get_package_format(self, root: str) -> str:
        """Get package format (e.g., 'rpm', 'deb')"""
        ...

    def inspect_get_package_management(self, root: str) -> str:
        """Get package management tool (e.g., 'apt', 'dnf')"""
        ...

    def inspect_get_mountpoints(self, root: str) -> Dict[str, str]:
        """Get filesystem mountpoints"""
        ...

    def inspect_list_applications(self, root: str) -> List[Dict[str, Any]]:
        """List installed packages"""
        ...

    # Device operations
    def list_devices(self) -> List[str]:
        """List all block devices"""
        ...

    def list_partitions(self) -> List[str]:
        """List all partitions"""
        ...

    def blockdev_getsize64(self, device: str) -> int:
        """Get device size in bytes"""
        ...

    # Filesystem operations
    def vfs_type(self, device: str) -> str:
        """Get filesystem type"""
        ...

    def vfs_label(self, device: str) -> str:
        """Get filesystem label"""
        ...

    def vfs_uuid(self, device: str) -> str:
        """Get filesystem UUID"""
        ...

    def mount(self, device: str, mountpoint: str) -> None:
        """Mount a filesystem (read-write)"""
        ...

    def mount_ro(self, device: str, mountpoint: str) -> None:
        """Mount a filesystem (read-only)"""
        ...

    def umount(self, mountpoint: str) -> None:
        """Unmount a filesystem"""
        ...

    def umount_all(self) -> None:
        """Unmount all filesystems"""
        ...

    def sync(self) -> None:
        """Synchronize filesystem"""
        ...

    # File operations
    def read_file(self, path: str) -> bytes:
        """Read file contents as bytes"""
        ...

    def cat(self, path: str) -> str:
        """Read file contents as string"""
        ...

    def write(self, path: str, content: bytes) -> None:
        """Write bytes to file"""
        ...

    def exists(self, path: str) -> bool:
        """Check if path exists"""
        ...

    def is_file(self, path: str) -> bool:
        """Check if path is a regular file"""
        ...

    def is_dir(self, path: str) -> bool:
        """Check if path is a directory"""
        ...

    def ls(self, directory: str) -> List[str]:
        """List directory contents"""
        ...

    def download(self, remotefilename: str, filename: str) -> None:
        """Download file from guest to host"""
        ...

    def upload(self, filename: str, remotefilename: str) -> None:
        """Upload file from host to guest"""
        ...

    # Directory operations
    def mkdir(self, path: str) -> None:
        """Create directory"""
        ...

    def mkdir_p(self, path: str) -> None:
        """Create directory with parents"""
        ...

    def rm(self, path: str) -> None:
        """Remove file"""
        ...

    def rmdir(self, path: str) -> None:
        """Remove empty directory"""
        ...

    def rm_rf(self, path: str) -> None:
        """Remove directory recursively"""
        ...

    # Permissions
    def chmod(self, mode: int, path: str) -> None:
        """Change file permissions"""
        ...

    def chown(self, owner: int, group: int, path: str) -> None:
        """Change file owner and group"""
        ...

    # Stat
    def stat(self, path: str) -> Dict[str, int]:
        """Get file stat information"""
        ...

    def statvfs(self, path: str) -> Dict[str, int]:
        """Get filesystem statistics"""
        ...

    # Command execution
    def command(self, arguments: List[str]) -> str:
        """Execute a command in the guest"""
        ...

    def sh(self, command: str) -> str:
        """Execute shell command"""
        ...

    def sh_lines(self, command: str) -> List[str]:
        """Execute shell command and return lines"""
        ...

    # LVM operations
    def vgscan(self) -> None:
        """Scan for LVM volume groups"""
        ...

    def vgs(self) -> List[str]:
        """List volume groups"""
        ...

    def pvs(self) -> List[str]:
        """List physical volumes"""
        ...

    def lvs(self) -> List[str]:
        """List logical volumes"""
        ...

    # Archive operations
    def tar_in(self, tarfile: str, directory: str) -> None:
        """Extract tar archive into guest directory"""
        ...

    def tar_out(self, directory: str, tarfile: str) -> None:
        """Create tar archive from guest directory"""
        ...

    def tgz_in(self, tarfile: str, directory: str) -> None:
        """Extract compressed tar archive into guest directory"""
        ...

    def tgz_out(self, directory: str, tarfile: str) -> None:
        """Create compressed tar archive from guest directory"""
        ...

    # Checksum operations
    def checksum(self, csumtype: str, path: str) -> str:
        """Calculate file checksum"""
        ...


class DiskConverter:
    """Class for converting disk image formats"""

    def __init__(self) -> None:
        """Create a new disk converter instance"""
        ...

    def convert(
        self,
        source: str,
        output: str,
        format: str = "qcow2",
        compress: bool = False,
        flatten: bool = True
    ) -> Dict[str, Any]:
        """Convert disk image format

        Returns:
            Dictionary with conversion results including:
            - source_path: Source file path
            - output_path: Output file path
            - source_format: Detected source format
            - output_format: Output format
            - output_size: Output file size in bytes
            - duration_secs: Conversion duration
            - success: True if successful
            - error: Error message (if failed)
        """
        ...

    def detect_format(self, image: str) -> str:
        """Detect disk image format

        Returns:
            Format string (e.g., 'qcow2', 'raw', 'vmdk')
        """
        ...

    def get_info(self, image: str) -> Dict[str, Any]:
        """Get disk image metadata

        Returns:
            Dictionary with image information
        """
        ...
