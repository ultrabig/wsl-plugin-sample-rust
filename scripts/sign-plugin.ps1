#Requires -RunAsAdministrator

[cmdletbinding(PositionalBinding = $false)]
param (
    [string]$PluginPath = "$PSScriptRoot\..\target\debug\wsl_plugin_sample.dll",
    [string]$CertPath = "$PSScriptRoot\..\cert.pfx",
    [string]$CertSubject = "CN=Contoso  Corporation, O=Contoso  Corporation, L=Chigago, S=Illinois, C=US",
    [string]$CertName = "Certificate for WSL demo plugin",
    [switch]$Trust = $false
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"


if (!(Test-Path $CertPath))
{
    Write-Host "Certificate '$CertPath' not found. Generating it now"
    
    $certificate = New-SelfSignedCertificate -KeyExportPolicy Exportable -Type Custom -Subject $CertSubject -KeyUsage DigitalSignature -FriendlyName $CertName -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}") -HashAlgorithm "SHA256" -NotAfter (Get-Date).AddYears(10)
    
    $blob = $certificate.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Pfx)

    # Save the blob to the PFX file
    [System.IO.File]::WriteAllBytes($CertPath, $blob)
}

if ($Trust) 
{
    Write-Host "Trusting certificate: $CertPath"
    
    Import-PfxCertificate -FilePath $CertPath -CertStoreLocation Cert:\LocalMachine\Root | Out-Null
}

Write-Host "Signing: $PluginPath with certificate: $CertPath"

SignTool.exe sign /a /v /fd SHA256 /f $CertPath $PluginPath
