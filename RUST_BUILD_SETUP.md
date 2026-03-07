# Rust Build Setup for Windows

## Issue Found

Your Rust installation is missing the **Microsoft C++ Build Tools** (MSVC linker).

Error: `linker 'link.exe' not found`

## Solution: Install Visual Studio Build Tools

### Option 1: Visual Studio Build Tools (Recommended - Smaller Download)

1. **Download Visual Studio Build Tools:**
   - Visit: https://visualstudio.microsoft.com/downloads/
   - Scroll down to "All Downloads"
   - Find "Build Tools for Visual Studio 2022"
   - Download and run the installer

2. **During Installation:**
   - Select "Desktop development with C++"
   - Make sure these are checked:
     - MSVC v143 - VS 2022 C++ x64/x86 build tools
     - Windows 10/11 SDK
     - C++ CMake tools for Windows
   - Click Install (about 6-7 GB)

3. **Restart Your Terminal/PowerShell**

### Option 2: Full Visual Studio Community (Larger)

1. Download Visual Studio Community 2022
2. During installation, select "Desktop development with C++"
3. Install and restart

### Option 3: Using winget (Command Line)

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

## After Installation

### Verify the Setup:

```powershell
# Restart PowerShell first, then:
cargo check
```

### Expected Output:
```
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.14
   Compiling syn v2.0.95
   ...
   Checking school_api v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 15s
```

## Alternative: Use GNU Toolchain (Faster Setup)

If you don't want to install Visual Studio Build Tools, you can switch to the GNU toolchain:

### Step 1: Install GNU Toolchain
```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

### Step 2: Install MinGW-w64
Download from: https://www.mingw-w64.org/downloads/
Or use winget:
```powershell
winget install mingw-w64
```

### Step 3: Add to PATH
Add MinGW bin directory to your PATH environment variable.

## Current Status

✅ Rust installed: v1.94.0
✅ Cargo installed: v1.94.0
✅ Swagger integration: Complete
❌ C++ Build Tools: Missing (needed to compile)

## Next Steps

1. Install Visual Studio Build Tools (Option 1 above)
2. Restart PowerShell
3. Run: `cargo check`
4. Run: `cargo build`
5. Run: `cargo run`
6. Visit: http://localhost:3000/docs

## Why This Is Needed

Rust on Windows uses the MSVC (Microsoft Visual C++) toolchain by default, which requires:
- `link.exe` - The Microsoft linker
- Windows SDK - For system libraries
- MSVC compiler - For C/C++ dependencies

Many Rust crates have C/C++ dependencies that need to be compiled, which is why the C++ build tools are required.

## Estimated Time

- Visual Studio Build Tools download: 10-20 minutes
- Installation: 10-15 minutes
- Total: ~30 minutes

After this, your Rust environment will be fully functional and you can build the project with all the Swagger integration!
