Name:       mdtoview
Version:    0.1.0
Release:    1
Summary:    AI-powered live Markdown preview desktop app
License:    MIT
URL:        https://github.com/SourabhDhalia/rustmd-viewer-for-llm
Source0:    %{name}-%{version}.tar.gz

BuildRequires: rust
BuildRequires: cargo
BuildRequires: cmake >= 3.16
BuildRequires: gcc-arm-linux-gnueabihf
BuildRequires: binutils-arm-linux-gnueabihf

Requires: glibc
Requires: libgcc

%description
mdToView is a split-pane desktop Markdown editor written in Rust.
Left: paste raw Markdown. Right: live rendered preview with LaTeX-to-Unicode
conversion and syntax-highlighted code blocks.
Includes an AI assistant panel supporting Claude, Ollama (local), and
OpenAI-compatible endpoints.
All Rust dependencies are vendored — fully offline build.

# ─────────────────────────────────────────────────────────────────────────────
%prep
%setup -q

# Install the Rust armv7 cross-compile target (requires network on first run;
# subsequent GBS builds use the cached rustup toolchain inside the buildroot).
rustup target add armv7-unknown-linux-gnueabihf || true

# ─────────────────────────────────────────────────────────────────────────────
%build
# Cross-compile for Tizen armv7l using the standard gnueabihf target.
# The vendored/ directory ensures no network access is needed for crates.

export CARGO_TARGET=armv7-unknown-linux-gnueabihf

# Tell the linker where the ARM cross-compiler lives
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

cmake \
    -DCARGO_OFFLINE=ON  \
    -DCARGO_RELEASE=ON  \
    -DCARGO_TARGET=${CARGO_TARGET} \
    -DCMAKE_INSTALL_PREFIX=%{_prefix} \
    .

cmake --build . --target mdtoview_cargo

# ─────────────────────────────────────────────────────────────────────────────
%install
rm -rf %{buildroot}
cmake --install . --prefix %{buildroot}%{_prefix}

install -d %{buildroot}%{_datadir}/applications
install -m 644 packaging/mdtoview.desktop \
              %{buildroot}%{_datadir}/applications/mdtoview.desktop

# ─────────────────────────────────────────────────────────────────────────────
%clean
rm -rf %{buildroot}

# ─────────────────────────────────────────────────────────────────────────────
%files
%defattr(-,root,root,-)
%{_bindir}/mdtoview
%{_datadir}/applications/mdtoview.desktop

# ─────────────────────────────────────────────────────────────────────────────
%changelog
* Fri Apr 18 2026 Sourabh Dhalia <manshadhalia@gmail.com> - 0.1.0-1
- Initial Tizen/GBS armv7l packaging
