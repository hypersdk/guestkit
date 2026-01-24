# PyPI Publishing Guide

This guide explains how to publish GuestKit to PyPI (Python Package Index).

## Prerequisites

### 1. PyPI Account Setup

Create accounts on both PyPI and TestPyPI:

1. **PyPI (Production):**
   - Go to https://pypi.org/account/register/
   - Verify your email
   - Enable 2FA (required for publishing)

2. **TestPyPI (Testing):**
   - Go to https://test.pypi.org/account/register/
   - Verify your email
   - Enable 2FA

### 2. API Tokens

Generate API tokens for automated publishing:

**For PyPI:**
1. Go to https://pypi.org/manage/account/token/
2. Create a new API token
3. Name it "guestkit-github-actions"
4. Select scope: "Entire account" (or "Project: guestkit" after first upload)
5. Copy the token (starts with `pypi-`)

**For TestPyPI:**
1. Go to https://test.pypi.org/manage/account/token/
2. Create a new API token
3. Name it "guestkit-github-actions-test"
4. Copy the token

### 3. Configure GitHub Secrets

⚠️ **Important:** The workflow now uses Trusted Publishing (OIDC), which is more secure than API tokens.

#### Option A: Trusted Publishing (Recommended)

1. Go to https://pypi.org/manage/account/publishing/
2. Add a new publisher:
   - **PyPI Project Name:** guestkit
   - **Owner:** ssahani
   - **Repository name:** guestkit
   - **Workflow name:** build-wheels.yml
   - **Environment name:** pypi

This allows GitHub Actions to publish without storing tokens!

#### Option B: Using API Tokens (Fallback)

If you can't use Trusted Publishing, add GitHub secrets:

1. Go to your GitHub repository settings
2. Navigate to Secrets and variables → Actions
3. Add these secrets:
   - `PYPI_API_TOKEN` - Your PyPI token
   - `TEST_PYPI_API_TOKEN` - Your TestPyPI token

## Local Testing

Before publishing, test the build locally:

### 1. Build Wheel for Your Platform

```bash
# Install maturin if not already installed
pip install maturin

# Set environment variable for Python 3.14 compatibility
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Build wheel
maturin build --release --features python-bindings

# The wheel will be in target/wheels/
ls -lh target/wheels/
```

### 2. Test Installation in Virtual Environment

```bash
# Create a fresh virtual environment
python3 -m venv test-env
source test-env/bin/activate

# Install from the wheel
pip install target/wheels/guestkit-*.whl

# Test import
python -c "from guestkit import Guestfs; print('Success!')"

# Test context manager
python -c "from guestkit import Guestfs; g = Guestfs(); print('Context manager works!')"

# Deactivate
deactivate
rm -rf test-env
```

### 3. Test with Different Python Versions

```bash
# Python 3.8
python3.8 -m venv test-env-38
source test-env-38/bin/activate
pip install target/wheels/guestkit-*.whl
python -c "from guestkit import Guestfs"
deactivate

# Python 3.11
python3.11 -m venv test-env-311
source test-env-311/bin/activate
pip install target/wheels/guestkit-*.whl
python -c "from guestkit import Guestfs"
deactivate

# Python 3.12
python3.12 -m venv test-env-312
source test-env-312/bin/activate
pip install target/wheels/guestkit-*.whl
python -c "from guestkit import Guestfs"
deactivate
```

## Publishing to TestPyPI

Test the full publication workflow using TestPyPI first:

### 1. Build Distribution

```bash
# Build wheel for your platform
maturin build --release --features python-bindings

# Build source distribution
maturin sdist
```

### 2. Upload to TestPyPI

```bash
# Install twine
pip install twine

# Upload to TestPyPI
twine upload --repository testpypi target/wheels/*

# You'll be prompted for username and password:
# Username: __token__
# Password: <your TestPyPI token starting with pypi->
```

### 3. Test Installation from TestPyPI

```bash
# Create fresh environment
python3 -m venv test-testpypi
source test-testpypi/bin/activate

# Install from TestPyPI
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple guestkit

# Test it works
python -c "from guestkit import Guestfs; print('TestPyPI install successful!')"

deactivate
rm -rf test-testpypi
```

## Publishing to PyPI (Production)

### Method 1: GitHub Actions (Recommended)

This is the easiest and most reliable method.

#### For Tagged Releases

```bash
# Make sure all changes are committed
git add -A
git commit -m "Prepare for v0.3.0 release"
git push

# Create and push a tag
git tag v0.3.0
git push origin v0.3.0
```

GitHub Actions will automatically:
1. Build wheels for Linux (x86_64, aarch64)
2. Build wheels for macOS (x86_64, aarch64)
3. Build source distribution
4. Run installation tests
5. Publish to PyPI

#### Manual Trigger

You can also trigger the workflow manually:

1. Go to GitHub Actions tab
2. Select "Build and Publish Python Wheels"
3. Click "Run workflow"
4. Select branch: main
5. Set "Publish to PyPI": true
6. Click "Run workflow"

### Method 2: Manual Upload

If you need to publish manually:

```bash
# Build everything
maturin build --release --features python-bindings
maturin sdist

# Upload to PyPI
twine upload target/wheels/*

# Username: __token__
# Password: <your PyPI token>
```

## Verification After Publishing

### 1. Check PyPI Page

Visit https://pypi.org/project/guestkit/ and verify:
- ✅ Correct version is listed
- ✅ Description renders correctly
- ✅ All wheels are present (Linux x86_64, aarch64, macOS x86_64, aarch64)
- ✅ Source distribution is available
- ✅ Project links work
- ✅ Classifiers are correct

### 2. Test Installation

```bash
# In a fresh environment
python3 -m venv verify-env
source verify-env/bin/activate

# Install from PyPI
pip install guestkit

# Verify it works
python -c "from guestkit import Guestfs; print('PyPI installation successful!')"
python -c "from guestkit import DiskConverter; print('DiskConverter available!')"

# Check version
pip show guestkit

deactivate
rm -rf verify-env
```

### 3. Test on Different Platforms

Ideally test on:
- ✅ Ubuntu 22.04 (x86_64)
- ✅ Fedora 39 (x86_64)
- ✅ macOS (Intel)
- ✅ macOS (Apple Silicon)

## Updating the Package

When you need to publish a new version:

### 1. Update Version Number

Edit `pyproject.toml`:
```toml
version = "0.3.1"  # Increment version
```

### 2. Update CHANGELOG.md

Add release notes for the new version.

### 3. Commit and Tag

```bash
git add pyproject.toml CHANGELOG.md
git commit -m "Bump version to 0.3.1"
git tag v0.3.1
git push origin main
git push origin v0.3.1
```

GitHub Actions will automatically build and publish!

## Troubleshooting

### Build Fails on Linux

**Error:** "Could not find OpenSSL"

**Solution:** The workflow installs OpenSSL, but if building locally:
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install openssl-devel
```

### Build Fails with Python Version Error

**Error:** "Python 3.14 is newer than PyO3's maximum supported version"

**Solution:**
```bash
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
maturin build --release --features python-bindings
```

### Upload Fails: "File already exists"

**Error:** "File already exists: guestkit-0.3.0-*.whl"

**Solution:** You can't re-upload the same version. Either:
- Use `--skip-existing` flag
- Increment the version number

### Import Fails After Installation

**Error:** "ImportError: cannot import name 'Guestfs'"

**Solution:**
1. Make sure you built with `--features python-bindings`
2. Check the wheel contains the binary module:
   ```bash
   unzip -l target/wheels/guestkit-*.whl | grep .so
   ```
3. Reinstall:
   ```bash
   pip uninstall guestkit
   pip install --no-cache-dir guestkit
   ```

### Trusted Publishing Not Working

**Error:** "OIDC token retrieval failed"

**Solution:**
1. Verify you've set up the publisher on PyPI
2. Check the workflow has the correct environment name
3. Make sure `id-token: write` permission is set in workflow

Fallback to API tokens if needed.

## Best Practices

### Version Numbering

Follow Semantic Versioning (SemVer):
- **Major (1.0.0):** Breaking changes
- **Minor (0.3.0):** New features, backward compatible
- **Patch (0.3.1):** Bug fixes

### Release Checklist

Before each release:
- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in pyproject.toml
- [ ] Build and test locally
- [ ] Test on TestPyPI first
- [ ] Create git tag
- [ ] Push tag to trigger build
- [ ] Verify on PyPI
- [ ] Test installation
- [ ] Announce release

### Security

- ✅ Always use 2FA on PyPI account
- ✅ Use Trusted Publishing when possible (no tokens stored)
- ✅ If using tokens, use scoped tokens (project-specific)
- ✅ Rotate tokens periodically
- ✅ Never commit tokens to git

## Automation

The GitHub Actions workflow handles everything automatically:

**On every tag push (v*):**
1. ✅ Builds wheels for all supported platforms
2. ✅ Builds source distribution
3. ✅ Tests installation on Linux and macOS
4. ✅ Publishes to PyPI
5. ✅ Creates GitHub release

**You just need to:**
```bash
git tag v0.3.1
git push origin v0.3.1
```

Done! 🎉

## Support

If you encounter issues:

1. Check workflow logs in GitHub Actions
2. Review PyPI project page for errors
3. Test locally first with the commands above
4. File an issue at https://github.com/ssahani/guestkit/issues

## Additional Resources

- **Maturin Docs:** https://www.maturin.rs/
- **PyPI Publishing:** https://packaging.python.org/
- **PyO3 Guide:** https://pyo3.rs/
- **GitHub Actions:** https://docs.github.com/en/actions
- **Trusted Publishing:** https://docs.pypi.org/trusted-publishers/

---

**Last Updated:** 2026-01-24
**Maintained By:** GuestKit Team
