$ErrorActionPreference = "Stop"

$srcDir = "extension"
$distDir = "dist"
$browsers = @("chrome", "firefox", "edge", "safari")

# Ensure we are in the project root
if (-not (Test-Path -Path $srcDir)) {
    Write-Error "Error: 'extension' directory not found. Please run this script from the project root."
    exit 1
}

# Create dist directory
if (Test-Path -Path $distDir) { Remove-Item -Path $distDir -Recurse -Force }
New-Item -ItemType Directory -Path $distDir | Out-Null

foreach ($browser in $browsers) {
    Write-Host "Packaging for $browser..."
    $buildDir = Join-Path $distDir "${browser}_build"
    New-Item -ItemType Directory -Path $buildDir | Out-Null

    # Copy src directory
    Copy-Item -Path "$srcDir\src" -Destination $buildDir -Recurse

    # Copy manifest
    $manifestSource = "$srcDir\manifests\$browser.json"
    if (Test-Path -Path $manifestSource) {
        Copy-Item -Path $manifestSource -Destination "$buildDir\manifest.json"
    } else {
        Write-Warning "Manifest for $browser not found at $manifestSource"
    }

    # Zip it
    $zipFile = Join-Path $distDir "superpoweredcv-$browser.zip"
    Compress-Archive -Path "$buildDir\*" -DestinationPath $zipFile
    
    Write-Host "Created $zipFile"
}
