$edge = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe", "C:\Program Files (x86)\Microsoft\EdgeCore\154.0.4258.53\msedge.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $edge) { Write-Host "REFERENCE FAILED: Edge not found"; exit 1 }
$html = (Resolve-Path "reference\holy-moly-editor.html").Path
$out = Join-Path (Resolve-Path "reference").Path "reference.png"
& $edge --headless=new --disable-gpu --hide-scrollbars --window-size=1152,792 --virtual-time-budget=5000 "--screenshot=$out" ("file:///" + $html.Replace('\','/'))
Start-Sleep -Seconds 3
if (Test-Path $out) { Write-Host "REFERENCE OK $out (editor starts at 16,16)" } else { Write-Host "REFERENCE FAILED"; exit 1 }
