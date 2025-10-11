
# List available commands
default:
  @just --list


# Build the project
build:
    meson compile -C builddir

# Clean the build directory
clean:
    meson compile -C builddir --clean

# Reset the build directory
reset:
    rm -rf builddir
    meson setup builddir

# Setup the build directory
setup-meson:
    meson setup builddir
