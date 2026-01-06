# IBC (Interactive Brokers Controller) - Learning Resources

**Source**: [IBC GitHub Repository](https://github.com/IbcAlpha/IBC) - Automation of Interactive Brokers TWS

**Repository**: `IbcAlpha/IBC`
**License**: GPL-3.0
**Stars**: 1.3k
**Latest Release**: 3.23.0 (July 3, 2025)

## Overview

IBC (Interactive Brokers Controller) is a Java-based automation tool that automates many aspects of running Interactive Brokers Trader Workstation
(TWS) and Gateway that would otherwise require manual intervention. It's especially useful for automated trading systems based on the Interactive
Brokers API, but many manual traders also find it helpful.

**Key Point**: IBC is a fork of the original IBController project, maintained by Richard L King (rlktradewright) since 2018.

## Core Capabilities

### 1. Automatic Login

- **Auto-fills username and password** in the Login dialog when TWS or Gateway start
- **Automatically clicks the Login button**
- Handles login automation without manual intervention

### 2. Session Management

- **Prevents multiple logins**: Ensures that while a TWS/Gateway session is running, attempts to logon from another computer or device do not succeed
- **Session isolation**: Protects running sessions from interference

### 3. Two-Factor Authentication (2FA)

- **IBKR Mobile integration**: Can participate in Two Factor Authentication using IBKR Mobile
- **Retry mechanism**: Users who miss the 2FA alert automatically get further opportunities without needing to be at the computer
- **Automated 2FA flow**: Streamlines the 2FA process

### 4. Dialog Box Handling

- **Automatic dialog management**: Handles various dialog boxes that TWS sometimes displays
- **Keeps TWS running smoothly**: No user involvement required for routine dialogs
- **Error handling**: Manages unexpected prompts and alerts

### 5. Auto-Restart Functionality

- **Daily auto-restart**: Allows TWS and Gateway to be auto-restarted each day during the week
- **No re-authentication needed**: Restarts without requiring user to re-authenticate
- **Weekday scheduling**: Configurable for specific days of the week

### 6. Scheduled Shutdown

- **Daily shutdown**: Allows TWS and Gateway to be shut down at a specified time every day
- **Weekly shutdown**: Allows TWS to be shut down at a specified time on a specified day of the week
- **Flexible scheduling**: Customizable shutdown times

### 7. Remote Control

- **Remote shutdown**: Can be remotely instructed to shut down TWS or Gateway
- **Cloud deployments**: Useful if TWS/Gateway are running in the cloud or on an inaccessible computer
- **Remote management**: Enables remote control of trading infrastructure

## Platform Support

IBC runs on:

- **Windows**
- **macOS**
- **Linux**

## Important Notices

### Self-Updating TWS Incompatibility

**CRITICAL**: IBC **DOES NOT WORK** with the self-updating version of TWS.

> IMPORTANT: By far the most common problem that users have when setting up IBC is the result of trying to use it with the self-updating version of
TWS.
>
> **IBC DOES NOT WORK with the self-updating version of TWS.**
>
> You must install the offline version of TWS for use with IBC.
>
> Note however that there is no self-updating version of the Gateway, so the normal Gateway installer will work fine if you only want to use the
Gateway.

### Two-Factor Authentication Limitations

IBC **cannot automatically complete your login** if Interactive Brokers have given you a card or device that you must use during login. IBC can still
enter your username and password, but you will have to:

- Type in the relevant code manually, or
- Use the IBKR Mobile app to complete the login

**Security Note**: You can request Interactive Brokers (via your Account Management page on their website) to relax this requirement when logging in
to TWS or Gateway, but you will lose certain guarantees should you suffer losses as a result of your account being compromised.

### Migration from IBController

If you're moving to IBC from IBController, there are some changes that you'll have to make. See the IBC User Guide for further information.

**Best Practice**: Install IBC from scratch using the download on the Releases page rather than trying to migrate an existing IBController
installation.

## Installation & Usage

### Downloads

**Official Releases**: [Latest Release](https://github.com/ibcalpha/ibc/releases/latest)

- Separate release files for **Windows, macOS and Linux**
- Official release ZIPs available for download
- **Do not build from repository** unless you plan to modify IBC
- If building from source, test thoroughly before deploying

### User Guide

The IBC User Guide provides comprehensive installation and usage instructions:

- Included as a **PDF file in the download ZIPs**
- Also available online (see repository documentation)
- Contains migration information from IBController

### Development

If you want to make changes to IBC:

1. Clone the repository
2. Build using `build.xml` (Java/Ant build system)
3. **Test thoroughly** before deploying
4. Repository may not always be in a fully self-consistent state

**Warning**: If you build IBC.jar directly from the repository, you should test thoroughly before deploying it (especially important when composing a
Docker image).

## Technical Details

### Technology Stack

- **Primary Language**: Java (76.4%)
- **Shell Scripts**: 13.0% (for platform-specific automation)
- **Batch Files**: 10.5% (Windows support)
- **VBScript**: 0.1% (Windows-specific automation)

### Build System

- **Build Tool**: Ant (build.xml)
- **Output**: IBC.jar (Java archive)
- **Platform Scripts**: Shell scripts and batch files for Windows/macOS/Linux

### Repository Structure

```
IBC/
├── src/ibcalpha/ibc/       # Java source code
├── samples/IbcLoader/      # Sample loader code
├── resources/              # Resource files
├── makedocs/              # Documentation generation
├── build.xml              # Ant build configuration
├── README.md              # Main documentation
├── userguide.md           # User guide
└── CONTRIBUTING.md        # Contribution guidelines
```

## Support & Community

### User Group

For assistance, queries, or suggestions:

- **IBC User Group**: Join for support and discussions
- Active community for troubleshooting and feature requests

### Bug Reports

If you've found a bug in IBC, report via:

- **IBC User Group**: Community support forum
- **GitHub Issue Tracker**: Official bug tracking

**Please provide**:

- Versions of IBC and TWS/Gateway
- Full description of incorrect behavior
- **IBC log file** (location prominently displayed when running IBC)

### Log Files

IBC creates detailed log files that record:

- Login attempts and results
- Dialog interactions
- Scheduled events
- Error conditions
- All automation activities

**Location**: The log file location is prominently displayed in the window that appears when you run IBC.

## Contribution

### How to Contribute

1. Read the [Contributor Guidelines](https://github.com/IbcAlpha/IBC/blob/master/CONTRIBUTING.md)
2. Submit a pull request with your changes
3. Follow project coding standards and testing requirements

### Acknowledgments

IBC acknowledges past contributors to the IBController project:

- Richard King (original creator and maintainer)
- Steven Kearns
- Ken Geis
- Ben Alex
- Shane Castle

## History & Background

### Origin

IBC is a fork of the original **IBController** project:

- **Original Maintainer**: Richard L King (rlktradewright on GitHub)
- **Maintenance Period**: 2004 to early 2018 (14+ years)
- **Fork Reason**: Withdrew direct support for original project in early 2018
- **Current Status**: Original IBController repository status is unclear

### Migration

**IBController users are invited to switch to IBC**.

The last section of the IBC User Guide contains useful information about differences between IBController and IBC.

**Recommendation**: Install IBC from scratch rather than trying to migrate an existing IBController installation.

## Use Cases for TWS Automated Trading Project

### 1. Unattended Trading Operations

IBC enables running TWS/Gateway in an unattended mode:

- Automated login without human intervention
- Automatic restart after disconnections
- Scheduled maintenance windows
- Remote management capabilities

### 2. Production Deployments

For production trading systems:

- **Automatic daily restarts**: Ensures fresh TWS/Gateway sessions
- **Scheduled shutdowns**: Clean shutdowns during market closures
- **Session protection**: Prevents unauthorized access during trading hours
- **Cloud deployments**: Enable remote TWS/Gateway instances

### 3. Development & Testing

For development workflows:

- **Automated testing**: Restart TWS automatically for test runs
- **Session management**: Isolate test sessions from production
- **Dialog handling**: Automate away manual interactions during testing

### 4. Integration with Native C++ Client

IBC can be used alongside your native C++ TWS client:

- **IBC manages TWS/Gateway lifecycle**: Handles login, restarts, shutdowns
- **Your C++ client connects to TWS**: Uses the TWS API for trading operations
- **Separation of concerns**: IBC for TWS management, your code for trading logic

### Example Integration Pattern

```
┌─────────────────┐
│   IBC (Java)    │  ← Manages TWS/Gateway lifecycle
│  Auto-login     │
│  Auto-restart   │
│  Shutdown       │
└────────┬────────┘
         │
         │ Controls
         ▼
┌─────────────────┐
│  TWS/Gateway    │  ← Runs on specified port (7497/4001)
└────────┬────────┘
         │
         │ TWS API (Socket Connection)
         ▼
┌─────────────────┐
│  Your C++       │  ← Your trading application
│  TWS Client     │
│  (Native Code)  │
└─────────────────┘
```

### Configuration Considerations

When using IBC with your C++ client:

1. **Port Configuration**: Ensure IBC starts TWS/Gateway on the correct port
   - TWS default: 7497 (paper) or 7496 (live)
   - Gateway default: 4001 (paper) or 4002 (live)

2. **API Configuration**: IBC should configure TWS/Gateway API settings
   - Enable API connections
   - Set trusted IP addresses
   - Configure API read-only mode if needed

3. **Session Management**: IBC handles TWS restarts, your client should:
   - Detect disconnections
   - Reconnect when TWS restarts
   - Handle connection failures gracefully

4. **Scheduled Operations**: Coordinate IBC shutdown/restart with your client:
   - Shut down positions before scheduled shutdowns
   - Flush pending orders
   - Save state before shutdown

## Comparison with Native C++ Client Approach

### IBC Advantages

- **Mature automation**: Battle-tested login and session management
- **Cross-platform**: Works on Windows, macOS, Linux
- **2FA support**: Handles complex authentication flows
- **Scheduling**: Built-in restart and shutdown scheduling
- **Remote control**: Remote management capabilities

### Native C++ Client Advantages

- **Direct control**: Full control over TWS API connection
- **Lower latency**: Direct socket connection, no Java overhead
- **Custom logic**: Complete control over connection logic
- **Dependencies**: Fewer dependencies (no Java runtime needed)

### Hybrid Approach

Use IBC for **TWS/Gateway lifecycle management** while your **native C++ client handles trading logic**:

- **IBC**: Handles login, restarts, shutdowns, dialog management
- **Your C++ Client**: Handles all trading operations, market data, order management
- **Best of both worlds**: Mature automation + native performance

## Alternatives

### Direct TWS API Connection

Your project currently uses direct TWS API connections without IBC:

- **Advantage**: Full control, no Java dependency
- **Challenge**: Must handle all TWS management manually

### Gateway vs TWS

- **Gateway**: Lighter-weight, API-only interface
- **TWS**: Full GUI, more features, heavier resource usage
- **IBC supports both**: Can automate either one

## Related Resources

- [IBC GitHub Repository](https://github.com/IbcAlpha/IBC) - Source code and releases
- [IBC Releases](https://github.com/ibcalpha/ibc/releases/latest) - Latest official releases
- [IBController (Original)](https://github.com/rlktradewright/IBController) - Original project (status unclear)
- [TWS Integration Status](../../research/integration/TWS_INTEGRATION_STATUS.md) - Current project's TWS integration
- [API Documentation Index](../../API_DOCUMENTATION_INDEX.md) - TWS API resources

## Key Takeaways

1. **IBC automates TWS/Gateway lifecycle**: Login, restarts, shutdowns, dialog handling
2. **Requires offline TWS**: Does NOT work with self-updating TWS
3. **2FA limitations**: Cannot handle hardware token cards, but supports IBKR Mobile
4. **Production-ready**: Mature tool with active community and regular updates
5. **Complementary tool**: Can work alongside native C++ TWS clients
6. **Remote capabilities**: Useful for cloud and headless deployments

## Decision Points for This Project

### Should You Use IBC?

**Consider IBC if**:

- You need unattended TWS/Gateway operations
- You want scheduled restarts and shutdowns
- You need remote management capabilities
- You're deploying to cloud or inaccessible servers
- You want mature, tested automation for TWS management

**Stick with Native Approach if**:

- You want minimal dependencies (no Java runtime)
- You need full control over connection logic
- You prefer to handle all TWS management in C++
- You want to avoid external automation dependencies

### Integration Strategy

If using IBC:

1. **Use IBC for TWS management**: Login, restarts, shutdowns
2. **Keep native C++ for trading**: All trading logic in your C++ client
3. **Coordinate operations**: Ensure clean shutdowns and reconnects
4. **Monitor both**: Track IBC logs and your application logs

---

*Last Updated: Based on IBC repository information (latest release 3.23.0, July 3, 2025)*
