#!/usr/bin/env node
/**
 * Parry Hook for Claude Code
 *
 * This hook intercepts file writes and validates them with Parry
 * before they are saved. If validation fails, it blocks the write
 * and shows errors/warnings to the user.
 *
 * Exit codes:
 * - 0: Success (allow operation)
 * - 1: Warning (show to user, don't block)
 * - 2: Block operation (show to user, block)
 * - 10: Configuration error (binary not found)
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Configuration
const PARRY_CONFIG = process.env.PARRY_CONFIG || path.join(os.homedir(), '.config', 'parry', 'config.toml');
const STRICT_MODE = process.env.PARRY_STRICT === 'true';
const AUTO_FIX = process.env.PARRY_AUTO_FIX !== 'false';

// Error tracking
let binaryNotFoundShown = false;

// Log file for debugging
const LOG_FILE = process.env.PARRY_LOG || path.join(os.homedir(), '.parry', 'hook.log');

function log(message) {
    const timestamp = new Date().toISOString();
    const logLine = `[${timestamp}] ${message}\n`;
    try {
        fs.appendFileSync(LOG_FILE, logLine);
    } catch (e) {
        // Ignore logging errors
    }
}

/**
 * Find the parry binary in common locations
 * Returns null if not found (instead of falling back to a guess)
 */
function findParryBin() {
    // Check environment variable first
    if (process.env.PARRY_BIN) {
        const envPath = process.env.PARRY_BIN;
        if (fs.existsSync(envPath)) {
            return envPath;
        }
        console.error(`[parry-hook] PARRY_BIN set but file not found: ${envPath}`);
    }

    const isWindows = os.platform() === 'win32';
    const binName = isWindows ? 'parry.exe' : 'parry';

    // Common cargo installation paths
    const cargoBin = path.join(os.homedir(), '.cargo', 'bin', binName);
    if (fs.existsSync(cargoBin)) {
        return cargoBin;
    }

    // Try to find via cargo which command
    try {
        const cargoPath = execSync('cargo which parry', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
        if (cargoPath && fs.existsSync(cargoPath)) {
            return cargoPath;
        }
    } catch (e) {
        // Cargo not available or oparry not installed
    }

    // Binary not found - return null to signal error
    return null;
}

/**
 * Show error message about missing binary
 */
function showBinaryNotFound() {
    if (binaryNotFoundShown) return;
    binaryNotFoundShown = true;

    const isWindows = os.platform() === 'win32';
    const binName = isWindows ? 'parry.exe' : 'parry';

    console.error('\n╔═══════════════════════════════════════════════════════════════════╗');
    console.error('║  🔴 PARRY - BINARY NOT FOUND                                  ║');
    console.error('╠═══════════════════════════════════════════════════════════════════╣');
    console.error('║                                                                   ║');
    console.error(`║  The Parry binary (${binName}) was not found.                    ║`);
    console.error('║                                                                   ║');
    console.error('║  To install Parry:                                               ║');
    console.error('║                                                                   ║');
    if (isWindows) {
        console.error('║    cargo install --path .                                    ║');
        console.error('║    (from the Parry project directory)                         ║');
    } else {
        console.error('║    cargo install --path .                                    ║');
        console.error('║    (from the Parry project directory)                         ║');
    }
    console.error('║                                                                   ║');
    console.error('║  Or build and install manually:                                   ║');
    console.error('║    cargo build --release                                         ║');
    console.error('║    cargo install --path crates/oparry-cli                       ║');
    console.error('║                                                                   ║');
    console.error('║  Files will NOT be validated until Parry is installed!            ║');
    console.error('║                                                                   ║');
    console.error('╚═══════════════════════════════════════════════════════════════════╝\n');
}

// Find binary at startup
const PARRY_BIN = findParryBin();

log(`Parry hook started - Binary: ${PARRY_BIN || 'NOT FOUND'}`);

// Show error immediately if binary not found
if (!PARRY_BIN) {
    showBinaryNotFound();
}

/**
 * Validate a file with Parry
 * @param {string} filePath - Path to the file
 * @param {string} content - File content
 * @returns {object} Validation result
 */
function validateFile(filePath, content) {
    // Early exit if no binary available
    if (!PARRY_BIN) {
        return { success: false, error: 'Binary not found', validation: null };
    }

    // Write to temp file with same extension for language detection
    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-tmp' + parsedPath.ext);
    fs.writeFileSync(tmpPath, content, 'utf8');

    try {
        // Run parry check using spawnSync for better Windows compatibility
        const result = spawnSync(
            PARRY_BIN,
            ['check', '--output', 'json', tmpPath],
            {
                encoding: 'utf-8',
                timeout: 30000, // 30s timeout
                env: { ...process.env, PARRY_CONFIG }
            }
        );

        // Check if process failed
        if (result.status !== 0 && result.stdout) {
            // Try to parse as validation result (Parry returns JSON even on validation failure)
            try {
                const validation = JSON.parse(result.stdout);
                fs.unlinkSync(tmpPath);
                return { success: true, validation };
            } catch (e) {
                // Not JSON, return error
                fs.unlinkSync(tmpPath);
                return { success: false, error: result.stderr || 'Unknown error', validation: null };
            }
        }

        // Parse successful output
        const validation = JSON.parse(result.stdout || '{"passed":true,"issues":[]}');
        fs.unlinkSync(tmpPath);
        return { success: true, validation };
    } catch (error) {
        // Clean up temp file
        if (fs.existsSync(tmpPath)) {
            fs.unlinkSync(tmpPath);
        }
        // Parry error - return failure
        log(`Parry validation error: ${error.message}`);
        return { success: false, error: error.message, validation: null };
    }
}

/**
 * Attempt to fix issues with Parry
 * @param {string} filePath - Path to the file
 * @param {string} content - Original content
 * @returns {object} Fix result
 */
function attemptFix(filePath, content) {
    if (!PARRY_BIN) {
        return { success: false, fixedContent: null };
    }

    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-tmp' + parsedPath.ext);

    // Write original content to temp file
    fs.writeFileSync(tmpPath, content, 'utf8');

    try {
        // Run parry check with fix using spawnSync
        const result = spawnSync(
            PARRY_BIN,
            ['check', '--fix', '--output', 'json', tmpPath],
            {
                encoding: 'utf-8',
                timeout: 30000,
                env: { ...process.env, PARRY_CONFIG }
            }
        );

        const fixedContent = fs.readFileSync(tmpPath, 'utf-8');
        fs.unlinkSync(tmpPath);
        return { success: true, fixedContent };
    } catch (error) {
        // Check if file was modified despite error
        const newContent = fs.existsSync(tmpPath) ? fs.readFileSync(tmpPath, 'utf-8') : null;
        if (newContent) fs.unlinkSync(tmpPath);

        // If content changed, consider it a success
        if (newContent && newContent !== content) {
            return { success: true, fixedContent: newContent };
        }

        return { success: false, fixedContent: null, error: error.message };
    }
}

/**
 * Format issues for display
 */
function formatIssues(issues) {
    let output = '';
    const errorCount = issues.filter(i => i.level === 'error').length;
    const warningCount = issues.filter(i => i.level === 'warning').length;

    output += `\n🔴 Parry detected ${errorCount} error${errorCount !== 1 ? 's' : ''} and ${warningCount} warning${warningCount !== 1 ? 's' : ''}:\n\n`;

    for (const issue of issues) {
        const icon = issue.level === 'error' ? '🔴' : '⚠️';
        output += `${icon} ${issue.message}\n`;
        if (issue.file) output += `   → ${issue.file}:${issue.line || '?'}\n`;
        if (issue.suggestion) output += `   💡 ${issue.suggestion}\n`;
    }

    return output;
}

/**
 * Main hook function
 */
function main() {
    let inputData = '';

    // Read JSON from stdin
    try {
        inputData = fs.readFileSync(0, 'utf-8');
    } catch (e) {
        log(`Failed to read stdin: ${e.message}`);
        process.exit(0); // Allow on JSON read error
    }

    // Allow empty input (not all hooks send data)
    if (!inputData || inputData.trim() === '') {
        process.exit(0);
    }

    log(`Received input: ${inputData.substring(0, 200)}`);

    let input;
    try {
        input = JSON.parse(inputData);
    } catch (e) {
        log(`Failed to parse JSON: ${e.message}`);
        log(`Input was: ${inputData.substring(0, 200)}`);
        process.exit(0); // Allow on JSON parse error
    }

    // Only process text_editor writes
    if (input.tool_name !== 'text_editor') {
        process.exit(0);
    }

    const toolInput = input.tool_input || {};
    const operation = toolInput.operation;
    const filePath = toolInput.file_path;
    const content = toolInput.content;

    log(`Processing text_editor: ${operation} on ${filePath}`);

    // Only validate write operations
    if (operation !== 'write' && operation !== 'create') {
        process.exit(0);
    }

    if (!filePath || !content) {
        process.exit(0);
    }

    // Check if binary is available before proceeding
    if (!PARRY_BIN) {
        // Show warning but allow the write (fail open)
        if (!binaryNotFoundShown) {
            showBinaryNotFound();
        }
        process.exit(0); // Allow write when Parry is not available
    }

    log(`Validating: ${filePath}`);

    // Validate the file
    const { success, validation, error } = validateFile(filePath, content);

    if (!success) {
        // Validation error - check if it's a fatal error or just validation issues
        if (!validation) {
            // Parry executable failed - log but allow
            log(`Validation error: ${error}`);
            console.error(`[parry-hook] ⚠️  Validation skipped: ${error}`);
            process.exit(0); // Allow on validation errors
        }
    }

    if (!validation) {
        process.exit(0);
    }

    // Check if validation passed
    if (validation.passed) {
        log('✓ Validation passed');
        process.exit(0);
    }

    // Validation failed - show issues
    const issues = validation.issues || [];
    if (issues.length === 0) {
        process.exit(0);
    }

    const hasErrors = issues.some(i => i.level === 'error');

    // In strict mode, warnings also fail
    const shouldBlock = hasErrors || (STRICT_MODE && issues.some(i => i.level === 'warning'));

    // Output to stderr (shown to user)
    console.error(formatIssues(issues));

    if (AUTO_FIX && !shouldBlock) {
        // Try to auto-fix
        log('Attempting auto-fix...');
        const { success: fixSuccess, fixedContent } = attemptFix(filePath, content);

        if (fixSuccess && fixedContent && fixedContent !== content) {
            console.error('\n✓ Auto-fixed! Parry has corrected the issues.\n');
            // Output the fixed content via stdout for Claude to use
            process.stdout.write(JSON.stringify({
                tool_use_id: input.tool_use_id,
                tool_result: { content: fixedContent },
                modified: true
            }));
            process.exit(0); // Allow with fixed content
        }
    }

    if (shouldBlock) {
        log('Blocking write due to errors');
        process.exit(2); // Block the write
    } else {
        log('Warning only, allowing write');
        process.exit(1); // Show warnings but allow
    }
}

// Run the hook
if (require.main === module) {
    main();
}
