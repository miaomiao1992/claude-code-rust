#!/usr/bin/env node

/**
 * Build script for Claude Code Rust
 *
 * This script builds the Rust project and prepares the npm package.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');

// Configuration
const ROOT_DIR = path.resolve(__dirname, '..');
const BIN_DIR = path.join(ROOT_DIR, 'bin');
const DIST_DIR = path.join(ROOT_DIR, 'dist');
const TARGET_DIR = path.join(ROOT_DIR, 'target');

// Platform detection
const PLATFORM = os.platform();
const ARCH = os.arch();

const EXECUTABLE_EXTENSION = PLATFORM === 'win32' ? '.exe' : '';
const BINARY_NAME = 'claude';
const BRIDGE_BINARY_NAME = 'claude-bridge';

/**
 * Ensure directory exists
 */
function ensureDir(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

/**
 * Run cargo build
 */
function runCargoBuild(release = true) {
    console.log(`🚀 Building Rust project (${release ? 'release' : 'debug'})...`);

    const args = ['build'];
    if (release) {
        args.push('--release');
    }

    try {
        execSync(`cargo ${args.join(' ')}`, {
            cwd: ROOT_DIR,
            stdio: 'inherit'
        });
        console.log('✅ Rust build successful');
    } catch (err) {
        console.error('❌ Rust build failed');
        process.exit(1);
    }
}

/**
 * Copy binary files to bin directory
 */
function copyBinaries(release = true) {
    console.log(`📦 Copying binaries...`);

    const buildType = release ? 'release' : 'debug';
    const sourceDir = path.join(TARGET_DIR, buildType);

    // Ensure bin directory exists
    ensureDir(BIN_DIR);

    // Copy main binary
    const sourceBinary = path.join(sourceDir, `${BINARY_NAME}${EXECUTABLE_EXTENSION}`);
    const destBinary = path.join(BIN_DIR, BINARY_NAME + (PLATFORM === 'win32' ? '.exe' : ''));

    if (fs.existsSync(sourceBinary)) {
        fs.copyFileSync(sourceBinary, destBinary);

        // Make executable on Unix
        if (PLATFORM !== 'win32') {
            fs.chmodSync(destBinary, '755');
        }

        console.log(`  ✅ Copied ${BINARY_NAME} to bin/`);
    } else {
        console.error(`  ❌ Binary not found: ${sourceBinary}`);
        process.exit(1);
    }

    // Copy bridge binary if it exists
    const bridgeSource = path.join(sourceDir, `${BRIDGE_BINARY_NAME}${EXECUTABLE_EXTENSION}`);
    const bridgeDest = path.join(BIN_DIR, BRIDGE_BINARY_NAME + (PLATFORM === 'win32' ? '.exe' : ''));

    if (fs.existsSync(bridgeSource)) {
        fs.copyFileSync(bridgeSource, bridgeDest);

        if (PLATFORM !== 'win32') {
            fs.chmodSync(bridgeDest, '755');
        }

        console.log(`  ✅ Copied ${BRIDGE_BINARY_NAME} to bin/`);
    }

    // Create package.json in bin directory for npm
    const binPackageJson = {
        name: 'claude-code-rust-bin',
        version: require('../package.json').version,
        description: 'Binary distribution for Claude Code Rust',
        os: [PLATFORM],
        cpu: [ARCH],
        bin: {
            claude: `./${BINARY_NAME}${PLATFORM === 'win32' ? '.exe' : ''}`,
            'claude-code': `./${BINARY_NAME}${PLATFORM === 'win32' ? '.exe' : ''}`
        }
    };

    fs.writeFileSync(
        path.join(BIN_DIR, 'package.json'),
        JSON.stringify(binPackageJson, null, 2)
    );
}

/**
 * Create distribution files
 */
function createDistFiles() {
    console.log('📄 Creating distribution files...');

    ensureDir(DIST_DIR);

    // Create a simple JavaScript wrapper for npm
    const wrapperContent = `#!/usr/bin/env node

// Wrapper script for Claude Code Rust
// This file is generated during build

const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

const platform = process.platform;
const arch = process.arch;

// Path to binary
const binPath = path.join(__dirname, '..', 'bin', 'claude' + (platform === 'win32' ? '.exe' : ''));

if (!fs.existsSync(binPath)) {
    console.error('Claude Code binary not found. Please rebuild the package.');
    process.exit(1);
}

// Pass all arguments to the binary
const args = process.argv.slice(2);
const child = spawn(binPath, args, {
    stdio: 'inherit',
    shell: platform === 'win32'
});

child.on('error', (err) => {
    console.error('Failed to start Claude Code:', err.message);
    process.exit(1);
});

child.on('exit', (code) => {
    process.exit(code || 0);
});

// Handle signals
process.on('SIGINT', () => child.kill('SIGINT'));
process.on('SIGTERM', () => child.kill('SIGTERM'));
`;

    fs.writeFileSync(path.join(DIST_DIR, 'index.js'), wrapperContent);

    // Create README for dist
    const readmeContent = `# Claude Code Rust

This is the binary distribution of Claude Code Rust.

## Installation

\`\`\`bash
npm install claude-code-rust
\`\`\`

## Usage

\`\`\`bash
npx claude-code
# or
npx claude
\`\`\`

## Platform Support

Currently built for:
- Platform: ${PLATFORM}
- Architecture: ${ARCH}

For other platforms, please build from source.
`;

    fs.writeFileSync(path.join(DIST_DIR, 'README.md'), readmeContent);

    console.log('✅ Distribution files created');
}

/**
 * Main function
 */
async function main() {
    console.log('🔨 Claude Code Rust Build Script');
    console.log('================================\n');

    const args = process.argv.slice(2);
    const isDev = args.includes('--dev');
    const release = !isDev;

    try {
        // Run cargo build
        runCargoBuild(release);

        // Copy binaries
        copyBinaries(release);

        // Create distribution files
        createDistFiles();

        console.log('\n🎉 Build completed successfully!');
        console.log('\nNext steps:');
        console.log('  1. Test the binary: ./bin/claude --version');
        console.log('  2. Publish to npm: npm publish');
        console.log('  3. Create GitHub release');

    } catch (err) {
        console.error('\n❌ Build failed:', err.message);
        process.exit(1);
    }
}

// Run the script
if (require.main === module) {
    main();
}

module.exports = {
    runCargoBuild,
    copyBinaries,
    createDistFiles
};