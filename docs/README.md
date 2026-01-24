# GuestKit Documentation

Welcome to the GuestKit documentation! This directory contains all project documentation organized by category.

## 📚 Documentation Structure

### 🎯 [Guides](guides/) - User-Facing Documentation

Start here if you're learning to use GuestKit!

- **[CLI Guide](guides/CLI_GUIDE.md)** - Command-line interface usage
- **[Interactive Mode](guides/INTERACTIVE_MODE.md)** - REPL for disk exploration
- **[Python Bindings](guides/PYTHON_BINDINGS.md)** - Python API guide
- **[Quick Start](guides/QUICKSTART.md)** - Get started quickly
- **[Output Formats](guides/OUTPUT_FORMATS.md)** - JSON, YAML, CSV output
- **[Inspection Profiles](guides/PROFILES_GUIDE.md)** - Security, migration, performance profiles
- **[Export Guide](guides/EXPORT_GUIDE.md)** - HTML and Markdown report export
- **[Comparison Guide](guides/COMPARISON_GUIDE.md)** - VM comparison and diff
- **[Quick Reference](guides/INSPECTION_QUICK_REF.md)** - Quick command reference
- **[PyPI Publishing](guides/PYPI_PUBLISHING.md)** - How to publish to PyPI
- **[Troubleshooting](guides/TROUBLESHOOTING.md)** - Common issues and solutions

### 📖 [API Documentation](api/) - API References

Complete API documentation for developers.

- **[Python API Reference](api/PYTHON_API_REFERENCE.md)** - Complete Python API (100+ methods)
- **[Rust API Reference](api/API_REFERENCE.md)** - Rust API documentation
- **[Ergonomic API](api/ERGONOMIC_API.md)** - Type-safe Rust API guide
- **[Migration Guide](api/MIGRATION_GUIDE.md)** - Migrating from libguestfs

### 🏗️ [Architecture](architecture/) - Technical Documentation

Understand how GuestKit works internally.

- **[Architecture Overview](architecture/ARCHITECTURE.md)** - System architecture
- **[libguestfs Comparison](architecture/LIBGUESTFS_COMPARISON.md)** - vs libguestfs
- **[Performance](architecture/PERFORMANCE.md)** - Performance characteristics
- **[UX Improvements](architecture/UX_IMPROVEMENTS.md)** - User experience enhancements

### 🔧 [Development](development/) - Contributor Documentation

For contributors and developers extending GuestKit.

- **[Enhancement Status](development/ENHANCEMENT_STATUS.md)** - Current enhancement status and priorities
- **[Next Enhancements](development/NEXT_ENHANCEMENTS.md)** - Detailed guides for next priority features
- **[Enhancement Roadmap](development/ENHANCEMENT_ROADMAP.md)** - Future enhancements (100+ ideas)
- **[Quick Enhancements](development/QUICK_ENHANCEMENTS.md)** - Quick wins to implement
- **[Enhancements Implemented](development/ENHANCEMENTS_IMPLEMENTED.md)** - What's been done
- **[Roadmap](development/ROADMAP.md)** - Project roadmap
- **[Missing APIs](development/MISSING_APIS.md)** - APIs not yet implemented

### 🧪 [Testing](testing/) - Testing Documentation

Testing guides and reports.

- **[Testing Guide](testing/TESTING.md)** - How to run tests
- **[Phase 3 Testing](testing/PHASE3_TESTING.md)** - Comprehensive test suite
- **[Testing Permissions](testing/TESTING_PERMISSIONS.md)** - Permission requirements
- **[Test Reports](testing/TEST_REPORT.md)** - Test results
- **[API Test Results](testing/API_TEST_RESULTS.md)** - API test coverage
- **[Test Status](testing/TEST_STATUS_SUMMARY.md)** - Current test status

**Distribution-Specific Testing:**
- [Ubuntu Testing](testing/UBUNTU_TESTING.md)
- [Debian Testing](testing/DEBIAN_TESTING.md)
- [Arch Linux Testing](testing/ARCH_TESTING.md)
- [Windows Testing](testing/WINDOWS_TESTING_REALISTIC.md)
- [Windows Test Summary](testing/WINDOWS_TESTING_SUMMARY.md)

### 📊 [Status](status/) - Implementation Status

Track implementation progress and project status.

- **[Project Summary](status/PROJECT_SUMMARY.md)** - Complete project overview
- **[Implementation Status](status/GUESTFS_IMPLEMENTATION_STATUS.md)** - 578 functions, 97.4% coverage
- **[Python Bindings Status](status/PYTHON_BINDINGS_STATUS.md)** - Python implementation status
- **[libguestfs Implementation](status/LIBGUESTFS_IMPLEMENTATION.md)** - libguestfs API coverage
- **[Implementation Complete](status/IMPLEMENTATION_COMPLETE.md)** - Completion summary

### 📦 [Archive](archive/) - Historical Documentation

Superseded or historical documents for reference.

- Previous completion summaries
- Old quick wins documentation
- Weekly completion reports
- Historical enhancement summaries

## 🚀 Quick Navigation

### New Users
1. Read [Quick Start](guides/QUICKSTART.md)
2. Check [CLI Guide](guides/CLI_GUIDE.md)
3. Try [Python Bindings](guides/PYTHON_BINDINGS.md)

### Python Developers
1. [Python Bindings Guide](guides/PYTHON_BINDINGS.md)
2. [Python API Reference](api/PYTHON_API_REFERENCE.md)
3. [Python Examples](../examples/python/)

### Rust Developers
1. [Architecture Overview](architecture/ARCHITECTURE.md)
2. [API Reference](api/API_REFERENCE.md)
3. [Ergonomic API Guide](api/ERGONOMIC_API.md)

### Contributors
1. [Enhancement Status](development/ENHANCEMENT_STATUS.md)
2. [Next Enhancements](development/NEXT_ENHANCEMENTS.md)
3. [Enhancement Roadmap](development/ENHANCEMENT_ROADMAP.md)
4. [Testing Guide](testing/TESTING.md)

### System Administrators
1. [CLI Guide](guides/CLI_GUIDE.md)
2. [Inspection Profiles](guides/PROFILES_GUIDE.md)
3. [Comparison Guide](guides/COMPARISON_GUIDE.md)

## 📝 Documentation Standards

All documentation follows these standards:

- **Format:** Markdown with GitHub-flavored extensions
- **Code Examples:** Tested and working
- **Updates:** Keep in sync with code changes
- **Links:** Use relative links within documentation

## 🔍 Search Tips

- **CLI Usage:** See [guides/CLI_GUIDE.md](guides/CLI_GUIDE.md)
- **Python API:** See [api/PYTHON_API_REFERENCE.md](api/PYTHON_API_REFERENCE.md)
- **Architecture:** See [architecture/ARCHITECTURE.md](architecture/ARCHITECTURE.md)
- **Testing:** See [testing/TESTING.md](testing/TESTING.md)
- **Status:** See [status/PROJECT_SUMMARY.md](status/PROJECT_SUMMARY.md)

## 🤝 Contributing to Documentation

Found an issue or want to improve documentation?

1. Check [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines
2. Submit issues at [GitHub Issues](https://github.com/ssahani/guestkit/issues)
3. Submit PRs for documentation improvements

## 📧 Support

- **Issues:** https://github.com/ssahani/guestkit/issues
- **Discussions:** https://github.com/ssahani/guestkit/discussions
- **Email:** ssahani@gmail.com

---

**Documentation Version:** 0.3.0
**Last Updated:** 2026-01-24
**License:** LGPL-3.0-or-later
