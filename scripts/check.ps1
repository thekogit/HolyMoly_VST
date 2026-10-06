# Runs format, tests, clippy and the placeholder scan. Prints CHECK PASSED or CHECK FAILED.
cargo fmt
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo fmt"; exit 1 }
cargo test --lib
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo test"; exit 1 }
cargo clippy --lib -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Host "CHECK FAILED: cargo clippy"; exit 1 }
$hits = Get-ChildItem -Recurse -Path src -Filter *.rs | Select-String -Pattern 'todo!\(|unimplemented!\(|TODO|FIXME|SCAFFOLD'
if ($hits) { $hits | ForEach-Object { Write-Host $_ }; Write-Host "CHECK FAILED: placeholders"; exit 1 }
Write-Host "CHECK PASSED"
