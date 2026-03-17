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
 * - 1: Error (show to user, don't block)
 * - 2: Block operation (show to user, block)
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Configuration
const PARRY_CONFIG = process.env.PARRY_CONFIG || path.join(os.homedir(), '.config', 'oparry', 'config.toml');
const STRICT_MODE = process.env.PARRY_STRICT === 'true';
const AUTO_FIX = process.env.PARRY_AUTO_FIX !== 'false';

/**
 * Find the oparry binary in common locations
 */
function findParryBin() {
    // Check environment variable first
    if (process.env.PARRY_BIN) {
        return process.env.PARRY_BIN;
    }

    const isWindows = os.platform() === 'win32';
    const binName = isWindows ? 'oparry.exe' : 'oparry';

    // Common cargo installation paths
    const cargoBin = path.join(os.homedir(), '.cargo', 'bin', binName);
    if (fs.existsSync(cargoBin)) {
        return cargoBin;
    }

    // Try to find via cargo which command
    try {
        const cargoPath = execSync('cargo which oparry', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
        if (cargoPath) return cargoPath;
    } catch (e) {
        // Cargo not available or oparry not installed
    }

    // Fall back to PATH lookup
    return binName;
}

const PARRY_BIN = findParryBin();

// Logger
function log(level, message) {
    const timestamp = new Date().toISOString();
    const method = level === 'stderr' ? 'error' : level;
    console[method](`[${timestamp}] [parry-hook] ${message}`);
}

function debug(message) {
    if (process.env.PARRY_DEBUG) {
        log('error', `[DEBUG] ${message}`);
    }
}

/**
 * Validate a file with Parry
 * @param {string} filePath - Path to the file
 * @param {string} content - File content
 * @returns {object} Validation result
 */
function validateFile(filePath, content) {
    // Write to temp file with same extension for language detection
    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-tmp' + parsedPath.ext);
    fs.writeFileSync(tmpPath, content, 'utf8');

    try {
        // Run oparry check
        const result = execSync(
            `"${PARRY_BIN || "oparry"}" check --output json "${tmpPath}"`,
            {
                encoding: 'utf-8',
                stdio: ['ignore', 'pipe', 'pipe'],
                timeout: 30000, // 30s timeout
                env: { ...process.env, PARRY_CONFIG }
            }
        );

        const validation = JSON.parse(result);
        fs.unlinkSync(tmpPath);
        return { success: true, validation };
    } catch (error) {
        // If file doesn't exist or other error, try to parse anyway
        try {
            const validation = JSON.parse(error.stdout);
            fs.unlinkSync(tmpPath);
            return { success: true, validation };
        } catch (e) {
            fs.unlinkSync(tmpPath);
            // Parry not found or other error - allow the write
            debug(`Parry not available: ${error.message}`);
            return { success: true, validation: null };
        }
    }
}

/**
 * Attempt to fix issues with Parry
 * @param {string} filePath - Path to the file
 * @returns {object} Fix result
 */
function attemptFix(filePath) {
    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-tmp' + parsedPath.ext);

    try {
        const result = execSync(
            `"${PARRY_BIN || "oparry"}" check --fix --output json "${tmpPath}"`,
            {
                encoding: 'utf-8',
                stdio: ['ignore', 'pipe', 'pipe'],
                timeout: 30000,
                env: { ...process.env, PARRY_CONFIG }
            }
        );

        const fixedContent = fs.readFileSync(tmpPath, 'utf-8');
        fs.unlinkSync(tmpPath);
        return { success: true, fixedContent };
    } catch (error) {
        const content = fs.existsSync(tmpPath) ? fs.readFileSync(tmpPath, 'utf-8') : null;
        if (content) fs.unlinkSync(tmpPath);
        return { success: false, fixedContent: content, error: error.message };
    }
}

/**
 * Format issues for display
 */
function formatIssues(issues) {
    let output = '';
    const errorCount = issues.filter(i => i.level === 'error').length;
    const warningCount = issues.filter(i => i.level === 'warning').length;

    output += `\n✗ Parry detected ${errorCount} error${errorCount > 1 ? 's' : ''} and ${warningCount} warning${warningCount > 1 ? 's' : ''}:\n\n`;

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
        debug(`Failed to read stdin: ${e.message}`);
        process.exit(0); // Allow on JSON parse error
    }

    let input;
    try {
        input = JSON.parse(inputData);
    } catch (e) {
        debug(`Failed to parse JSON: ${e.message}`);
        process.exit(0);
    }

    // Only process text_editor writes
    if (input.tool_name !== 'text_editor') {
        process.exit(0);
    }

    const toolInput = input.tool_input || {};
    const operation = toolInput.operation;
    const filePath = toolInput.file_path;
    const content = toolInput.content;

    // Only validate write operations
    if (operation !== 'write' && operation !== 'create') {
        process.exit(0);
    }

    if (!filePath || !content) {
        process.exit(0);
    }

    debug(`Validating: ${filePath}`);

    // Validate the file
    const { success, validation, error } = validateFile(filePath, content);

    if (!success || !validation) {
        // Parry not available or error - allow the write
        debug('Parry validation skipped, allowing write');
        process.exit(0);
    }

    // Check if validation passed
    if (validation.passed) {
        debug('Validation passed');
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
        debug('Attempting auto-fix...');
        const { success: fixSuccess, fixedContent } = attemptFix(filePath);

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
        debug('Blocking write due to errors');
        process.exit(2); // Block the write
    } else {
        debug('Warning only, allowing write');
        process.exit(1); // Show warnings but allow
    }
}

// Run the hook
if (require.main === module) {
    main();
}
