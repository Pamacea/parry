#!/usr/bin/env node
/**
 * Oparry Daemon Starter Hook
 *
 * This hook runs when Claude Code starts and automatically starts
 * the oparryd daemon if it's not already running.
 *
 * The daemon handles multi-session and multi-project validation.
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Find oparryd binary
function findParrydBin() {
    const isWindows = os.platform() === 'win32';
    const binName = isWindows ? 'oparryd.exe' : 'oparryd';

    // Check cargo bin path
    const cargoBin = path.join(os.homedir(), '.cargo', 'bin', binName);
    if (fs.existsSync(cargoBin)) {
        return cargoBin;
    }

    // Try cargo which
    try {
        const cargoPath = execSync('cargo which oparryd', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
        if (cargoPath) return cargoPath;
    } catch (e) {
        // Not found
    }

    return binName;
}

// Check if oparryd is already running
function isDaemonRunning() {
    try {
        const isWindows = os.platform() === 'win32';
        const cmd = isWindows
            ? 'tasklist /FI "IMAGENAME eq oparryd.exe" 2>NUL | find /I /N "oparryd.exe"'
            : 'pgrep -x oparryd';

        execSync(cmd, { stdio: ['ignore', 'pipe', 'ignore'] });
        return true;
    } catch (e) {
        return false;
    }
}

// Start daemon in background
function startDaemon() {
    const parryd = findParrydBin();
    const isWindows = os.platform() === 'win32';

    try {
        if (isWindows) {
            // On Windows, use start /b to run in background
            execSync(`start /b "" "${parryd}" run --foreground`, {
                stdio: ['ignore', 'pipe', 'pipe'],
                shell: true
            });
        } else {
            // On Unix, use spawn with detached
            spawn(parryd, ['run', '--foreground'], {
                detached: true,
                stdio: ['ignore', 'pipe', 'pipe']
            }).unref();
        }

        console.error('✓ Oparry daemon started');
        return true;
    } catch (e) {
        console.error(`⚠️  Failed to start Oparry daemon: ${e.message}`);
        return false;
    }
}

function main() {
    // Check if already running
    if (isDaemonRunning()) {
        // Daemon already running - nothing to do
        process.exit(0);
    }

    // Start the daemon
    const started = startDaemon();

    if (started) {
        console.error('✓ Oparry daemon auto-started with Claude Code');
    }

    // Always allow session to start
    process.exit(0);
}

if (require.main === module) {
    main();
}
