# Downloads python-build-standalone CPython 3.11 for Windows.
# Extracts, strips test suites/__pycache__/idle/tkinter to reduce size.
# Output: target/python-dist/{python.exe,Lib,...}

$ErrorActionPreference = 'Stop'

$PYTHON_VERSION = "3.11.11"
$PBS_TAG = "20250317"

$URL = "https://github.com/indygreg/python-build-standalone/releases/download/${PBS_TAG}/cpython-${PYTHON_VERSION}+${PBS_TAG}-x86_64-pc-windows-msvc-install_only_stripped.tar.gz"

Write-Output "Downloading CPython ${PYTHON_VERSION} for Windows x86_64..."
Write-Output "URL: ${URL}"

New-Item -ItemType Directory -Path target -Force | Out-Null
Invoke-WebRequest -Uri $URL -OutFile target/python-standalone.tar.gz

Write-Output "Extracting..."
if (Test-Path "target/python-dist") {
    Remove-Item -Path "target/python-dist" -Recurse -Force
}
New-Item -ItemType Directory -Path target/python-dist -Force | Out-Null

# tar supports .tar.gz on modern Windows
tar xzf target/python-standalone.tar.gz -C target/python-dist --strip-components=1

Write-Output "Stripping unnecessary files..."
$dirsToRemove = @(
    "target/python-dist/Lib/test",
    "target/python-dist/Lib/idlelib",
    "target/python-dist/Lib/tkinter",
    "target/python-dist/Lib/turtledemo"
)
foreach ($dir in $dirsToRemove) {
    if (Test-Path $dir) {
        Remove-Item -Path $dir -Recurse -Force
    }
}

# Remove turtle files
Get-ChildItem -Path "target/python-dist/Lib" -Filter "turtle*" -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

# Remove bundled wheel files from ensurepip
Get-ChildItem -Path "target/python-dist/Lib/ensurepip/_bundled" -Filter "*.whl" -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue

# Remove __pycache__ directories
Get-ChildItem -Path "target/python-dist" -Directory -Recurse -Filter "__pycache__" -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue

# Remove .pyc files
Get-ChildItem -Path "target/python-dist" -File -Recurse -Filter "*.pyc" -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue

Write-Output "Done. Python distribution at target/python-dist/"
$size = (Get-ChildItem -Path "target/python-dist" -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB
Write-Output ("Size: {0:N1} MB" -f $size)
