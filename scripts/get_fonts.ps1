# Downloads static font instances from the Google Fonts CSS API. Prints FONTS OK or FONTS INCOMPLETE.
$fonts = @(
  @{ File = "BigShouldersStencil-800.ttf"; Url = "https://fonts.googleapis.com/css2?family=Big+Shoulders+Stencil:wght@800" },
  @{ File = "BigShouldersStencil-900.ttf"; Url = "https://fonts.googleapis.com/css2?family=Big+Shoulders+Stencil:wght@900" },
  @{ File = "MartianMono-Condensed-500.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,500" },
  @{ File = "MartianMono-Condensed-600.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,600" },
  @{ File = "MartianMono-Condensed-700.ttf"; Url = "https://fonts.googleapis.com/css2?family=Martian+Mono:wdth,wght@75,700" }
)
$ok = $true
foreach ($f in $fonts) {
  $dest = Join-Path "assets\fonts" $f.File
  try {
    $css = (Invoke-WebRequest -UseBasicParsing -Uri $f.Url -UserAgent "Wget/1.21").Content
    $m = [regex]::Match($css, 'url\((https://[^)]+)\)')
    if (-not $m.Success) { throw "no font url in css" }
    Invoke-WebRequest -UseBasicParsing -Uri $m.Groups[1].Value -OutFile $dest -UserAgent "Wget/1.21"
    $bytes = [System.IO.File]::ReadAllBytes($dest)
    $magic = [System.BitConverter]::ToString($bytes, 0, 4)
    if ($bytes.Length -lt 20000 -or ($magic -ne "00-01-00-00" -and $magic -ne "4F-54-54-4F")) { throw "not a TTF/OTF ($magic, $($bytes.Length) bytes)" }
    Write-Host "OK   $($f.File)  $($bytes.Length) bytes"
  } catch {
    Write-Host "FAIL $($f.File): $_"
    if (Test-Path $dest) { Remove-Item $dest }
    $ok = $false
  }
}
foreach ($lic in @(@{ Dir = "martianmono"; Out = "OFL-MartianMono.txt" }, @{ Dir = "bigshouldersstencil"; Out = "OFL-BigShouldersStencil.txt" })) {
  try {
    Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/google/fonts/main/ofl/$($lic.Dir)/OFL.txt" -OutFile (Join-Path "assets\fonts" $lic.Out)
    Write-Host "OK   $($lic.Out)"
  } catch { Write-Host "FAIL licence $($lic.Out)" }
}
if ($ok) { Write-Host "FONTS OK" } else { Write-Host "FONTS INCOMPLETE" }
