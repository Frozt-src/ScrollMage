# Third-party notices

Scroll Mage's original code, documentation, and original visual assets are proprietary and closed source under LICENSE. This does not change the licenses or copyright notices of third-party components.

Pinned dependencies are listed in source/Cargo.lock. Unmodified copies of upstream license and copyright files are included under source/vendor-licenses/ by exact package version. These include the dependencies' MIT, Apache, Unicode, Boost, and other applicable license and copyright files. They apply to the corresponding third-party components, not to Scroll Mage's original code.

Rust standard-library and toolchain copyright notices are included under source/vendor-licenses/rust-toolchain/. Preserve the applicable third-party license and copyright files with redistributed copies of this package.

The native executable also relies on Windows system libraries and the Microsoft C runtime installed on the host. Those components are not copied into this package. Source rebuild requires a compatible Rust MSVC toolchain and Windows linker.
