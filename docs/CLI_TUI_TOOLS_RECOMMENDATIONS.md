# CLI/TUI Tools Recommendations for LEAN REST API Wrapper Development

**Date**: 2025-11-18
**Source**: [Essential CLI/TUI Tools for Developers](https://www.freecodecamp.org/news/essential-cli-tui-tools-for-developers/)
**Purpose**: Identify relevant CLI/TUI tools for developing and testing the LEAN REST API wrapper

---

## Highly Relevant Tools for LEAN REST API Wrapper

### 🔥 **ATAC** - Terminal API Client (Postman Alternative)

**Why It's Perfect for This Project:**
- Test LEAN REST API wrapper endpoints without leaving terminal
- No GUI needed - works in headless environments
- Cross-platform support
- Perfect for testing `/api/v1/snapshot`, `/strategy/start`, `/strategy/stop` endpoints
- Can save request collections for regression testing

**Use Cases:**
- Testing LEAN REST API wrapper during development (T-50)
- Validating API contract compliance (`agents/shared/API_CONTRACT.md`)
- Quick API debugging without opening browser/Postman
- CI/CD integration for API testing

**Installation:**
```bash
# macOS
brew tap julien-cpsn/atac
brew install atac

# Arch Linux
pacman -S atac
```

**Integration with Tasks:**
- **T-50**: Use ATAC to test FastAPI endpoints during implementation
- **T-52**: Use ATAC to verify PWA/TUI integration endpoints

---

### 🔥 **httpie** - Modern HTTP Client

**Why It's Useful:**
- Human-friendly syntax (better than `curl`)
- JSON support (perfect for REST API testing)
- Colorized output (easier to read responses)
- Great for testing LEAN wrapper endpoints

**Use Cases:**
- Quick API endpoint testing
- Testing WebSocket connections
- Validating JSON responses
- Debugging API issues

**Example Usage for LEAN Wrapper:**
```bash
# Test snapshot endpoint
http GET http://localhost:8000/api/v1/snapshot

# Start strategy
http POST http://localhost:8000/api/v1/strategy/start

# Test with authentication
http GET http://localhost:8000/api/v1/account/summary Authorization:"Bearer token"
```

**Installation:**
```bash
# macOS
brew install httpie

# Debian
sudo apt install httpie

# Arch Linux
pacman -Syu httpie

# Windows
choco install httpie
```

---

### 🔥 **jq** - JSON Processor

**Why It's Essential:**
- Parse LEAN REST API JSON responses
- Filter and transform API data
- Extract specific fields from API responses
- Perfect for scripting and automation

**Use Cases:**
- Parse `/api/v1/snapshot` responses
- Extract specific metrics from API responses
- Transform LEAN data format to API contract format
- Debug JSON structure issues

**Example Usage:**
```bash
# Get snapshot and extract net_liq
http GET http://localhost:8000/api/v1/snapshot | jq '.metrics.net_liq'

# Filter positions by symbol
http GET http://localhost:8000/api/v1/snapshot | jq '.positions[] | select(.symbol == "SPY")'

# Count active orders
http GET http://localhost:8000/api/v1/snapshot | jq '.orders | length'
```

**Installation:**
```bash
# Download from GitHub releases
# https://github.com/jqlang/jq/releases
```

---

### 🔥 **k6** - Load Testing Tool

**Why It's Important:**
- Test LEAN REST API wrapper performance
- Load test WebSocket connections
- Validate API can handle concurrent requests
- Measure latency and throughput

**Use Cases:**
- Load testing LEAN wrapper endpoints
- Testing WebSocket bridge (T-51) under load
- Performance benchmarking
- Stress testing before production

**Example Test Script:**
```javascript
// load_test.js
import http from 'k6/http';
import { check } from 'k6';

export default function () {
  const res = http.get('http://localhost:8000/api/v1/snapshot');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });
}
```

**Installation:**
```bash
# macOS
brew install k6

# Debian
sudo apt-get install k6

# Windows
choco install k6
```

**Integration with Tasks:**
- **T-50**: Load test REST API wrapper after implementation
- **T-51**: Stress test WebSocket bridge with multiple concurrent connections

---

### 🔥 **btop** - Resource Monitor

**Why It's Useful:**
- Monitor LEAN REST API wrapper resource usage
- Track memory/CPU usage of LEAN + Python wrapper
- Identify performance bottlenecks
- Monitor during load testing

**Use Cases:**
- Monitor LEAN wrapper during development
- Debug performance issues
- Track resource usage during load tests
- Monitor multiple services (LEAN, REST wrapper, PWA backend)

**Installation:**
```bash
# macOS
brew install btop

# Debian (via snap)
sudo snap install btop
```

---

### 🔥 **tmux** or **zellij** - Terminal Multiplexer

**Why It's Essential:**
- Run multiple services simultaneously:
  - LEAN algorithm
  - REST API wrapper (FastAPI)
  - PWA backend
  - TUI client
- Monitor logs from multiple services
- Session persistence (survives disconnections)

**Use Cases:**
- Development workflow: Run LEAN + REST wrapper + TUI in separate panes
- Monitor logs from all services simultaneously
- Test complete system integration
- Remote development sessions

**Installation:**
```bash
# tmux (macOS)
brew install tmux

# zellij (macOS) - Modern alternative
brew install zellij

# Debian
sudo apt install tmux
```

**Recommended Setup:**
```
┌─────────────┬─────────────┐
│  LEAN Logs  │  REST API   │
│  (Python)   │  (FastAPI)  │
├─────────────┼─────────────┤
│  TUI Client │  PWA Dev    │
│  (C++)      │  (npm)      │
└─────────────┴─────────────┘
```

---

## Moderately Relevant Tools

### **bat** - Enhanced `cat` with Syntax Highlighting

**Why It's Useful:**
- View LEAN configuration files with syntax highlighting
- Read API response JSON files
- View Python code (FastAPI wrapper)
- Better than `cat` for code/config files

**Use Cases:**
- View `config/lean_config.json`
- Read API contract documentation
- View Python wrapper code
- Check LEAN log files

**Installation:**
```bash
# macOS
brew install bat

# Debian
sudo apt install bat

# Arch Linux
pacman -S bat
```

---

### **ripgrep** - Fast Text Search

**Why It's Useful:**
- Search codebase for API endpoint definitions
- Find references to LEAN wrapper code
- Search for API contract usage
- Faster than `grep` for large codebases

**Use Cases:**
- Find all REST endpoint definitions
- Search for LEAN API usage
- Find API contract references
- Search Python wrapper code

**Installation:**
```bash
# macOS
brew install ripgrep

# Debian
sudo apt-get install ripgrep

# Arch Linux
pacman -S ripgrep
```

---

### **asciinema** - Terminal Session Recorder

**Why It's Useful:**
- Record LEAN REST API wrapper demos
- Share API testing workflows
- Document integration procedures
- Create tutorials for PWA/TUI integration

**Use Cases:**
- Record LEAN wrapper API testing
- Create demo videos for documentation
- Share integration workflows
- Document troubleshooting procedures

**Installation:**
```bash
# macOS
brew install asciinema

# Debian
sudo apt install asciinema

# Arch Linux
sudo pacman -S asciinema
```

---

## Less Relevant (But Still Useful)

### **gping** - Ping with Graph

**Why It Might Be Useful:**
- Monitor network latency to LEAN wrapper
- Test WebSocket connection stability
- Monitor API response times visually
- Debug network issues

**Use Cases:**
- Monitor latency to LEAN wrapper server
- Test WebSocket connection quality
- Visualize API response times

**Installation:**
```bash
# macOS
brew install gping

# Windows
choco install gping
```

---

## Recommended Tool Stack for LEAN REST API Wrapper Development

### Essential (Must Have)
1. **ATAC** - API testing (replaces Postman in terminal)
2. **httpie** - Quick API testing
3. **jq** - JSON processing
4. **tmux/zellij** - Terminal multiplexer for running multiple services

### Highly Recommended
5. **k6** - Load testing
6. **btop** - Resource monitoring
7. **bat** - Code/config viewing
8. **ripgrep** - Fast code search

### Nice to Have
9. **asciinema** - Demo recording
10. **gping** - Network monitoring

---

## Integration with Current Workflow

### Development Workflow with Tools

**Terminal Setup (tmux/zellij):**
```
Pane 1: LEAN algorithm (python lean run)
Pane 2: REST API wrapper (uvicorn api_wrapper:app)
Pane 3: TUI client (./build/ib_box_spread_tui)
Pane 4: API testing (ATAC or httpie)
```

**Testing Workflow:**
1. Start LEAN algorithm
2. Start REST API wrapper
3. Use **ATAC** or **httpie** to test endpoints
4. Use **jq** to parse responses
5. Use **k6** for load testing
6. Use **btop** to monitor resources

**Debugging Workflow:**
1. Use **ripgrep** to find code references
2. Use **bat** to view config files
3. Use **httpie** with **jq** to test and parse API responses
4. Use **btop** to monitor resource usage
5. Use **gping** to check network latency

---

## Task Integration

### T-49: Design LEAN REST API Wrapper
- Use **ATAC** to test existing API contract endpoints
- Use **jq** to analyze API response formats
- Use **ripgrep** to search for API contract references

### T-50: Implement LEAN REST API Wrapper
- Use **ATAC** or **httpie** to test endpoints during development
- Use **jq** to validate JSON responses
- Use **btop** to monitor resource usage
- Use **k6** for load testing

### T-51: Implement WebSocket Bridge
- Use **httpie** to test WebSocket connections
- Use **k6** to load test WebSocket bridge
- Use **gping** to monitor connection latency
- Use **btop** to monitor resource usage

### T-52: Integrate with PWA/TUI
- Use **ATAC** to test integration endpoints
- Use **jq** to validate data format conversion
- Use **tmux/zellij** to run all services simultaneously
- Use **asciinema** to record integration demos

---

## Quick Start Guide

### Install Essential Tools (macOS)
```bash
# API Testing
brew tap julien-cpsn/atac
brew install atac
brew install httpie

# JSON Processing
# Download jq from: https://github.com/jqlang/jq/releases

# Terminal Multiplexer
brew install tmux
# OR
brew install zellij

# Load Testing
brew install k6

# Monitoring
brew install btop

# Code Tools
brew install bat ripgrep
```

### Test LEAN REST API Wrapper (After T-50)
```bash
# Start LEAN wrapper
cd python/lean_integration
uvicorn api_wrapper:app --reload

# In another terminal, test endpoints
http GET http://localhost:8000/api/v1/snapshot | jq

# Or use ATAC
atac http://localhost:8000/api/v1/snapshot
```

---

## References

- [Essential CLI/TUI Tools for Developers](https://www.freecodecamp.org/news/essential-cli-tui-tools-for-developers/) - Source article
- [ATAC GitHub](https://github.com/julien-cpsn/atac) - Terminal API client
- [httpie Documentation](https://httpie.io/docs) - HTTP client
- [k6 Documentation](https://k6.io/docs/) - Load testing
- [jq Manual](https://stedolan.github.io/jq/manual/) - JSON processor

---

**Recommendation**: Install ATAC, httpie, jq, and tmux/zellij as essential tools for LEAN REST API wrapper development. These will significantly improve your development and testing workflow.
