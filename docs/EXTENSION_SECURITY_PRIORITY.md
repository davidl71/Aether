# Extension Security Priority Review

## Critical Security Concerns (2024-2025)

Based on recent security incidents, here are the priority extensions to review:

### 🔴 High Priority - Review Immediately

**1. Amazon Extensions (After July 2025 Breach)**
- `amazonwebservices.codewhisperer-for-command-line-companion`
- **Action**: Verify latest version, check for security updates
- **Risk**: Known security incident in July 2025

**2. Unknown Publishers with Low Visibility**
- `backnotprop.prompt-tower` - Unknown publisher
- `pascalx.sketchprompt` - Unknown publisher
- `fridaplatform.fridagpt` - Unknown publisher
- `quantconnect.quantconnect` - Verify QuantConnect identity
- **Action**: Research publisher, check download counts, reviews

**3. MCP Extensions (Multiple Unknown Publishers)**
- `yutengjing.vscode-mcp-bridge` - MCP bridge (verify source)
- `cjl.lsp-mcp` - Unknown publisher
- `daninemonic.mcp4humans` - Unknown publisher
- `interactive-mcp.interactive-mcp` - Unknown publisher
- `kirigaya.openmcp` - Unknown publisher
- `pimzino.agentic-tools-mcp-companion` - Unknown publisher
- `raz-labs.interactive-mcp` - Unknown publisher
- **Action**: Verify which MCP extensions you actually use, remove unused ones

### 🟡 Medium Priority - Review Soon

**4. Popular but Verify**
- `donjayamanne.githistory` - Popular, but verify publisher
- `vadimcn.vscode-lldb` - Popular debugger, verify
- `vscodevim.vim` - Popular Vim extension, verify
- `continue.continue` - Popular AI tool, verify
- `usernamehw.errorlens` - Popular, verify
- **Action**: Check marketplace reviews, verify publisher identity

**5. Specialized/Enterprise Tools**
- `halcyontechltd.*` - IBM i tools (if you use IBM i, verify)
- `barrettotte.ibmi-languages` - IBM i (if needed, verify)
- `broadcommfd.cobol-language-support` - COBOL (if needed, verify)
- `zowe.*` - Zowe mainframe tools (if needed, verify)
- `ibm.zopendebug` - IBM official (should be safe, but verify)
- **Action**: Only keep if you actually use these technologies

**6. C++ Development Helpers**
- `franneck94.*` - C++ helpers (verify if needed)
- `jbenden.*` - C++ linter (verify if needed)
- `jeff-hykin.better-cpp-syntax` - Syntax helper (verify)
- `kylinideteam.cmake-intellisence` - CMake helper (verify)
- `twxs.cmake` - CMake helper (verify)
- **Action**: Review if these add value beyond `anysphere.cpptools`

### 🟢 Low Priority - Likely Safe

**7. Well-Known but Not in Trusted List**
- `1password.op-vscode` - 1Password official (likely safe)
- `firefox-devtools.vscode-firefox-debug` - Mozilla official (likely safe)
- `vercel.turbo-vsc` - Vercel official (likely safe)
- `ibm.zopendebug` - IBM official (likely safe)
- **Action**: Quick verification recommended

## Verification Checklist

For each extension needing review:

- [ ] **Publisher Identity**: Does publisher match expected source?
- [ ] **Download Count**: Is it >10,000 downloads? (be cautious if <1,000)
- [ ] **Ratings**: Check star ratings and reviews
- [ ] **Recent Updates**: Has it been updated in last 6 months?
- [ ] **GitHub Repository**: Does it have a public repo? Is it active?
- [ ] **Security Advisories**: Check for known vulnerabilities
- [ ] **Permissions**: What permissions does it request?
- [ ] **Telemetry**: Does it send data? Where?

## Quick Verification Commands

```bash
# Check extension versions
cursor --list-extensions --show-versions | grep -E "(amazonwebservices|backnotprop|pascalx|fridaplatform|quantconnect)"

# Check for updates
cursor --update-extensions

# List all extensions with details
cursor --list-extensions --show-versions > extensions_list.txt
```

## Recommended Actions

### Immediate (This Week)

1. **Verify Amazon Extensions**
   - Check `amazonwebservices.codewhisperer-for-command-line-companion` is latest version
   - Review if you actually use it (consider removing if not)

2. **Review Unknown AI/Prompt Extensions**
   - `backnotprop.prompt-tower`
   - `pascalx.sketchprompt`
   - `fridaplatform.fridagpt`
   - Research each, keep only if trusted

3. **Consolidate MCP Extensions**
   - You have 7 MCP extensions
   - Keep only 1-2 you actually use
   - Remove others

### Short Term (This Month)

4. **Review Popular Extensions**
   - Verify `donjayamanne.githistory`
   - Verify `vadimcn.vscode-lldb`
   - Verify `vscodevim.vim`
   - Verify `continue.continue`

5. **Remove Unused Enterprise Tools**
   - If you don't use IBM i: Remove `halcyontechltd.*`, `barrettotte.*`
   - If you don't use COBOL: Remove `broadcommfd.*`
   - If you don't use mainframe: Remove `zowe.*`

### Ongoing

6. **Regular Security Audits**
   - Run `./scripts/check_extension_security.sh` monthly
   - Keep extensions updated
   - Review extension changelogs
   - Monitor security advisories

## Resources

- **VS Code Marketplace**: https://marketplace.visualstudio.com/vscode
- **Extension Security Scanner**: `./scripts/check_extension_security.sh`
- **GitHub Security Advisories**: https://github.com/advisories
- **VS Code Security**: https://code.visualstudio.com/docs/supporting/security

## Summary

- **Trusted**: 26 extensions (43%)
- **Needs Review**: 35 extensions (57%)
- **High Priority**: ~10 extensions (Amazon, unknown AI tools, MCP)
- **Medium Priority**: ~15 extensions (popular but verify)
- **Low Priority**: ~10 extensions (likely safe but verify)

**Recommendation**: Focus on reviewing high-priority extensions first, especially Amazon extensions and unknown AI/prompt tools.
