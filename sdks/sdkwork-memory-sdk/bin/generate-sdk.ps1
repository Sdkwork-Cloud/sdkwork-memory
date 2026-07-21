param(
    [string[]]$Languages = @("typescript"),
    [string]$BaseUrl = "http://localhost:8080",
    [string]$SdkVersion = "0.1.0"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$FamilyRoot = (Get-Item $ScriptDir).Parent.FullName
$MemoryRoot = (Get-Item $FamilyRoot).Parent.Parent.FullName
$WorkspaceRoot = (Get-Item (Join-Path $MemoryRoot "..")).FullName
$GeneratorPath = Join-Path $WorkspaceRoot "sdkwork-sdk-generator\bin\sdkgen.js"
$InputPath = Join-Path $FamilyRoot "openapi\memory-open-api.openapi.json"
$SdkName = "sdkwork-memory-sdk"
$ApiPrefix = "/mem/v3/api"
$PackageName = "@sdkwork/memory-sdk"

if (-not (Test-Path $GeneratorPath)) {
    throw "Canonical SDK generator not found: $GeneratorPath"
}
if (-not (Test-Path $InputPath)) {
    & node (Join-Path $MemoryRoot "tools\materialize_phase1_contracts.mjs")
}
if (-not (Test-Path $InputPath)) {
    throw "OpenAPI sdkgen input not found: $InputPath"
}

foreach ($LanguageValue in $Languages) {
    foreach ($LanguagePart in "$LanguageValue".Split(",")) {
        $Language = $LanguagePart.Trim()
        if ([string]::IsNullOrWhiteSpace($Language)) {
            continue
        }

        $LanguageWorkspace = Join-Path $FamilyRoot "$SdkName-$Language"
        $OutputPath = Join-Path $LanguageWorkspace "generated\server-openapi"
        $ResolvedLanguageWorkspace = [System.IO.Path]::GetFullPath($LanguageWorkspace)
        $ResolvedOutputPath = [System.IO.Path]::GetFullPath($OutputPath)
        $LanguageWorkspacePrefix = $ResolvedLanguageWorkspace.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

        if (-not $ResolvedOutputPath.StartsWith($LanguageWorkspacePrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clean SDK output outside language workspace: $ResolvedOutputPath"
        }

        Write-Host "Generating $Language SDK at $OutputPath" -ForegroundColor Cyan
        & node $GeneratorPath generate `
            -i $InputPath `
            -o $OutputPath `
            -n $SdkName `
            -t custom `
            -l $Language `
            --fixed-sdk-version $SdkVersion `
            --base-url $BaseUrl `
            --api-prefix $ApiPrefix `
            --package-name $PackageName `
            --standard-profile sdkwork-v3 `
            --sdk-root $FamilyRoot `
            --sdk-name $SdkName `
            --no-sync-published-version

        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
    }
}
