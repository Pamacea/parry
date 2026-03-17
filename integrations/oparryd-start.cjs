#!/usr/bin/env node
/**
 * Parry Daemon Starter Hook
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
        if (cargoPath && fs.existsSync(cargoPath)) {
            return cargoPath;
        }
    } catch (e) {
        // Not found
    }

    return null; // Return null if not found
}

// Check if oparryd is already running
function isDaemonRunning() {
    try {
        const isWindows = os.platform() === 'win32';

        if (isWindows) {
            // Use wmic or tasklist for Windows
            try {
                const result = execSync('tasklist /FI "IMAGENAME eq oparryd.exe"', {
                    encoding: 'utf8',
                    stdio: ['ignore', 'pipe', 'ignore']
                });
                return result.includes('oparryd.exe');
            } catch (e) {
                return false;
            }
        } else {
            // Use pgrep on Unix
            try {
                execSync('pgrep -x oparryd', { stdio: ['ignore', 'pipe', 'ignore'] });
                return true;
            } catch (e) {
                return false;
            }
        }
    } catch (e) {
        return false;
    }
}

// Start daemon in background
function startDaemon(parrydPath) {
    const isWindows = os.platform() === 'win32';

    if (!parrydPath) {
        console.error('⚠️  Parry daemon binary not found. Skipping daemon start.');
        console.error('   Install with: cargo install --path crates/oparry-daemon');
        return false;
    }

    try {
        if (isWindows) {
            // On Windows, use spawn with detached to run in background
            spawn(parrydPath, ['run', '--foreground'], {
                detached: true,
                stdio: ['ignore', 'pipe', 'pipe'],
                shell: true
            }).unref();
        } else {
            // On Unix, use spawn with detached
            spawn(parrydPath, ['run', '--foreground'], {
                detached: true,
                stdio: ['ignore', 'pipe', 'pipe']
            }).unref();
        }

        console.error('✓ Parry daemon started');
        return true;
    } catch (e) {
        console.error(`⚠️  Failed to start Parry daemon: ${e.message}`);
        return false;
    }
}

function main() {
    // Check if already running
    if (isDaemonRunning()) {
        // Daemon already running - nothing to do
        process.exit(0);
    }

    // Find the daemon binary
    const parrydPath = findParrydBin();

    if (!parrydPath) {
        console.error('⚠️  Parry daemon not found. Daemon auto-start disabled.');
        console.error('   To enable: cargo install --path crates/oparry-daemon');
        process.exit(0); // Allow session to start anyway
    }

    // Start the daemon
    const started = startDaemon(parrydPath);

    if (started) {
        console.error('✓ Parry daemon auto-started with Claude Code');
    }

    // Always allow session to start
    process.exit(0);
}

if (require.main === module) {
    main();
}
