# 从 generated.rs 提取所有 ASN.1 类型名
$content = Get-Content -Path "$PSScriptRoot/../src/generated.rs" -Raw
$types = @()
$content -split 'pub (?:struct|enum) ' | Select-Object -Skip 1 | ForEach-Object {
    $name = ($_ -split '[\s({]')[0]
    if ($name -and $name -notmatch '^_|^(struct|enum)$') {
        $types += $name
    }
}
$types | Sort-Object -Unique | Write-Host
