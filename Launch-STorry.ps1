$ErrorActionPreference = 'Stop'
$appPath = Join-Path $PSScriptRoot 'src-tauri\target\debug\storry.exe'

try {
    if (-not (Test-Path -LiteralPath $appPath)) {
        throw "STorry has not been built. Open a terminal in $PSScriptRoot and run: npm run tauri build -- --debug --no-bundle"
    }
    Start-Process -FilePath $appPath -WorkingDirectory $PSScriptRoot -WindowStyle Hidden
} catch {
    Add-Type -AssemblyName PresentationFramework
    [System.Windows.MessageBox]::Show($_.Exception.Message, 'STorry launcher', 'OK', 'Error') | Out-Null
    exit 1
}
