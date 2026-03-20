#!/usr/bin/env node
/**
 * Parry PostToolUse Hook
 *
 * Validates files AFTER they are written and auto-corrects fixable issues.
 * For non-fixable issues, logs them for review.
 *
 * This hook runs after Write/Edit operations complete.
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Configuration
const PARRY_CONFIG = process.env.PARRY_CONFIG || path.join(os.homedir(), '.config', 'parry', 'config.toml');
const AUTO_FIX = process.env.PARRY_AUTO_FIX !== 'false';
const TRACK_CHANGES = process.env.PARRY_TRACK === '1'; // RTK-style tracking

// Log files
const HOOK_LOG = path.join(os.homedir(), '.parry', 'post-write.log');
const CHANGES_LOG = path.join(os.homedir(), '.parry', 'changes.log');

function log(msg) {
    const timestamp = new Date().toISOString();
    const line = `[${timestamp}] ${msg}\n`;
    try {
        fs.appendFileSync(HOOK_LOG, line);
    } catch (e) {}
}

function logChange(filePath, issues, fixed) {
    if (!TRACK_CHANGES) return;
    const timestamp = new Date().toISOString();
    const line = `[${timestamp}] ${filePath} | ${fixed ? 'FIXED' : 'ERROR'} | ${issues.length} issues\n`;
    try {
        fs.appendFileSync(CHANGES_LOG, line);
    } catch (e) {}
}

/**
 * Find the parry binary
 */
function findParryBin() {
    if (process.env.PARRY_BIN) {
        const envPath = process.env.PARRY_BIN;
        if (fs.existsSync(envPath)) return envPath;
    }

    const isWindows = os.platform() === 'win32';
    const binName = isWindows ? 'parry.exe' : 'parry';
    const cargoBin = path.join(os.homedir(), '.cargo', 'bin', binName);

    if (fs.existsSync(cargoBin)) return cargoBin;

    try {
        const cargoPath = execSync('cargo which parry', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
        if (cargoPath && fs.existsSync(cargoPath)) return cargoPath;
    } catch (e) {}

    return null;
}

const PARRY_BIN = findParryBin();

/**
 * Check if an issue is auto-fixable
 */
function isFixable(issue) {
    const fixableCodes = [
        'tailwind-blocked-width',      // Replace with container
        'tailwind-unknown-class',       // Remove or suggest alternative
        'trailing-whitespace',          // Remove
        'unused-import',                // Remove
        'console-log',                  // Remove (in non-debug files)
        'debugger-statement',           // Remove (in non-debug files)
        'missing-semicolon',            // Add
        'quotes',                       // Fix quotes consistency
        'indentation',                  // Fix indentation
    ];

    return fixableCodes.some(code => issue.code.includes(code)) ||
           issue.suggestion && issue.suggestion.includes('Use');
}

/**
 * Count fixable vs non-fixable issues
 */
function categorizeIssues(issues) {
    const fixable = issues.filter(isFixable);
    const nonFixable = issues.filter(i => !isFixable(i));
    return { fixable, nonFixable };
}

/**
 * Attempt to auto-fix a file
 */
function autoFix(filePath, content) {
    if (!PARRY_BIN || !AUTO_FIX) return { success: false, fixedContent: null };

    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-fix' + parsedPath.ext);

    try {
        fs.writeFileSync(tmpPath, content, 'utf-8');

        const result = spawnSync(PARRY_BIN, ['check', '--fix', '--output', 'json', tmpPath], {
            encoding: 'utf-8',
            timeout: 30000,
            env: { ...process.env, PARRY_CONFIG }
        });

        if (result.stdout) {
            const validation = JSON.parse(result.stdout || '{"passed":true,"issues":[]}');
            const fixedContent = fs.readFileSync(tmpPath, 'utf-8');
            fs.unlinkSync(tmpPath);

            return {
                success: true,
                fixedContent: fixedContent !== content ? fixedContent : null,
                validation
            };
        }

        fs.unlinkSync(tmpPath);
        return { success: false, fixedContent: null };
    } catch (e) {
        if (fs.existsSync(tmpPath)) fs.unlinkSync(tmpPath);
        return { success: false, fixedContent: null, error: e.message };
    }
}

/**
 * Format issues for display
 */
function formatIssues(issues) {
    let output = '\n🔴 Parry found issues:\n\n';
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

    try {
        inputData = fs.readFileSync(0, 'utf-8');
    } catch (e) {
        log(`Failed to read stdin: ${e.message}`);
        process.exit(0);
    }

    if (!inputData?.trim()) {
        process.exit(0);
    }

    let input;
    try {
        input = JSON.parse(inputData);
    } catch (e) {
        log(`Failed to parse JSON: ${e.message}`);
        process.exit(0);
    }

    // Only process Write and Edit tools
    if (input.tool_name !== 'Write' && input.tool_name !== 'Edit') {
        process.exit(0);
    }

    const toolInput = input.tool_input || {};
    const toolResponse = input.tool_response || {};
    const filePath = toolInput.file_path;

    if (!filePath) {
        process.exit(0);
    }

    log(`Processing ${input.tool_name}: ${filePath}`);

    // Get the file content
    let content;
    try {
        if (input.tool_name === 'Write') {
            // For Write, content is in tool_response
            content = toolResponse.content || '';
        } else {
            // For Edit, read the current file
            content = fs.readFileSync(filePath, 'utf-8');
        }
    } catch (e) {
        log(`Failed to read content: ${e.message}`);
        process.exit(0);
    }

    // Skip non-code files
    const ext = path.extname(filePath).toLowerCase();
    if (!['.ts', '.tsx', '.js', '.jsx', '.rs', '.vue', '.svelte'].includes(ext)) {
        process.exit(0);
    }

    // Validate with Parry
    if (!PARRY_BIN) {
        log('Parry binary not found, skipping validation');
        process.exit(0);
    }

    const parsedPath = path.parse(filePath);
    const tmpPath = path.join(parsedPath.dir, parsedPath.name + '.parry-tmp' + parsedPath.ext);

    try {
        fs.writeFileSync(tmpPath, content, 'utf-8');

        const result = spawnSync(PARRY_BIN, ['check', '--output', 'json', tmpPath], {
            encoding: 'utf-8',
            timeout: 30000,
            env: { ...process.env, PARRY_CONFIG }
        });

        const validation = JSON.parse(result.stdout || '{"passed":true,"issues":[]}');
        fs.unlinkSync(tmpPath);

        if (validation.passed) {
            log(`✓ ${filePath} - No issues`);
            process.exit(0);
        }

        const issues = validation.issues || [];
        if (issues.length === 0) {
            process.exit(0);
        }

        const { fixable, nonFixable } = categorizeIssues(issues);

        log(`Found ${issues.length} issues (${fixable.length} fixable, ${nonFixable.length} not fixable)`);

        // Log non-fixable issues
        if (nonFixable.length > 0) {
            console.error(formatIssues(nonFixable));
            logChange(filePath, nonFixable, false);
        }

        // Try to auto-fix fixable issues
        if (fixable.length > 0 && AUTO_FIX) {
            log(`Attempting auto-fix for ${fixable.length} issues...`);

            const { success, fixedContent } = autoFix(filePath, content);

            if (success && fixedContent && fixedContent !== content) {
                // Write the fixed content back to the file
                fs.writeFileSync(filePath, fixedContent, 'utf-8');
                console.error(`\n✓ Parry auto-fixed ${fixable.length} issue(s) in ${path.basename(filePath)}`);
                logChange(filePath, fixable, true);

                // Re-validate to see what's left
                const recheckResult = spawnSync(PARRY_BIN, ['check', '--output', 'json', filePath], {
                    encoding: 'utf-8',
                    timeout: 30000,
                    env: { ...process.env, PARRY_CONFIG }
                });

                const recheck = JSON.parse(recheckResult.stdout || '{"passed":true,"issues":[]}');
                if (!recheck.passed && recheck.issues?.length > 0) {
                    const remaining = recheck.issues.filter(i => isFixable(i));
                    if (remaining.length > 0) {
                        console.error(`\n⚠️  ${remaining.length} issue(s) remain (needs manual fix)`);
                    }
                }
            }
        }

        process.exit(0);
    } catch (e) {
        log(`Validation error: ${e.message}`);
        process.exit(0);
    }
}

if (require.main === module) {
    main();
}
