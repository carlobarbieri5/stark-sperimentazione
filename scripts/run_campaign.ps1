# STARK measurement-campaign orchestrator (winterfell 0.13.1).
#
# Builds the release harness, then runs every sweep configuration, capturing one
# CSV row per repetition into results.csv. Any config that errors / OOMs / times
# out is recorded in failures.csv (with the reason) and the campaign continues.
#
# Every number in results.csv comes from a real prove/verify run. Nothing is
# fabricated or averaged here.

$ErrorActionPreference = "Stop"

# ---- paths / constants ----------------------------------------------------------
# Repo root = parent of this script's directory (script lives in scripts/).
$Root        = Split-Path -Parent $PSScriptRoot
$DataDir     = Join-Path $Root "data"
if (-not (Test-Path $DataDir)) { New-Item -ItemType Directory -Path $DataDir | Out-Null }
$Exe         = Join-Path $Root "target\release\stark_campaign.exe"
$ResultsCsv  = Join-Path $DataDir "results.csv"
$FailuresCsv = Join-Path $DataDir "failures.csv"
$RunLog      = Join-Path $DataDir "run_log.txt"
$EnvTxt      = Join-Path $DataDir "ENV.txt"

$Machine     = "laptop_ryzen7_7840u_32gb"
$Repetitions = 7
$TimeoutSec  = 7200          # hard per-invocation safety cap (most configs far under)

$Header = "sweep,workload,n,commit_hash,lambda_bits,num_queries,blowup_factor,grinding_factor,field_extension,fri_folding_factor,threads,machine,run_index,t_native_ms,t_prove_ms,t_verify_ms,proof_bytes,security_bits"

# baseline configuration (one axis varied per sweep, rest held here)
$Base = [ordered]@{ n=65536; hash="blake3_256"; q=32; blowup=8; grind=16; fe="none"; fold=8; threads=1 }

# ---- helpers --------------------------------------------------------------------
function New-Config([string]$sweep, [hashtable]$over) {
    $c = [ordered]@{ sweep=$sweep }
    foreach ($k in $Base.Keys) { $c[$k] = $Base[$k] }
    if ($over) { foreach ($k in $over.Keys) { $c[$k] = $over[$k] } }
    return [pscustomobject]$c
}

function Log([string]$msg) {
    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $msg
    Add-Content -Path $RunLog -Value $line -Encoding utf8
    Write-Host $line
}

# Runs one configuration. Appends rows to results.csv on success, or a row to
# failures.csv on error/timeout. Returns the number of result rows captured.
function Invoke-Config([pscustomobject]$c) {
    $argList = @(
        "--n", $c.n,
        "--commit-hash", $c.hash,
        "--num-queries", $c.q,
        "--blowup-factor", $c.blowup,
        "--grinding-factor", $c.grind,
        "--field-extension", $c.fe,
        "--fri-folding-factor", $c.fold,
        "--threads", $c.threads,
        "--repetitions", $Repetitions,
        "--sweep", $c.sweep,
        "--machine", $Machine
    )
    $env:RAYON_NUM_THREADS = "$($c.threads)"
    $cmd = "RAYON_NUM_THREADS=$($c.threads) stark_campaign.exe " + ($argList -join " ")
    Log "RUN  $cmd"

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Exe
    $psi.Arguments = ($argList -join " ")
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = [System.Diagnostics.Process]::Start($psi)
    $outTask = $proc.StandardOutput.ReadToEndAsync()
    $errTask = $proc.StandardError.ReadToEndAsync()

    $timedOut = $false
    if (-not $proc.WaitForExit($TimeoutSec * 1000)) {
        $timedOut = $true
        try { $proc.Kill() } catch {}
        $proc.WaitForExit()
    }
    $sw.Stop()
    $out  = $outTask.Result
    $err  = $errTask.Result
    $code = $proc.ExitCode
    $secs = [math]::Round($sw.Elapsed.TotalSeconds, 1)

    $rows = @()
    if ($out) { $rows = $out -split "`r?`n" | Where-Object { $_.Trim().Length -gt 0 } }

    if ($timedOut) {
        $reason = "timeout after ${TimeoutSec}s"
    } elseif ($code -ne 0) {
        $errTail = ($err -split "`r?`n" | Where-Object { $_.Trim().Length -gt 0 } | Select-Object -Last 1)
        $reason = "exit_code=$code; $errTail"
    } elseif ($rows.Count -ne $Repetitions) {
        $reason = "expected $Repetitions rows, got $($rows.Count)"
    } else {
        $reason = $null
    }

    if ($null -eq $reason) {
        Add-Content -Path $ResultsCsv -Value $rows -Encoding utf8
        Log "OK   ${secs}s  rows=$($rows.Count)  (last security_bits/proof_bytes shown below)"
        Log "     $($rows[-1])"
        return $rows.Count
    } else {
        $clean = ($reason -replace '[\r\n,]', ' ').Trim()
        $frow = "$($c.sweep),$($c.n),$($c.hash),$($c.q),$($c.blowup),$($c.grind),$($c.fe),$($c.fold),$($c.threads),$clean"
        Add-Content -Path $FailuresCsv -Value $frow -Encoding utf8
        Log "FAIL ${secs}s  $clean"
        return 0
    }
}

# ---- (re)initialize output files ------------------------------------------------
Set-Content -Path $ResultsCsv  -Value $Header -Encoding utf8
Set-Content -Path $FailuresCsv -Value "sweep,n,commit_hash,num_queries,blowup_factor,grinding_factor,field_extension,fri_folding_factor,threads,reason" -Encoding utf8
Set-Content -Path $RunLog      -Value ("Campaign started {0}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss")) -Encoding utf8

# ---- ENV.txt --------------------------------------------------------------------
$cpu = Get-CimInstance Win32_Processor
$cs  = Get-CimInstance Win32_ComputerSystem
$os  = Get-CimInstance Win32_OperatingSystem
$wfVer = (& cargo tree --manifest-path (Join-Path $Root "Cargo.toml")) | Select-String -Pattern "winterfell v" | Select-Object -First 1
$envLines = @(
    "winterfell: $($wfVer.ToString().Trim())"
    "rustc: $((& rustc --version))"
    "cargo: $((& cargo --version))"
    "cpu: $($cpu.Name.Trim())"
    "cores: $($cpu.NumberOfCores)  logical: $($cpu.NumberOfLogicalProcessors)"
    "ram_gb: $([math]::Round($cs.TotalPhysicalMemory/1GB,1))"
    "os: $($os.Caption) $($os.Version)"
    "machine_tag: $Machine"
    "repetitions: $Repetitions  (plus 1 discarded warm-up per config)"
)
Set-Content -Path $EnvTxt -Value $envLines -Encoding utf8
Log "ENV captured -> ENV.txt"

# Rp64_256 cannot run with the f128 rescue AIR (it is f64-only). Record the omission.
Add-Content -Path $FailuresCsv -Value "S2,65536,rp64_256,32,8,16,none,8,1,Rp64_256 is f64-only; stock rescue AIR is f128 - incompatible (excluded by design)" -Encoding utf8
Log "Noted Rp64_256 exclusion in failures.csv"

# ---- build campaign config list -------------------------------------------------
$configs = @()

# S2 backend: Blake3_256, Sha3_256 (Rp64_256 dropped above)
foreach ($h in @("blake3_256","sha3_256")) { $configs += New-Config "S2" @{ hash=$h } }

# S3 lambda: Blake3_256 (256) vs Blake3_192 (192)
foreach ($h in @("blake3_256","blake3_192")) { $configs += New-Config "S3" @{ hash=$h } }

# S4 queries
foreach ($q in @(16,24,32,48,64)) { $configs += New-Config "S4" @{ q=$q } }

# S5 grinding
foreach ($g in @(0,8,16,24,32)) { $configs += New-Config "S5" @{ grind=$g } }

# S6 optional: blowup {4,8,16} (fe=none) + field_extension quad (blowup=8)
foreach ($b in @(4,8,16)) { $configs += New-Config "S6" @{ blowup=$b } }
$configs += New-Config "S6" @{ fe="quad" }

# MT: baseline re-run multi-threaded (verifier still single-thread inside the harness)
$configs += New-Config "MT" @{ threads=16 }

# S1 scaling (heaviest -> placed last so partial data survives interruption)
foreach ($n in @(1024,4096,16384,65536,262144,1048576)) { $configs += New-Config "S1" @{ n=$n } }

# ---- run ------------------------------------------------------------------------
Log "Total configurations: $($configs.Count)  x $Repetitions reps"
$total = 0
$idx = 0
foreach ($c in $configs) {
    $idx++
    Log "=== [$idx/$($configs.Count)] sweep=$($c.sweep) n=$($c.n) hash=$($c.hash) q=$($c.q) blowup=$($c.blowup) grind=$($c.grind) fe=$($c.fe) fold=$($c.fold) threads=$($c.threads) ==="
    $total += (Invoke-Config $c)
}

Log "Campaign finished. results.csv data rows: $total"
Write-Host ""
Write-Host "DONE. Data rows in results.csv: $total"
