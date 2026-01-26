# Performance Validation Framework

This document describes the performance validation framework for guestctl, which helps validate performance improvements and detect regressions.

## Overview

The performance validation framework provides:
- **Mock VM disk generation** for consistent testing
- **Performance test suite** with multiple iterations
- **Baseline comparison** to detect regressions
- **Synthetic workload generation** for stress testing
- **Automated reporting** of performance metrics

## Components

### 1. Mock Disk Generator

Located in `tests/helpers/mock_disk.rs`, this module generates mock VM disk images with configurable characteristics.

#### Usage

```rust
use tests::helpers::{MockDiskBuilder, MockDiskType};

// Create a mock disk
let disk = MockDiskBuilder::new()
    .disk_type(MockDiskType::Small)  // 10MB disk
    .os_type("ubuntu")
    .hostname("test-vm")
    .num_files(100)
    .num_packages(200)
    .build("test.img")?;

// Access disk properties
assert_eq!(disk.os_type(), "ubuntu");
assert_eq!(disk.hostname(), "test-vm");
```

#### Disk Types

- **Minimal**: 1 MB - for basic functionality tests
- **Small**: 10 MB - for quick performance tests
- **Medium**: 100 MB - for realistic scenarios
- **Large**: 1 GB - for stress testing
- **Custom**: Any size - for specific requirements

### 2. Performance Test Suite

Located in `tests/performance_validation.rs`, this provides a framework for running and analyzing performance tests.

#### Usage

```rust
use tests::performance_validation::PerfTestSuite;

let mut suite = PerfTestSuite::new();

// Run a test with 10 iterations
suite.run_test("cache_lookup", 10, || {
    let start = Instant::now();
    // ... perform operation ...
    start.elapsed()
});

// Generate report
let report = suite.generate_report();
println!("{}", report);
```

#### Features

- **Multiple iterations**: Run tests multiple times for statistical significance
- **Automatic metrics**: Calculates average, min, max durations
- **Target validation**: Check if performance meets targets
- **Improvement tracking**: Compare against baselines

### 3. Workload Generator

Generates synthetic workloads for testing.

```rust
use tests::performance_validation::WorkloadGenerator;

let generator = WorkloadGenerator::new()?;

// Generate batch of identical disks
let disks = generator.generate_disk_batch(10, MockDiskType::Small)?;

// Generate varied batch
let varied = generator.generate_varied_batch(20)?;
```

### 4. Regression Detection

Compare current performance against baseline to detect regressions.

```rust
let baseline = PerfTestSuite::new();
// ... run baseline tests ...

let current = PerfTestSuite::new();
// ... run current tests ...

let comparison = current.compare_with_baseline(&baseline, 5.0);  // 5% threshold

if comparison.has_regressions() {
    println!("⚠️ Performance regressions detected!");
}

println!("{}", comparison.generate_report());
```

## Running Validation

### Automated Script

```bash
# Run full performance validation
./scripts/validate-performance.sh
```

This script:
1. Runs all performance validation tests
2. Compares against baseline (if exists)
3. Generates detailed report
4. Detects regressions

### Manual Testing

```bash
# Run validation tests
cargo test --release --test performance_validation

# Run with output
cargo test --release --test performance_validation -- --nocapture
```

## Performance Targets

Based on Q1 2026 goals, the following targets are defined:

| Operation | Baseline | Week 4 Target | Week 12 Target |
|-----------|----------|---------------|----------------|
| Cache lookup | 500ms | <100ms (✅) | <100ms |
| Appliance launch | 2500ms | <2000ms | <2000ms |
| OS inspection | 500ms | <400ms | <400ms |
| Package listing | 3500ms | <2800ms | <2800ms |
| Parallel batch (4 disks) | 4000ms | <1000ms (✅) | <1000ms |

## Baseline Management

### Creating Baseline

First run automatically creates baseline:

```bash
./scripts/validate-performance.sh
# Creates: performance-results/baseline.json
```

### Updating Baseline

After verifying improvements:

```bash
cp performance-results/current.json performance-results/baseline.json
git add performance-results/baseline.json
git commit -m "Update performance baseline"
```

### Regression Threshold

Default threshold: **5%**
- Improvements >5%: Reported as improvements
- Changes <5%: Reported as unchanged
- Regressions >5%: Reported as regressions (warning)

## Example Validation Report

```markdown
# Performance Validation Report

**Date:** 2026-01-26 10:30:00
**Commit:** abc1234

## Test Results

| Test Name | Iterations | Avg Duration | Min | Max |
|-----------|------------|--------------|-----|-----|
| cache_lookup | 10 | 85ms | 80ms | 95ms |
| parallel_batch | 10 | 950ms | 920ms | 980ms |

## Baseline Comparison

### ✅ Improvements

- cache_lookup: +83.0% faster (500ms → 85ms)
- parallel_batch: +76.3% faster (4000ms → 950ms)

### ❌ Regressions

None detected

**Overall improvement:** 79.7%
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Performance Validation
  run: |
    ./scripts/validate-performance.sh
    if [ $? -eq 2 ]; then
      echo "::warning::Performance changes detected"
    fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running performance validation..."
./scripts/validate-performance.sh

if [ $? -eq 2 ]; then
    echo "⚠️  Performance changes detected. Review before committing."
    read -p "Continue? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi
```

## Best Practices

1. **Run on dedicated hardware**: Consistent results require consistent environment
2. **Multiple iterations**: Use at least 10 iterations for statistical significance
3. **Warm-up runs**: Discard first iteration to avoid cold-start effects
4. **Consistent load**: Close other applications during testing
5. **Document changes**: Note system changes that might affect performance
6. **Version baselines**: Keep baseline for each major version

## Troubleshooting

### Tests are flaky

- Increase iteration count
- Check for background processes
- Ensure sufficient system resources
- Use `--test-threads=1` for serial execution

### Baseline comparison fails

- Verify baseline file exists
- Check file format (should be valid JSON)
- Ensure tests haven't been renamed
- Update baseline if intentional

### Disk generation slow

- Use smaller disk types for quick tests
- Generate disks in parallel (for batch operations)
- Use tmpfs for faster I/O if available

## Future Enhancements

- [ ] Integration with criterion.rs for statistical analysis
- [ ] Automated baseline updates based on approval
- [ ] Historical performance tracking
- [ ] Performance graphs and visualizations
- [ ] Flamegraph integration for hotspot analysis
- [ ] Memory profiling integration
- [ ] Multi-platform testing

## References

- [Q1 2026 Implementation Plan](q1-2026-medium-term.md)
- [Performance Baseline](performance-baseline.md)
- [Enhancement Roadmap](enhancement-roadmap.md)
