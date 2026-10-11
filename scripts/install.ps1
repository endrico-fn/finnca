# ==============================================================================
# Finnca — Windows One-Liner Installer Shim
# Usage: irm https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/install.ps1 | iex
# ==============================================================================

$EngineUrl = "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.ps1"
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $Script = (Invoke-WebRequest -Uri $EngineUrl -UseBasicParsing).Content
    Invoke-Expression $Script
} catch {
    Write-Error "Failed to bootstrap Finnca installer from $EngineUrl: $($_.Exception.Message)"
}
