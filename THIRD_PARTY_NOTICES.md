# Third-party notices

Scroll Mage's original code and documentation are licensed under the MIT License in LICENSE, and its original artwork under CC BY 4.0 as described in LICENSE-ARTWORK.md. This does not change the licenses or copyright notices of third-party components.

Pinned dependencies are listed in source/Cargo.lock; candidate ZIPs include DEPENDENCIES.json with their versions. Unmodified copies of upstream license and copyright files are included under source/vendor-licenses/ in the repository and vendor-licenses/ beside the packaged executable, by exact package version. These include the dependencies' MIT, Apache, Unicode, Boost, and other applicable license and copyright files. They apply to the corresponding third-party components, not to Scroll Mage's original code.

Rust standard-library and toolchain copyright notices are included in the rust-toolchain/ subfolder of that license directory. Preserve the applicable third-party license and copyright files with redistributed copies of this package.

The release build links the Microsoft C runtime statically and relies on host Windows system libraries. Windows system DLLs are not copied into this package. Source rebuild requires a compatible Rust MSVC toolchain, Microsoft C++ build tools and Windows SDK. Applicable compiler/runtime redistribution terms still apply.
