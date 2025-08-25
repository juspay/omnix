# Omnix Development Instructions

**ALWAYS follow these instructions first and fallback to additional search and context gathering only if the information here is incomplete or found to be in error.**

Omnix is a Rust-based CLI tool that supplements the Nix CLI to improve developer experience. The project requires Nix and uses a Nix flake-based development environment with multiple Rust crates organized as a workspace.

## Prerequisites and Environment Setup

### Essential Requirements
- **Nix Package Manager**: Install from [nixos.asia/en/install](https://nixos.asia/en/install) or run:
  ```bash
  curl -L https://nixos.org/nix/install | sh -s -- --daemon
  ```
- **direnv**: Install and set up according to [nixos.asia/en/direnv](https://nixos.asia/en/direnv)
- **Network Access**: Required for downloading dependencies and Nix packages

### Development Environment Setup
```bash
# 1. Clone and enter the repository
git clone <repo-url>
cd omnix

# 2. Activate direnv (this may take several minutes on first run)
direnv allow

# 3. Verify the environment is working
just --list
```

The `direnv allow` command will:
- Download and cache Nix dependencies
- Set up the development shell environment
- Configure all necessary environment variables
- Install development tools (just, bacon, etc.)

## Building and Running

### **CRITICAL**: Nix Environment Required
**DO NOT attempt to build with `cargo build`, `just run`, `just clippy`, or `just cargo-test` directly** - they will fail because the code requires compile-time environment variables (FLAKE_ADDSTRINGCONTEXT, TRUE_FLAKE, FALSE_FLAKE, FLAKE_METADATA, DEFAULT_FLAKE_SCHEMAS, INSPECT_FLAKE, NIX_SYSTEMS) set by the Nix development environment. Always use the Nix-based workflows or ensure `direnv` is properly activated.

### Primary Build Commands
```bash
# Full Nix build - NEVER CANCEL: Takes 10-15 minutes. Set timeout to 30+ minutes.
nix build --accept-flake-config

# Build and run the CLI
nix run --accept-flake-config

# For development with live reloading (recommended approach)
just watch          # or `just w` - requires Nix environment
just watch show .    # with arguments
```

### Alternative Build Testing (Network Limited)
When Nix installation is not possible due to network restrictions, you can verify project structure and some tooling:
```bash
# These work without Nix but will fail at compilation:
just --list                    # Shows available commands  
cargo check                    # Will fail with env var errors after ~30 seconds
just clippy                    # Will fail with same errors
```

### Development Workflow
```bash
# Start live development mode - NEVER CANCEL: Initial compile takes 3-5 minutes
just watch

# Run specific commands during development (requires active direnv shell)
nix run -- show .              # Direct nix run approach
nix run -- health .            # Health checking
```

## Testing and Quality Assurance

### **CRITICAL**: All Testing Requires Nix Environment
All test and quality assurance commands require the full Nix environment to be active.

### Running Tests
```bash
# Run all tests - NEVER CANCEL: Takes 5-10 minutes. Set timeout to 20+ minutes.
just cargo-test                # Requires direnv/Nix environment

# Run CI locally - NEVER CANCEL: Takes 15-30 minutes. Set timeout to 60+ minutes.
just ci                        # Full Nix-based CI

# Run CI with cargo in devshell (faster for development)
just ci-cargo                  # Requires direnv/Nix environment
```

### Code Quality Checks
```bash
# Run clippy linting - NEVER CANCEL: Fails after ~30 seconds without Nix. Set timeout to 10+ minutes with Nix.
just clippy                    # Requires direnv/Nix environment

# Run all pre-commit hooks and formatting (nixpkgs-fmt + rustfmt)
just pca                       # Requires direnv/Nix environment

# Build documentation  
just cargo-doc                 # Requires direnv/Nix environment
```

### **MANDATORY**: Pre-commit Validation
ALWAYS run these commands before committing changes or CI will fail:
```bash
just pca           # Auto-format the entire codebase (nixpkgs-fmt + rustfmt)
just clippy        # Lint checks with zero warnings policy
just cargo-test    # Ensure all tests pass
```

### Network-Limited Development
When Nix environment is not available, you can still:
```bash
# Verify project structure and documentation
ls -la                         # Inspect repository layout
cat justfile                   # Review available commands
cat README.md                  # Read project documentation
find . -name "*.rs" | head     # Explore Rust source files
```

## Documentation

### Website Documentation
```bash
# Preview documentation website
just doc run

# Check documentation for broken links
just doc check
```

### API Documentation
```bash
# Build Rust API documentation
just cargo-doc
```

## Project Structure and Navigation

## Project Structure and Navigation

### Key Directories  
- `crates/` - Rust workspace with 8 crates (~1MB total source):
  - `omnix-cli/` - Main CLI application (184K)
  - `nix_rs/` - Nix integration library (224K)
  - `omnix-ci/` - CI functionality (196K)
  - `omnix-health/` - Health checks (120K)
  - `omnix-gui/` - GUI components (152K) 
  - `omnix-init/` - Project initialization (72K)
  - `omnix-develop/` - Development environment (32K)
  - `omnix-common/` - Shared utilities (44K)
- `doc/` - Documentation source (~224K, emanote-based website)
- `nix/` - Nix modules and environment definitions
- `.github/workflows/` - CI/CD pipeline definitions
- `target/` - Rust build artifacts (auto-generated, can be large)

### Configuration Files
- `flake.nix` - Main Nix flake definition and dependency management
- `om.yaml` - Omnix configuration (CI, health, development settings)  
- `justfile` - Command definitions and development workflows
- `bacon.toml` - File watching configuration for live development
- `.envrc` - direnv setup that sources omnix's own development environment
- `Cargo.toml` - Rust workspace configuration
- `rust-toolchain.toml` - Rust compiler version specification

### Common File Locations
- Build outputs: `result/` (created by `nix build`)
- Test configurations: `crates/*/tests/`
- CI configuration: `om.yaml` (ci section)
- Module definitions: `nix/modules/flake/`
- Documentation source: `doc/*.md` and `doc/om/*.md`
- Registry templates: `crates/omnix-init/registry/`

### Code Organization Patterns
```bash
# Find all Rust source files  
find crates/ -name "*.rs" | wc -l     # ~200+ source files

# Explore main CLI structure
ls crates/omnix-cli/src/               # Main application logic

# Find all configuration files
find . -name "*.nix" -o -name "*.yaml" -o -name "*.toml" | head -10

# Review test organization  
find . -path "*/tests/*" -name "*.rs" | head -5
```

## Troubleshooting and Common Issues

### Build Failures
- **`environment variable 'FLAKE_ADDSTRINGCONTEXT' not defined`**: This error occurs when building without Nix environment. Run `direnv allow` first.
- **Cargo build fails with env var errors**: Use `nix build` or ensure `direnv` is properly activated
- **`just` commands fail with compilation errors**: All `just` commands that invoke `cargo` require the Nix development environment
- **Network timeout during `direnv allow`**: Builds require internet access for Nix packages from nixos.org and cache.nixos.asia

### Environment Setup Issues
- **`direnv: error .envrc is blocked`**: Run `direnv allow` in the repository root
- **`nix: command not found`**: Install Nix first using the installer or package manager
- **Slow first-time setup**: Initial `direnv allow` downloads ~1GB+ of dependencies

### Network and Cache Issues  
- **Builds taking > 30 minutes**: Check access to cache.nixos.asia for binary cache
- **DNS resolution failures**: Ensure access to nixos.org, github.com, and crates.io
- **Behind corporate firewall**: May need to configure proxy settings for Nix

### Performance Notes
- **First-time setup**: 15-20 minutes for Nix to download dependencies
- **Incremental builds**: 30 seconds to 2 minutes with `just watch` 
- **CI runs**: 15-30 minutes depending on system and cache availability
- **Compilation without Nix environment**: Fails after 20-30 seconds with env var errors

### Working in Network-Limited Environments
When full Nix installation is not possible:
1. **Read documentation thoroughly**: `cat README.md`, `cat justfile`, `find . -name "*.md"`
2. **Explore codebase structure**: `find . -name "*.rs"`, `ls -la crates/`
3. **Review configurations**: `cat flake.nix`, `cat om.yaml`, `cat Cargo.toml`
4. **Test basic tooling**: `just --list` (works), `cargo check` (fails fast with clear error)
5. **Document findings**: When network access is restored, validate all commands work

## Validation Scenarios

### **ESSENTIAL**: Verify Nix Environment First
Before attempting any builds, validate the environment:
```bash
# Check if direnv is active (should show environment variables)
env | grep -E "(FLAKE_|NIX_|TRUE_FLAKE|FALSE_FLAKE)"

# Verify nix command is available
nix --version

# Test basic just functionality
just --list
```

### After Making Code Changes (Full Validation)
**Only possible with complete Nix environment:**
1. **Environment validation**: `env | grep FLAKE_` should show variables
2. **Build validation**: `nix build --accept-flake-config` completes successfully  
3. **Basic functionality**: `nix run -- show .` displays project information
4. **Health check**: `nix run -- health .` passes all checks
5. **Live development**: `just watch` starts without compilation errors
6. **Full test suite**: `just cargo-test` passes all tests
7. **Linting**: `just clippy` shows no warnings

### Before Committing (Mandatory Checks)
**All require Nix environment:**
1. **Format code**: `just pca`
2. **Lint**: `just clippy`  
3. **Test**: `just cargo-test`
4. **Build**: `nix build --accept-flake-config`

### Manual Testing Scenarios (Post-Development)
After changes, manually verify functionality:
- **CLI help**: `nix run -- --help` shows current commands
- **Show command**: `nix run -- show .` displays flake information correctly
- **Health checks**: `nix run -- health .` reports system status
- **Init functionality**: `nix run -- init --help` shows template options

### Limited Validation (Network-Constrained Environments)
When Nix is not available, you can still validate:
1. **Project structure**: Verify files exist in expected locations
2. **Documentation completeness**: `find . -name "*.md" -exec wc -l {} +`
3. **Source code organization**: `find crates/ -name "*.rs" | wc -l`
4. **Configuration validity**: `cat om.yaml` and `cat flake.nix` are parseable
5. **Dependency analysis**: `grep -r "env!" crates/` shows compile-time env var usage
6. **Build command syntax**: `just --show clippy` displays command correctly

### Error Detection Patterns
Learn to recognize these patterns:
- **Missing Nix environment**: `environment variable 'FLAKE_ADDSTRINGCONTEXT' not defined`
- **Network issues**: `curl: (6) Could not resolve host: nixos.org`  
- **direnv not activated**: `nix: command not found` in repository with .envrc
- **Cache miss**: Builds taking > 15 minutes when they should be < 5 minutes

## Time Expectations and Timeouts

**CRITICAL**: Never cancel long-running operations. Use these timeout values:

### Environment Setup Times
- **Initial direnv setup**: 15-20 minutes (first time only, downloads ~1GB+)
- **Subsequent direnv activation**: 5-10 seconds
- **Tool installation (just, bacon)**: 1-3 minutes each

### Build and Compilation Times  
- **Nix build (full)**: 10-15 minutes → Set timeout to 30+ minutes
- **Nix build (with cache)**: 2-5 minutes → Set timeout to 15+ minutes
- **Cargo compilation (with Nix env)**: 3-8 minutes → Set timeout to 20+ minutes
- **Cargo compilation (without Nix env)**: Fails after 20-30 seconds with env var errors

### Testing and Quality Assurance Times
- **Full CI run (`just ci`)**: 15-30 minutes → Set timeout to 60+ minutes
- **Cargo tests (`just cargo-test`)**: 5-10 minutes → Set timeout to 20+ minutes
- **Clippy linting (`just clippy`)**: 2-3 minutes with Nix env → Set timeout to 10+ minutes
- **Pre-commit hooks (`just pca`)**: 1-2 minutes → Set timeout to 5+ minutes

### Development Workflow Times
- **Live reloading (`just watch`)**: 30 seconds - 2 minutes per change
- **Documentation build**: 2-5 minutes → Set timeout to 15+ minutes
- **Single command execution**: 5-30 seconds depending on complexity

### Network-Dependent Operations
- **Package downloads**: Highly variable, 1-20 minutes depending on connection
- **Cache fetching**: 30 seconds - 5 minutes from cache.nixos.asia
- **Git operations**: Usually < 1 minute for this repository size

### Fast Operations (< 1 minute)
These should complete quickly and may indicate issues if they take longer:
- `just --list`
- `nix --version`
- `cat` commands for documentation
- Basic file system operations (`ls`, `find`, `grep`)

## Network and Dependency Requirements

### Required Network Access
- `nixos.org` - Nix package downloads and documentation
- `cache.nixos.org` - Default Nix binary cache  
- `cache.nixos.asia` - Custom project cache (oss channel)
- `github.com` - Git dependencies and source repositories
- `crates.io` - Rust dependencies (downloaded by Nix)

### Network-Free Operations (Safe to Run Always)
These commands work without network access and don't modify files:
```bash
# Repository exploration (always safe)
ls -la                         # List repository contents
find . -name "*.md" -exec wc -l {} +    # Count documentation lines
find crates/ -name "*.rs" | wc -l       # Count Rust source files 
du -sh crates/*                # Show crate sizes

# Configuration inspection (always safe)
cat justfile                   # View available commands
cat README.md                  # Read project documentation  
cat om.yaml                    # View omnix configuration
cat flake.nix                  # View Nix flake definition
cat Cargo.toml                 # View Rust workspace setup

# Tool verification (safe, may install tools)
just --version                 # Check just installation
just --list                    # Show available commands (works without Nix)
bacon --help                   # Check bacon availability (if installed)
cargo --version                # Check Rust toolchain

# Code analysis (safe, read-only)
grep -r "env!" crates/         # Find compile-time environment variables
find . -name "*.nix" | head    # List Nix configuration files
git log --oneline -5           # Recent commit history
```

### Commands That Require Network (Use With Caution)
These will fail in network-limited environments:
```bash
# Environment setup (requires network)
direnv allow                   # Downloads Nix dependencies
nix build                      # Requires Nix packages
cargo install just             # Downloads from crates.io

# Any compilation (requires network for dependencies) 
cargo check                    # Will download crates if cache empty
just clippy                    # Requires full environment
just cargo-test                # Requires full environment
```

### Offline Development Workflow
1. **With network**: Complete initial setup (`direnv allow`)
2. **Offline**: Limited development is possible using cached dependencies
3. **Validation**: Most build and test operations still require network for Nix evaluation

### Cache and Storage Requirements
- **Initial download**: ~1-2GB for complete development environment
- **Build cache**: ~500MB-1GB for compiled artifacts in `target/`
- **Nix store**: ~2-4GB for all dependencies and tools
- **Repository**: ~200MB source code (plus build artifacts)