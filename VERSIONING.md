# Version Management System

This document describes the automated version management and release system for Claude Code Rust.

## Overview

The system provides automatic version number generation, cross-platform builds, and publishing to both npm and GitHub Releases. It follows Semantic Versioning (SemVer) and uses conventional commit messages to determine version bumps.

## Key Components

1. **Version Generator Script** (`scripts/version.js`)
   - Reads current version from Cargo.toml
   - Analyzes git commit history since last tag
   - Determines next version based on commit types
   - Updates both Cargo.toml and package.json
   - Creates git tags

2. **Build Script** (`scripts/build.js`)
   - Runs `cargo build --release`
   - Copies binaries to `bin/` directory
   - Creates distribution files

3. **GitHub Actions Workflow** (`.github/workflows/release.yml`)
   - Triggers on git tags (v*)
   - Builds for Linux, macOS, and Windows
   - Runs tests
   - Publishes to npm
   - Creates GitHub Releases with binaries

4. **npm Package** (`package.json`)
   - Binary distribution via npm
   - Platform-specific binaries
   - Wrapper scripts for CLI usage

## Versioning Rules

The system uses these rules to determine version bumps:

| Commit Type | Version Increment | Example Commit Message |
|-------------|-------------------|------------------------|
| Breaking change | MAJOR (1.0.0 → 2.0.0) | `feat!: breaking change` or includes "BREAKING CHANGE" |
| Feature | MINOR (1.0.0 → 1.1.0) | `feat: add new feature` |
| Fix | PATCH (1.0.0 → 1.0.1) | `fix: resolve issue` |
| Other | PATCH (1.0.0 → 1.0.1) | `chore: update dependencies` |

## Workflow

### 1. Development
```bash
# Make changes with conventional commit messages
git commit -m "feat: add new command"
git commit -m "fix: resolve crash in upgrade"
```

### 2. Generate New Version
```bash
# Run version generator (interactive)
npm run version

# Or non-interactive (for CI)
node scripts/version.js --non-interactive
```

The script will:
- Show current version and recent commits
- Calculate next version based on commit types
- Ask for confirmation
- Update Cargo.toml and package.json
- Optionally create git tag

### 3. Commit and Tag
```bash
# Commit version changes
git commit -am "chore: bump version to X.Y.Z"

# Create and push tag (if script didn't create it)
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### 4. Automated Release (GitHub Actions)
Pushing the tag triggers:
1. **Build**: Compiles for Linux, macOS, Windows
2. **Test**: Runs test suite
3. **Publish npm**: Publishes to npm registry
4. **GitHub Release**: Creates release with binaries and checksums

## Manual Release

If you need to release manually:

```bash
# Generate version
npm run version

# Build binaries
npm run build

# Publish to npm
npm run publish:npm

# Create GitHub release
npm run publish:github
```

## Setup Requirements

### 1. GitHub Secrets
Configure these secrets in your GitHub repository settings:

- `NPM_TOKEN`: npm access token with publish permissions
- `GH_TOKEN`: GitHub token with repo permissions (automatically provided)

### 2. npm Configuration
```bash
# Login to npm
npm login

# Set registry (if using custom registry)
npm config set registry https://registry.npmjs.org/
```

### 3. Git Configuration
```bash
# Enable signed commits (recommended)
git config commit.gpgsign true
```

## Configuration Files

### `package.json` Key Fields
- `version`: Synchronized with Cargo.toml
- `bin`: CLI entry points
- `files`: Files included in npm package
- `scripts`: Build and publish commands
- `publishConfig`: npm registry settings

### `Cargo.toml` Version Fields
- `[workspace.package].version`: Workspace version
- `[package].version`: Virtual package version
- Individual crate versions inherit from workspace

## Troubleshooting

### Version Mismatch
If Cargo.toml and package.json versions get out of sync:
```bash
# Run consistency check
npm run version-check
```

### Build Failures
```bash
# Clean build
npm run clean
npm run build
```

### Publish Issues
- Check npm authentication: `npm whoami`
- Verify package name availability
- Check GitHub token permissions

## Advanced Usage

### Custom Version Tags
```bash
# Use pre-release versions
node scripts/version.js --prerelease beta

# Use specific version
node scripts/version.js --version 1.2.3
```

### Platform-Specific Builds
```bash
# Build for specific target
cargo build --release --target x86_64-apple-darwin
```

### Local Testing
```bash
# Test release process locally
npm run build
npm pack --dry-run
```

## Support

For issues with the version management system:
1. Check GitHub Actions logs
2. Verify configuration files
3. Ensure all secrets are properly set
4. Check npm and GitHub API rate limits