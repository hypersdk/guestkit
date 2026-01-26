# GuestKit TUI Development Plan

**Created:** 2026-01-27
**Status:** Active Development

---

## Current Status ✅

The TUI is **already substantially implemented** with:

### Implemented Features
- ✅ **12 Complete Views:**
  - Dashboard (system overview)
  - Network (interfaces, DNS)
  - Packages (package management)
  - Services (systemd services)
  - Databases (MySQL, PostgreSQL, MongoDB, Redis)
  - WebServers (Apache, Nginx)
  - Security (SELinux, AppArmor, firewall)
  - Issues (security findings)
  - Storage (fstab, LVM, RAID)
  - Users (accounts)
  - Kernel (modules, parameters)
  - Profiles (security, migration, performance, compliance, hardening)

- ✅ **UI Components:**
  - Header with view icons and descriptions
  - Stats bar (packages, services, users, risk summary, bookmarks)
  - Tab navigation
  - Content area
  - Footer with keyboard shortcuts
  - Splash screen (ASCII art, not yet integrated)

- ✅ **Features:**
  - Keyboard navigation (1-9 quick jump, Tab/Shift+Tab, arrows, PgUp/PgDn, Home/End)
  - Search/filter (/) with live results
  - Sort modes (Default, Name ↑, Name ↓)
  - Export menu (JSON, YAML, HTML, PDF)
  - Help overlay (h, F1)
  - Detail views (Enter)
  - Bookmarks (b) - max 20
  - Stats bar toggle (i)
  - Notifications
  - Profile tabs (← → to switch)

- ✅ **Color Theme:**
  - Coral-Terracotta Orange (Pantone 7416 C inspired)
  - Consistent color coding: Green (OK), Yellow (Warning), Red (Error/Critical)

---

## Phase 1: Testing & Bug Fixes (Week 1)

### Priority: HIGH ⭐⭐⭐⭐⭐

1. **Manual Testing**
   - Test all 12 views with real VM images
   - Verify keyboard shortcuts work correctly
   - Test search/filter functionality
   - Test export to all formats
   - Test on different terminal sizes
   - Test with various VM types (Ubuntu, CentOS, Windows)

2. **Bug Fixes**
   - Fix any crashes or panics
   - Handle edge cases (empty data, missing files)
   - Improve error messages
   - Fix scrolling issues if any

3. **Performance**
   - Profile startup time
   - Optimize data loading
   - Add caching where appropriate
   - Test with large datasets (1000+ packages, etc.)

---

## Phase 2: Enhancements (Week 2-3)

### Priority: HIGH ⭐⭐⭐⭐

### 2.1 Integrate Splash Screen
**Effort:** 0.5 days

- Show splash on startup (2-3 seconds or until data loads)
- Fade transition to main UI
- Make splash optional with --no-splash flag

### 2.2 Add Real-Time Updates
**Effort:** 1 day

- Background refresh every N seconds (configurable)
- Show "updated" indicator when data refreshes
- Allow manual refresh with 'r' key

### 2.3 Improve Export Functionality
**Effort:** 1-2 days

- Actually implement HTML export (currently "coming soon")
- Actually implement PDF export (currently "coming soon")
- Add export templates
- Support exporting specific views vs full report
- Add export history/recent exports

### 2.4 Enhanced Search
**Effort:** 1 day

- Regex support
- Case-insensitive toggle
- Search across all views vs current view only
- Highlight search matches
- Search history navigation (↑/↓)

### 2.5 View-Specific Features
**Effort:** 2-3 days

**Dashboard:**
- Add quick actions (jump to high-risk issues, etc.)
- Show recent changes/updates
- Add system health score

**Network:**
- Visualize network topology
- Show traffic patterns (if available)
- DNS query simulation

**Packages:**
- Group by category
- Show update availability
- Security vulnerability scanning

**Services:**
- Service dependency graph
- Start/stop simulation (read-only inspection)
- Service health indicators

**Security:**
- Risk score calculation
- Compliance percentage
- Quick fix recommendations

**Issues:**
- Filter by severity
- Group by category
- Show remediation steps

### 2.6 Mouse Support
**Effort:** 1 day

- Click to switch tabs
- Click to select items
- Scroll with mouse wheel
- Hover tooltips

---

## Phase 3: Advanced Features (Week 4+)

### Priority: MEDIUM ⭐⭐⭐

### 3.1 Charts & Graphs
**Effort:** 2-3 days

- Package distribution chart (ratatui-charts)
- Service status pie chart
- Network bandwidth graph
- Disk usage visualization
- Security risk radar chart

### 3.2 Comparison Mode
**Effort:** 2-3 days

- Load multiple VM images
- Side-by-side comparison
- Diff highlighting
- Export comparison report

### 3.3 Plugin System
**Effort:** 3-5 days

- Custom views
- Custom profiles
- Custom export formats
- Plugin discovery

### 3.4 Remote Inspection
**Effort:** 3-5 days

- SSH into running VM
- Live system inspection
- Real-time monitoring
- Performance metrics

---

## Phase 4: Polish & Documentation (Ongoing)

### Priority: MEDIUM ⭐⭐⭐

1. **User Experience**
   - Add loading indicators for slow operations
   - Improve error messages with suggestions
   - Add confirmation dialogs for destructive actions
   - Keyboard shortcut cheat sheet (printable)

2. **Documentation**
   - TUI user guide
   - Screenshots/GIFs for README
   - Video demo
   - Keyboard shortcuts poster

3. **Accessibility**
   - Support screen readers
   - High contrast mode
   - Configurable colors
   - Font size adjustment

4. **Configuration**
   - Config file (~/.config/guestkit/tui.toml)
   - Customize colors
   - Customize keybindings
   - Save window state

---

## Quick Wins (1-2 hours each)

1. Add status line with current time
2. Add "loading..." animation
3. Add "press any key" on errors
4. Add copy-to-clipboard (if terminal supports)
5. Add view descriptions in help
6. Add view counts in tabs (e.g., "Services (42)")
7. Add filter indicators in footer
8. Add vim keybindings (j/k for up/down, etc.)

---

## Testing Plan

### Unit Tests
- Test view rendering
- Test keyboard input handling
- Test search/filter logic
- Test sort functions
- Test export functions

### Integration Tests
- Test full workflow (startup → navigate → export → quit)
- Test with various VM images
- Test error conditions
- Test edge cases

### Performance Tests
- Startup time < 5 seconds
- Navigation latency < 100ms
- Search results < 500ms
- Export < 2 seconds

---

## Code Organization

```
src/cli/tui/
├── mod.rs              # Main TUI entry point
├── app.rs              # Application state
├── ui.rs               # UI orchestration
├── events.rs           # Event handling
├── splash.rs           # Splash screen
└── views/
    ├── mod.rs
    ├── dashboard.rs
    ├── network.rs
    ├── packages.rs
    ├── services.rs
    ├── databases.rs
    ├── webservers.rs
    ├── security.rs
    ├── issues.rs
    ├── storage.rs
    ├── users.rs
    ├── kernel.rs
    └── profiles.rs
```

---

## Dependencies

Current:
- ratatui = "0.28" ✅
- crossterm = "0.28" ✅

Potential additions:
- ratatui-charts (for graphs)
- tui-realm (for component framework)
- arboard (for clipboard support)

---

## Success Metrics

### Week 1
- ✅ All views working without crashes
- ✅ Export to all 4 formats
- ✅ Search functioning correctly
- ✅ Documentation updated

### Week 2
- ✅ Splash screen integrated
- ✅ Real-time updates working
- ✅ Enhanced search with regex
- ✅ Mouse support

### Week 3
- ✅ Charts/graphs rendering
- ✅ View-specific enhancements
- ✅ Performance optimized
- ✅ User guide complete

---

## Next Steps

1. **TODAY:** Test TUI with real VM image
2. **THIS WEEK:** Fix bugs, integrate splash screen
3. **NEXT WEEK:** Add enhanced search and mouse support
4. **MONTH 1:** Complete all Phase 2 enhancements

---

**Focus:** Make the TUI the BEST way to inspect VM disks!
