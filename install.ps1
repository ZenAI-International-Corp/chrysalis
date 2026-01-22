# Chrysalis installer script for Windows PowerShell

$ErrorActionPreference = "Stop"

# Repository information
$Repo = "ZenAI-International-Corp/chrysalis"
$BinaryName = "chrysalis.exe"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Success { Write-ColorOutput Green $args }
function Write-Info { Write-ColorOutput Cyan $args }
function Write-Warning { Write-ColorOutput Yellow $args }
function Write-Error { Write-ColorOutput Red $args }

# Detect architecture
function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64" -or $arch -eq "x86_64") {
        return "amd64"
    } elseif ($arch -eq "ARM64") {
        return "arm64"
    } else {
        Write-Error "Unsupported architecture: $arch"
        exit 1
    }
}

# Get latest release version
function Get-LatestVersion {
    Write-Info "Fetching latest version..."
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $version = $release.tag_name
        if (-not $version) {
            throw "Could not fetch version"
        }
        Write-Success "Latest version: $version"
        return $version
    } catch {
        Write-Error "Error: Could not fetch latest version"
        Write-Error $_.Exception.Message
        exit 1
    }
}

# Download and install
function Install-Chrysalis {
    param (
        [string]$Version,
        [string]$Arch
    )
    
    $assetName = "chrysalis-windows-$Arch"
    $downloadUrl = "https://github.com/$Repo/releases/download/$Version/${assetName}.exe.zip"
    $tempDir = [System.IO.Path]::GetTempPath()
    $zipPath = Join-Path $tempDir "${assetName}.zip"
    $extractDir = Join-Path $tempDir "chrysalis-install"
    
    Write-Info "Downloading Chrysalis $Version..."
    Write-Info "URL: $downloadUrl"
    
    try {
        # Download
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
        
        # Extract
        Write-Info "Extracting..."
        if (Test-Path $extractDir) {
            Remove-Item $extractDir -Recurse -Force
        }
        Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force
        
        # Determine install location
        $installDir = "$env:LOCALAPPDATA\chrysalis\bin"
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }
        
        Write-Info "Installing to $installDir..."
        
        # Copy binary
        $binaryPath = Join-Path $installDir $BinaryName
        Copy-Item -Path (Join-Path $extractDir $BinaryName) -Destination $binaryPath -Force
        
        # Cleanup
        Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
        Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue
        
        Write-Success "✓ Chrysalis installed successfully!"
        
        # Add to PATH if not already there
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$installDir*") {
            Write-Warning "Adding $installDir to PATH..."
            [Environment]::SetEnvironmentVariable(
                "Path",
                "$userPath;$installDir",
                "User"
            )
            $env:Path = "$env:Path;$installDir"
            Write-Success "Added to PATH. You may need to restart your terminal."
        }
        
        # Verify installation
        Write-Info "Verifying installation..."
        $version = & $binaryPath --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Chrysalis version: $version"
        } else {
            Write-Warning "Installation completed, but version check failed."
            Write-Warning "You may need to restart your terminal."
        }
        
    } catch {
        Write-Error "Error during installation: $($_.Exception.Message)"
        # Cleanup on error
        if (Test-Path $zipPath) { Remove-Item $zipPath -Force -ErrorAction SilentlyContinue }
        if (Test-Path $extractDir) { Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue }
        exit 1
    }
}

# Main
function Main {
    Write-Success "========================================"
    Write-Success "   Chrysalis Installer"
    Write-Success "========================================"
    Write-Output ""
    
    $arch = Get-Architecture
    Write-Success "Detected architecture: windows-$arch"
    
    $version = Get-LatestVersion
    Install-Chrysalis -Version $version -Arch $arch
    
    Write-Output ""
    Write-Success "========================================"
    Write-Success "Installation complete!"
    Write-Success "Run 'chrysalis --help' to get started"
    Write-Success "========================================"
    Write-Output ""
    Write-Warning "Note: If 'chrysalis' command is not found, please restart your terminal."
}

Main
