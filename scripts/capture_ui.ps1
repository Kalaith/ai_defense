<#
.SYNOPSIS
    Headless screenshot harness for Last Assembly.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (LAST_ASSEMBLY_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. `menu` seeds the main menu and `gameplay` seeds a fresh
    gameplay session; `sector` selects a repaired section core so its awakening
    trade-off can be reviewed; `specialization` opens a max-level tower's final
    branch choice (see Game::begin_capture_scene in src/game.rs).

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("menu", "gameplay"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
