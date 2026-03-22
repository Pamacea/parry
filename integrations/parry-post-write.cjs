#!/usr/bin/env node
/**
 * Parry Post-Write Hook for Claude Code
 *
 * Validates and auto-corrects code after Write/Edit operations.
 *
 * INSTALLATION:
 * 1. Copy to ~/.claude/hooks/parry-post-write.cjs
 * 2. Add to ~/.claude/settings.json:
 *    {
 *      "hooks": {
 *        "PostToolUse": [{
 *          "hooks": [{
 *            "command": "node ~/.claude/hooks/parry-post-write.cjs",
 *            "timeout": 10000,
 *            "type": "command"
 *          }],
 *          "matcher": "Write|Edit"
 *        }]
 *      }
 *    }
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const PARRY_BIN = process.env.PARRY_BIN || 'parry';
const PARRY_AUTO_FIX = process.env.PARRY_AUTO_FIX !== 'false';
const PARRY_DEBUG = process.env.PARRY_DEBUG === 'true';
const PARRY_CONFIG = process.env.PARRY_CONFIG;

// Valid file extensions to check
const VALID_EXTENSIONS = ['.ts', '.tsx', '.js', '.jsx', '.rs'];

// Paths to exclude
const EXCLUDE_PATHS = [
  'node_modules',
  '.git',
  'target',
  'dist',
  'build',
  '.next',
  '.claude',
];

/**
 * Check if a file should be validated
 */
function shouldValidate(filePath) {
  const ext = path.extname(filePath);
  if (!VALID_EXTENSIONS.includes(ext)) return false;

  for (const exclude of EXCLUDE_PATHS) {
    if (filePath.includes(exclude)) return false;
  }

  return true;
}

/**
 * Run Parry validation on a file
 */
function validateFile(filePath) {
  const cmd = [PARRY_BIN, 'check', filePath, '--fix'];

  if (PARRY_CONFIG) {
    cmd.push('--config', PARRY_CONFIG);
  }

  if (PARRY_DEBUG) {
    console.error(`[Parry] Running: ${cmd.join(' ')}`);
  }

  try {
    const output = execSync(cmd.join(' '), {
      encoding: 'utf-8',
      stdio: PARRY_DEBUG ? 'inherit' : 'pipe',
      timeout: 10000,
    });

    if (PARRY_DEBUG && output) {
      console.error('[Parry] Output:', output);
    }

    return { success: true, output };
  } catch (error) {
    // Parry found issues but may have auto-fixed them
    const stdout = error.stdout || '';
    const stderr = error.stderr || '';

    if (PARRY_DEBUG) {
      console.error('[Parry] Validation failed:', error.status);
      console.error('[Parry] Stdout:', stdout);
      console.error('[Parry] Stderr:', stderr);
    }

    // Exit code 1 means validation failed but fixes may have been applied
    return {
      success: false,
      output: stdout || stderr,
      exitCode: error.status || 1,
    };
  }
}

/**
 * Parse tool input from Claude Code
 */
function parseToolInput() {
  const input = process.argv[2] || '{}';

  try {
    return JSON.parse(input);
  } catch {
    // If not JSON, check if it's a file path directly
    if (fs.existsSync(input) && shouldValidate(input)) {
      return { files: [input] };
    }
    return { files: [] };
  }
}

/**
 * Main hook logic
 */
function main() {
  if (PARRY_DEBUG) {
    console.error('[Parry] Post-write hook started');
    console.error('[Parry] Args:', process.argv);
  }

  // Try to get the file that was written
  let filesToCheck = [];

  // Method 1: Parse from command line argument (JSON from Claude Code)
  const toolInput = parseToolInput();

  // Method 2: Check for files argument
  if (toolInput.files && Array.isArray(toolInput.files)) {
    filesToCheck = toolInput.files.filter(shouldValidate);
  }

  // Method 3: Check stdout for file patterns (Write/Edit output)
  // This is a fallback when JSON parsing fails
  if (filesToCheck.length === 0) {
    const stdout = '';
    // In PostToolUse, we can sometimes get the file from the tool output
    // For now, skip if we can't determine the file
  }

  if (filesToCheck.length === 0) {
    if (PARRY_DEBUG) {
      console.error('[Parry] No files to validate');
    }
    process.exit(0);
  }

  if (PARRY_DEBUG) {
    console.error(`[Parry] Validating ${filesToCheck.length} file(s):`, filesToCheck);
  }

  let hasErrors = false;
  let hasFixes = false;

  for (const file of filesToCheck) {
    const result = validateFile(file);

    if (!result.success) {
      hasErrors = true;

      // Check if fixes were applied
      if (result.output && result.output.includes('Fixed')) {
        hasFixes = true;
      }

      // Print summary
      if (result.output) {
        console.error(`\n🔴 Parry found issues in ${path.basename(file)}:`);
        console.error(result.output);
      }
    }
  }

  if (hasFixes) {
    console.error('\n✓ Parry auto-fixed applicable issues.');
  }

  if (hasErrors && !hasFixes) {
    console.error('\n⚠️  Parry found issues that could not be auto-fixed.');
    // Don't fail the hook - just warn
  }

  if (PARRY_DEBUG) {
    console.error('[Parry] Hook completed');
  }

  process.exit(0); // Always succeed - don't block Claude
}

// Run
main();
