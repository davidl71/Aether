# setup_platform_settings.ps1 - Auto-detect and configure platform-specific settings for Windows
# Usage: .\scripts\setup_platform_settings.ps1 [-Force] [-CmakeConfigure]

param(
    [switch]$Force,
    [switch]$CmakeConfigure
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$SettingsUser = Join-Path $ProjectRoot ".vscode\settings.json.user"
$SettingsExample = Join-Path $ProjectRoot ".vscode\settings.json.user.example"

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Detect platform
function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64" -or $arch -eq "x86_64") {
        return "windows-x64"
    }
    return "windows-unknown"
}

# Detect C++ compiler (MSVC)
function Get-Compiler {
    $platform = Get-Platform

    # Try to find MSVC
    $vsBase = "C:\Program Files\Microsoft Visual Studio\2022"
    $editions = @("Community", "Professional", "Enterprise")

    foreach ($edition in $editions) {
        $msvcPath = Join-Path $vsBase $edition "VC\Tools\MSVC"
        if (Test-Path $msvcPath) {
            $versions = Get-ChildItem $msvcPath | Sort-Object Name -Descending | Select-Object -First 1
            if ($versions) {
                $clPath = Join-Path $msvcPath $versions.Name "bin\Hostx64\x64\cl.exe"
                if (Test-Path $clPath) {
                    return $clPath
                }
            }
        }
    }

    # Try MinGW-w64
    $mingwPaths = @(
        "C:\mingw64\bin\g++.exe",
        "C:\msys64\mingw64\bin\g++.exe"
    )

    foreach ($path in $mingwPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    # Try PATH
    $gpp = Get-Command g++ -ErrorAction SilentlyContinue
    if ($gpp) {
        return $gpp.Source
    }

    return $null
}

# Detect IntelliSense mode
function Get-IntelliSenseMode {
    param(
        [string]$Platform,
        [string]$CompilerPath
    )

    if ($CompilerPath -match "cl\.exe") {
        return "windows-msvc-x64"
    } elseif ($CompilerPath -match "g\+\+") {
        return "windows-gcc-x64"
    }

    return "windows-msvc-x64"  # Default
}

# Detect include paths
function Get-IncludePaths {
    param([string]$Platform)

    $paths = @(
        '${workspaceFolder}/native/include',
        '${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client'
    )

    # Try to find MSVC include paths
    $vsBase = "C:\Program Files\Microsoft Visual Studio\2022"
    $editions = @("Community", "Professional", "Enterprise")

    foreach ($edition in $editions) {
        $msvcPath = Join-Path $vsBase $edition "VC\Tools\MSVC"
        if (Test-Path $msvcPath) {
            $versions = Get-ChildItem $msvcPath | Sort-Object Name -Descending | Select-Object -First 1
            if ($versions) {
                $includePath = Join-Path $msvcPath $versions.Name "include"
                if (Test-Path $includePath) {
                    $paths += $includePath.Replace('\', '/')
                }
            }
        }
    }

    # MinGW paths
    if (Test-Path "C:\mingw64\include") {
        $paths += "C:/mingw64/include"
    }

    return $paths
}

# Detect Python interpreter
function Get-Python {
    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($python) {
        return $python.Source
    }

    $python3 = Get-Command python3 -ErrorAction SilentlyContinue
    if ($python3) {
        return $python3.Source
    }

    # Check common locations
    $commonPaths = @(
        "C:\Python3*\python.exe",
        "$env:LOCALAPPDATA\Programs\Python\Python3*\python.exe"
    )

    foreach ($pattern in $commonPaths) {
        $found = Get-ChildItem $pattern -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            return $found.FullName
        }
    }

    return $null
}

# Generate settings.json.user
function New-SettingsUser {
    param(
        [string]$Platform,
        [string]$IntelliSenseMode,
        [string]$CompilerPath,
        [string]$PythonPath,
        [string[]]$IncludePaths
    )

    Write-Info "Generating .vscode\settings.json.user for platform: $Platform"

    $includePathsJson = ($IncludePaths | ForEach-Object { "    `"$_`"" }) -join ",\n"

    $content = @"
{
  // ==========================================
  // Platform-Specific Settings (Auto-Generated)
  // ==========================================
  // Generated: $(Get-Date -Format "yyyy-MM-ddTHH:mm:ss")
  // Platform: $Platform
  // ==========================================

  // C++ IntelliSense Configuration
  "C_Cpp.default.intelliSenseMode": "$IntelliSenseMode",
  "C_Cpp.default.compilerPath": "$($CompilerPath.Replace('\', '/'))",
  "C_Cpp.default.includePath": [
$includePathsJson
  ],

  // Python Configuration
"@

    if ($PythonPath) {
        $content += @"
  "python.defaultInterpreterPath": "$($PythonPath.Replace('\', '/'))",
"@
    } else {
        $content += @"
  // Python interpreter not detected - using PATH
  // "python.defaultInterpreterPath": "",
"@
    }

    $content += @"

  // ==========================================
  // Personal Preferences (Add your own below)
  // ==========================================
  // See .vscode\settings.json.user.example for examples
}
"@

    # Ensure .vscode directory exists
    $vscodeDir = Join-Path $ProjectRoot ".vscode"
    if (-not (Test-Path $vscodeDir)) {
        New-Item -ItemType Directory -Path $vscodeDir | Out-Null
    }

    $content | Out-File -FilePath $SettingsUser -Encoding UTF8
    Write-Success "Generated $SettingsUser"
}

# Configure CMake preset
function Invoke-CmakeConfigure {
    param([string]$Platform)

    $preset = switch ($Platform) {
        "windows-x64" { "windows-x64-debug" }
        default { "windows-x64-debug" }
    }

    Write-Info "Configuring CMake with preset: $preset"

    Push-Location $ProjectRoot
    try {
        $result = & cmake --preset $preset 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "CMake configured successfully"
            Write-Info "compile_commands.json generated - VS Code will auto-detect settings"
            return $true
        } else {
            Write-Warn "CMake configuration failed - you may need to install dependencies"
            Write-Info "Run manually: cmake --preset $preset"
            return $false
        }
    } finally {
        Pop-Location
    }
}

# Main function
function Main {
    # Check if settings.json.user already exists
    if ((Test-Path $SettingsUser) -and -not $Force) {
        Write-Warn "$SettingsUser already exists"
        Write-Info "Use -Force to overwrite, or edit manually"
        return
    }

    # Detect platform
    Write-Info "Detecting platform..."
    $platform = Get-Platform
    Write-Success "Detected platform: $platform"

    # Detect compiler
    Write-Info "Detecting C++ compiler..."
    $compilerPath = Get-Compiler
    if (-not $compilerPath) {
        Write-Error "No C++ compiler detected!"
        Write-Info "Please install:"
        Write-Info "  Visual Studio 2022 (Community/Professional/Enterprise)"
        Write-Info "  Or MinGW-w64: https://www.mingw-w64.org/"
        exit 1
    }
    Write-Success "Found compiler: $compilerPath"

    # Detect IntelliSense mode
    $intellisenseMode = Get-IntelliSenseMode $platform $compilerPath
    Write-Info "IntelliSense mode: $intellisenseMode"

    # Detect include paths
    Write-Info "Detecting include paths..."
    $includePaths = Get-IncludePaths $platform
    Write-Success "Found include paths"

    # Detect Python
    Write-Info "Detecting Python interpreter..."
    $pythonPath = Get-Python
    if ($pythonPath) {
        Write-Success "Found Python: $pythonPath"
    } else {
        Write-Warn "Python not detected - will use PATH"
    }

    # Generate settings.json.user
    New-SettingsUser $platform $intellisenseMode $compilerPath $pythonPath $includePaths

    # Configure CMake if requested
    if ($CmakeConfigure) {
        Invoke-CmakeConfigure $platform
    } else {
        Write-Info "To configure CMake, run: .\scripts\setup_platform_settings.ps1 -CmakeConfigure"
    }

    # Summary
    Write-Host ""
    Write-Success "Platform settings configured!"
    Write-Host ""
    Write-Info "Next steps:"
    Write-Info "  1. Review: $SettingsUser"
    Write-Info "  2. Configure CMake: cmake --preset windows-x64-debug"
    Write-Info "  3. Reload VS Code: Ctrl+Shift+P → 'Developer: Reload Window'"
    Write-Host ""
    Write-Info "Platform-specific preset:"
    Write-Info "  cmake --preset windows-x64-debug"
}

# Run main function
Main
