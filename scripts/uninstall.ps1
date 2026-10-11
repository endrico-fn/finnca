# ==============================================================================
# Finnca — Windows Uninstaller Shim
# Usage: irm https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/uninstall.ps1 | iex
# ==============================================================================

$EngineUrl = "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.ps1"
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $Script = (Invoke-WebRequest -Uri $EngineUrl -UseBasicParsing).Content
    & ([scriptblock]::Create($Script)) -Action "uninstall"
} catch {
    Write-Error "Failed to bootstrap Finnca uninstaller from $EngineUrl: $($_.Exception.Message)"
}
