_default:
  @just --list

# Bumps the version of the package and updates the changelog
version:
  release-plz update

# Creates a new release
release:
  release-plz release --repo-url git@github.com:JorgeMayoral/ycm.git
