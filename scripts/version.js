#!/usr/bin/env node

/**
 * Version generation script for Claude Code Rust
 *
 * This script generates version numbers based on git tags and commit history.
 * It follows semantic versioning and updates both Cargo.toml and package.json.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Configuration
const ROOT_DIR = path.resolve(__dirname, '..');
const CARGO_TOML_PATH = path.join(ROOT_DIR, 'Cargo.toml');
const PACKAGE_JSON_PATH = path.join(ROOT_DIR, 'package.json');

/**
 * Get current version from Cargo.toml
 */
function getCurrentVersionFromCargo() {
    try {
        const content = fs.readFileSync(CARGO_TOML_PATH, 'utf8');
        const match = content.match(/version\s*=\s*"([^"]+)"/);
        if (match) {
            return match[1];
        }
    } catch (err) {
        console.error('Error reading Cargo.toml:', err.message);
    }
    return '0.1.0'; // Default version
}

/**
 * Get latest git tag
 */
function getLatestGitTag() {
    try {
        const tag = execSync('git describe --tags --abbrev=0 2>/dev/null || echo ""', {
            cwd: ROOT_DIR,
            encoding: 'utf8'
        }).trim();
        return tag || null;
    } catch (err) {
        return null;
    }
}

/**
 * Get commit history since last tag
 */
function getCommitsSinceLastTag() {
    const latestTag = getLatestGitTag();
    try {
        const range = latestTag ? `${latestTag}..HEAD` : 'HEAD';
        const commits = execSync(`git log ${range} --oneline --no-decorate`, {
            cwd: ROOT_DIR,
            encoding: 'utf8'
        }).trim().split('\n').filter(line => line);
        return commits;
    } catch (err) {
        return [];
    }
}

/**
 * Parse semantic version
 */
function parseSemver(version) {
    // Remove 'v' prefix if present
    version = version.replace(/^v/, '');

    const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-([\w.-]+))?(?:\+([\w.-]+))?$/);
    if (!match) {
        throw new Error(`Invalid semver format: ${version}`);
    }

    return {
        major: parseInt(match[1], 10),
        minor: parseInt(match[2], 10),
        patch: parseInt(match[3], 10),
        prerelease: match[4] || null,
        build: match[5] || null
    };
}

/**
 * Format semantic version
 */
function formatSemver(version) {
    let result = `${version.major}.${version.minor}.${version.patch}`;
    if (version.prerelease) {
        result += `-${version.prerelease}`;
    }
    if (version.build) {
        result += `+${version.build}`;
    }
    return result;
}

/**
 * Determine next version based on commit types
 */
function determineNextVersion(currentVersion, commits) {
    const current = parseSemver(currentVersion);

    // Default: increment patch
    const next = { ...current };

    // Check commit messages for feature or breaking changes
    let hasFeature = false;
    let hasBreaking = false;

    for (const commit of commits) {
        const message = commit.toLowerCase();

        // Check for breaking changes indicator
        if (message.includes('breaking change') || message.includes('!:')) {
            hasBreaking = true;
            break;
        }

        // Check for feature commits (conventional commits)
        if (message.startsWith('feat') || message.includes(':feat')) {
            hasFeature = true;
        }
    }

    if (hasBreaking) {
        next.major += 1;
        next.minor = 0;
        next.patch = 0;
        next.prerelease = null;
        next.build = null;
    } else if (hasFeature) {
        next.minor += 1;
        next.patch = 0;
        next.prerelease = null;
        next.build = null;
    } else {
        next.patch += 1;
        next.prerelease = null;
        next.build = null;
    }

    return formatSemver(next);
}

/**
 * Update version in Cargo.toml
 */
function updateCargoVersion(newVersion) {
    try {
        let content = fs.readFileSync(CARGO_TOML_PATH, 'utf8');

        // Update workspace.package version
        content = content.replace(
            /(\[workspace\.package\][\s\S]*?version\s*=\s*)"[^"]*"/,
            `$1"${newVersion}"`
        );

        // Update virtual package version
        content = content.replace(
            /(\[package\][\s\S]*?name\s*=\s*"claude-code-workspace"[\s\S]*?version\s*=\s*)"[^"]*"/,
            `$1"${newVersion}"`
        );

        fs.writeFileSync(CARGO_TOML_PATH, content, 'utf8');
        console.log(`Updated Cargo.toml version to ${newVersion}`);
    } catch (err) {
        console.error('Error updating Cargo.toml:', err.message);
        throw err;
    }
}

/**
 * Update version in package.json if it exists
 */
function updatePackageJsonVersion(newVersion) {
    try {
        if (!fs.existsSync(PACKAGE_JSON_PATH)) {
            console.log('package.json not found, skipping');
            return;
        }

        const packageJson = JSON.parse(fs.readFileSync(PACKAGE_JSON_PATH, 'utf8'));
        packageJson.version = newVersion;

        fs.writeFileSync(PACKAGE_JSON_PATH, JSON.stringify(packageJson, null, 2) + '\n', 'utf8');
        console.log(`Updated package.json version to ${newVersion}`);
    } catch (err) {
        console.error('Error updating package.json:', err.message);
        throw err;
    }
}

/**
 * Create git tag for the new version
 */
function createGitTag(newVersion) {
    try {
        const tagName = `v${newVersion}`;
        execSync(`git tag -a ${tagName} -m "Release ${tagName}"`, {
            cwd: ROOT_DIR,
            stdio: 'inherit'
        });
        console.log(`Created git tag: ${tagName}`);
        return tagName;
    } catch (err) {
        console.error('Error creating git tag:', err.message);
        throw err;
    }
}

/**
 * Main function
 */
async function main() {
    console.log('🚀 Claude Code Rust Version Generator');
    console.log('=====================================\n');

    try {
        // Get current state
        const currentVersion = getCurrentVersionFromCargo();
        console.log(`Current version (from Cargo.toml): ${currentVersion}`);

        const latestTag = getLatestGitTag();
        console.log(`Latest git tag: ${latestTag || 'none'}`);

        const commits = getCommitsSinceLastTag();
        console.log(`Commits since last tag: ${commits.length}`);

        if (commits.length > 0) {
            console.log('\nRecent commits:');
            commits.slice(0, 5).forEach(commit => console.log(`  ${commit}`));
            if (commits.length > 5) {
                console.log(`  ... and ${commits.length - 5} more`);
            }
        }

        // Determine next version
        const nextVersion = determineNextVersion(currentVersion, commits);
        console.log(`\nNext version: ${nextVersion}`);

        // Ask for confirmation
        const readline = require('readline');
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        const answer = await new Promise(resolve => {
            rl.question(`\nUpdate to version ${nextVersion}? (y/N): `, resolve);
        });
        rl.close();

        if (answer.toLowerCase() !== 'y') {
            console.log('Version update cancelled.');
            process.exit(0);
        }

        // Update files
        updateCargoVersion(nextVersion);
        updatePackageJsonVersion(nextVersion);

        // Create git tag
        const createTag = await new Promise(resolve => {
            const rl2 = readline.createInterface({
                input: process.stdin,
                output: process.stdout
            });
            rl2.question(`Create git tag v${nextVersion}? (y/N): `, answer => {
                rl2.close();
                resolve(answer.toLowerCase() === 'y');
            });
        });

        if (createTag) {
            createGitTag(nextVersion);
        }

        console.log('\n✅ Version update complete!');
        console.log(`\nNext steps:`);
        console.log(`  1. Commit the changes: git commit -am "chore: bump version to ${nextVersion}"`);
        console.log(`  2. Push the tag: git push origin v${nextVersion}`);
        console.log(`  3. Build and publish the package`);

    } catch (err) {
        console.error('\n❌ Error:', err.message);
        process.exit(1);
    }
}

// Run the script
if (require.main === module) {
    main();
}

module.exports = {
    getCurrentVersionFromCargo,
    getLatestGitTag,
    getCommitsSinceLastTag,
    determineNextVersion,
    updateCargoVersion,
    updatePackageJsonVersion,
    createGitTag
};