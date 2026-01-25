# Enhanced HTML Export Guide

GuestCtl now generates professional, interactive HTML reports with modern features for VM inspection results.

## Features

### 🎨 Modern UI/UX
- Clean, professional design with gradient headers
- Responsive layout works on all devices
- Smooth animations and transitions
- Icons for better visual navigation

### 🌓 Dark Mode
- Toggle between light and dark themes
- Persistent theme selection (saves to localStorage)
- Charts automatically update colors
- Optimized for both day and night viewing

### 🔍 Search Functionality
- Real-time search across all tables
- Search packages, services, users simultaneously
- Instant filtering with highlighting
- Case-insensitive matching

### 📊 Interactive Charts
- **Chart.js Integration** for data visualization
- Services distribution pie chart
- Package statistics bar chart
- Automatically adapts to theme changes
- Responsive and mobile-friendly

### 📋 Collapsible Sections
- Click any section header to expand/collapse
- Visual indicators (▼/▶) show state
- All sections expanded by default
- State persists during session

### 📱 Responsive Design
- Mobile-friendly layout
- Adapts to tablet and desktop screens
- Touch-friendly controls
- Print-optimized styles

## Usage

### Basic HTML Export

```bash
# Export inspection results to HTML
guestctl inspect vm.qcow2 --export html --export-output report.html

# With caching for faster subsequent exports
guestctl inspect vm.qcow2 --export html --export-output report.html --cache
```

### Advanced Usage

```bash
# Inspect with profile and export to HTML
guestctl inspect vm.qcow2 --profile security --export html --export-output security-report.html

# Batch inspection with HTML export
guestctl inspect-batch vm*.qcow2 --parallel 4 --cache --output json > results.json
# Then convert individual results to HTML
```

## Report Sections

### 📊 Summary Cards
Quick overview at the top showing:
- Operating System and distribution
- Version and architecture
- Package count
- Service count
- User count

### 💻 Operating System
Detailed system information:
- Hostname
- Product name
- Package format
- Package manager

### 📦 Packages
Interactive package list with:
- Package names
- Versions
- Visual chart showing distribution
- Searchable table
- Limited to top 100 packages for performance

### ⚙️ Services
System services information:
- Service names
- Service states
- Visual pie chart
- Color-coded status badges

### 👥 Users
User account details:
- Usernames
- UIDs
- Home directories
- Searchable list

### 🌐 Network Interfaces
Network configuration:
- Interface names
- IP addresses
- MAC addresses

## Keyboard Shortcuts

- **Search**: Click search box or start typing
- **Print**: Ctrl+P / Cmd+P (all sections auto-expand)
- **Toggle Theme**: Click "Toggle Theme" button

## Browser Compatibility

### Supported Browsers
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Modern mobile browsers

### Required Features
- JavaScript (for interactivity)
- CSS Grid support
- LocalStorage (for theme persistence)
- Canvas API (for charts)

## Customization

### Theme Colors
The report uses CSS custom properties (variables) for easy theming:

```css
:root {
    --accent-primary: #667eea;  /* Primary brand color */
    --accent-secondary: #764ba2; /* Secondary brand color */
}
```

### Chart.js CDN
Charts are powered by Chart.js v4.4.0 from CDN:
```html
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
```

**Note**: Requires internet connection for initial load. Charts will not display if offline.

## Performance

### Optimizations
- Lazy chart rendering (only when section visible)
- Limited to 100-200 items per table
- CSS transitions for smooth animations
- Minimal JavaScript footprint

### File Size
- Typical report: 50-200 KB
- With 100 packages: ~75 KB
- Inline styles and scripts for portability

## Examples

### Security Audit Report
```bash
# Generate comprehensive security audit
guestctl inspect production-vm.qcow2 \
  --profile security \
  --export html \
  --export-output security-audit-2026-01-24.html \
  --cache
```

### Fleet Comparison
```bash
# Inspect multiple VMs
for vm in web*.qcow2; do
  guestctl inspect "$vm" \
    --export html \
    --export-output "reports/$(basename $vm .qcow2)-report.html"
done
```

### CI/CD Integration
```yaml
# GitHub Actions example
- name: Inspect VM Image
  run: |
    guestctl inspect build/image.qcow2 \
      --export html \
      --export-output inspection-report.html

- name: Upload HTML Report
  uses: actions/upload-artifact@v3
  with:
    name: inspection-report
    path: inspection-report.html
```

## Troubleshooting

### Charts Not Displaying
**Problem**: Pie charts and bar charts don't appear
**Solution**: Check internet connection - Chart.js loads from CDN

### Theme Not Saving
**Problem**: Dark mode doesn't persist after refresh
**Solution**: Enable localStorage in browser settings

### Search Not Working
**Problem**: Search box doesn't filter results
**Solution**: Ensure JavaScript is enabled in browser

### Print Issues
**Problem**: Report doesn't print correctly
**Solution**: Use browser's print preview, all sections auto-expand

## Best Practices

### For Readability
1. Use dark mode for extended viewing
2. Collapse sections not currently needed
3. Use search to find specific items quickly

### For Sharing
1. Include timestamp in filename
2. Export with descriptive names
3. Consider PDF export for archival

### For Compliance
1. Include in audit trail
2. Archive HTML reports with VM snapshots
3. Use with security profile for compliance checks

## Future Enhancements

Planned features:
- Offline Chart.js bundling
- PDF export from HTML
- Custom branding/logos
- Comparison view for multiple VMs
- Filesystem usage charts
- Network topology diagrams

## Support

For issues or feature requests:
- GitHub: https://github.com/ssahani/guestkit/issues
- Documentation: See docs/OUTPUT_FORMATS.md

---

**Generated with GuestCtl v0.3.0** - Pure Rust VM Inspection Toolkit
