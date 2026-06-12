# Zeus VM Tools — Windows guest agent installer
# Run as Administrator inside the guest VM.
param(
    [string]$AgentUrl = $env:ZYVOR_AGENT_URL,
    [string]$InstallDir = "$env:ProgramFiles\Zyvor\VM Tools"
)

$ErrorActionPreference = "Stop"

if (-not $AgentUrl) {
    Write-Error "Set -AgentUrl or ZYVOR_AGENT_URL to the zyvor-guest-agent.exe download URL."
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$agentPath = Join-Path $InstallDir "zyvor-guest-agent.exe"

Write-Host "Downloading Zeus VM Tools agent from $AgentUrl"
Invoke-WebRequest -Uri $AgentUrl -OutFile $agentPath -UseBasicParsing

$svcName = "ZyvorGuestAgent"
$svcDisplay = "Zeus VM Tools Guest Agent"
$binPath = "`"$agentPath`" --service"

if (Get-Service -Name $svcName -ErrorAction SilentlyContinue) {
    Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
    sc.exe delete $svcName | Out-Null
    Start-Sleep -Seconds 2
}

New-Service -Name $svcName -BinaryPathName $binPath -DisplayName $svcDisplay -StartupType Automatic | Out-Null
Start-Service -Name $svcName

Write-Host "Zeus VM Tools installed. Service $svcName is running."
