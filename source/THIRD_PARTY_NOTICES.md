# Third-party notices

Scroll Mage's original code and documentation are licensed under the MIT License in LICENSE, and its original artwork under CC BY 4.0 as described in LICENSE-ARTWORK.md at the repository root. This does not change the licenses or copyright notices of third-party components.

Pinned dependencies are listed in Cargo.lock. Unmodified copies of upstream license and copyright files are included under vendor-licenses/ by exact package version. These include the dependencies' MIT, Apache, Unicode, Boost, and other applicable license and copyright files. They apply to the corresponding third-party components, not to Scroll Mage's original code.

Rust standard-library and toolchain copyright notices are included under vendor-licenses/rust-toolchain/. Preserve the applicable third-party license and copyright files with redistributed copies of this package.

The native executable also relies on Windows system libraries and the Microsoft C runtime installed on the host. Those components are not copied into this package. Source rebuild requires a compatible Rust MSVC toolchain and Windows linker.
