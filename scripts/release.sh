#!/bin/bash
set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}🚀 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if version argument is provided
if [ $# -eq 0 ]; then
    print_error "Please provide a version number"
    echo "Usage: $0 <version> (e.g., $0 0.2.0)"
    exit 1
fi

VERSION=$1
TAG="v$VERSION"

print_step "Starting release process for version $VERSION"

# Check if we're on master/main branch
BRANCH=$(git symbolic-ref --short HEAD)
if [[ "$BRANCH" != "master" && "$BRANCH" != "main" ]]; then
    print_warning "You're not on master/main branch. Current branch: $BRANCH"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Release cancelled"
        exit 1
    fi
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    print_error "Working directory is not clean. Please commit or stash changes."
    exit 1
fi

# Update version in Cargo.toml
print_step "Updating Cargo.toml version to $VERSION"
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update version in Homebrew formula
print_step "Updating Homebrew formula version to $VERSION"
sed -i.bak "s/archive\/v.*\.tar\.gz/archive\/v$VERSION.tar.gz/" homebrew/nameforge.rb
rm homebrew/nameforge.rb.bak

# Run tests
print_step "Running tests"
cargo test

# Check formatting and clippy
print_step "Checking code formatting and linting"
cargo fmt --check
cargo clippy -- -D warnings

# Build release version
print_step "Building release version"
cargo build --release

# Commit changes
print_step "Committing version changes"
git add Cargo.toml homebrew/nameforge.rb
git commit -m "chore: bump version to $VERSION"

# Create and push tag
print_step "Creating and pushing tag $TAG"
git tag -a "$TAG" -m "Release $TAG"
git push origin "$BRANCH"
git push origin "$TAG"

print_success "Release process completed!"
print_step "Next steps:"
echo "1. 🔄 GitHub Actions will automatically create the release"
echo "2. 📦 Binaries will be built for all platforms"
echo "3. 🦀 Package will be published to crates.io"
echo "4. 🍺 Update the SHA256 in homebrew/nameforge.rb after the release is created"
echo "5. 📝 Consider creating a PR to homebrew-core with the formula"

print_warning "To publish to Homebrew core, you'll need to:"
echo "1. Fork https://github.com/Homebrew/homebrew-core"
echo "2. Add the formula to Formula/nameforge.rb"
echo "3. Create a PR with the correct SHA256 hash from the release tarball"