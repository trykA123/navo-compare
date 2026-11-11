# Runs inside the Windows VM, started by vm-build.sh: checks out one commit from the bundle
# into C:\src\navo-compare and runs the asked stages. Prints "=== ALL GREEN" only when all pass.
param([Parameter(Mandatory)][string]$Sha, [string]$Stages = 'clippy,test,build')
$ErrorActionPreference = 'Stop'
$Repo = 'C:\src\navo-compare'
$Bundle = 'C:\winvm\sidewise.bundle'

$env:CARGO_TERM_COLOR = 'never'
# Incremental caches fill the VM disk and gain little for one-off builds.
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_PROFILE_DEV_DEBUG = 'line-tables-only'
$env:CARGO_PROFILE_TEST_DEBUG = 'line-tables-only'

# Runs one named stage and stops the script with exit code 1 when its last command fails.
function Step([string]$name, [scriptblock]$body) {
  $start = Get-Date
  Write-Output "=== $name"
  & $body
  if ($LASTEXITCODE -ne 0) { Write-Output "=== FAIL $name ($LASTEXITCODE)"; exit 1 }
  Write-Output ("=== ok {0} {1:n0}s" -f $name, ((Get-Date) - $start).TotalSeconds)
}

Write-Output ("=== free {0:n1} GB" -f ((Get-PSDrive C).Free / 1GB))
if (-not (Test-Path "$Repo\.git")) {
  New-Item -ItemType Directory -Force $Repo | Out-Null
  cmd /c "git init -q $Repo 2>&1"
}
Set-Location $Repo
cmd /c "git fetch -q $Bundle +refs/heads/vm-build:refs/heads/vm-build 2>&1"
cmd /c "git checkout -q -f --detach $Sha 2>&1"
cmd /c "git clean -fdq -e target 2>&1"
Write-Output "=== at $(git log --oneline -1)"
# A failed checkout would test an old commit and report a false pass.
if ((git rev-parse HEAD) -ne $Sha) { Write-Output "=== FAIL checkout (HEAD is not $Sha)"; exit 1 }

$want = $Stages.Split(',')
if ($want -contains 'clippy') {
  Step clippy { cmd /c "cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1" }
}
if ($want -contains 'test') {
  Step test { cmd /c "cargo test --workspace --locked 2>&1" }
}
if ($want -contains 'build') {
  Step build { cmd /c "cargo build -p navo-compare-app --locked 2>&1" }
}
# Fat LTO runs out of memory in the 8 GB VM; the shipped exe is built elsewhere.
if ($want -contains 'release') {
  $env:CARGO_PROFILE_RELEASE_LTO = 'thin'
  $env:CARGO_PROFILE_RELEASE_DEBUG = '0'
  Step release { cmd /c "cargo build --release -p navo-compare-app --locked 2>&1" }
}
Write-Output '=== ALL GREEN'
exit 0
