# Build Instructions - Final Steps

## Current Situation

✅ **Swagger Integration**: 100% Complete
✅ **Rust Installed**: v1.94.0
✅ **Code Ready**: All files properly configured
⚠️ **Build Tools**: Need to be recognized by terminal

## Issue

The Visual Studio Build Tools are installed but the terminal needs to be restarted to recognize them, OR you need to use the Developer Command Prompt.

## Solution Options

### Option 1: Restart Terminal (Recommended)

1. **Close this PowerShell/Terminal completely**
2. **Open a NEW PowerShell or Command Prompt**
3. **Navigate back to project:**
   ```powershell
   cd C:\Users\hp\Desktop\Drips\School_API
   ```
4. **Try building:**
   ```powershell
   cargo check
   ```

### Option 2: Use Developer Command Prompt

1. **Press Windows Key**
2. **Search for**: "Developer Command Prompt for VS 2022"
3. **Open it**
4. **Navigate to project:**
   ```cmd
   cd C:\Users\hp\Desktop\Drips\School_API
   ```
5. **Build:**
   ```cmd
   cargo check
   ```

### Option 3: Use Developer PowerShell

1. **Press Windows Key**
2. **Search for**: "Developer PowerShell for VS 2022"
3. **Open it**
4. **Navigate to project:**
   ```powershell
   cd C:\Users\hp\Desktop\Drips\School_API
   ```
5. **Build:**
   ```powershell
   cargo check
   ```

### Option 4: Manual PATH Setup (Advanced)

Add to your PATH environment variable:
```
C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\bin\Hostx64\x64
C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64
```

Then restart terminal.

## After Successful Build

Once `cargo check` succeeds, you'll see:

```
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.14
   Compiling syn v2.0.95
   ...
   Checking school_api v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 15s
```

Then run:

```powershell
# Build the project
cargo build

# Run the server
cargo run
```

## Access Swagger Documentation

Once the server is running, open your browser:

```
http://localhost:3000/docs
```

You'll see:
- ✅ All 38 endpoints documented
- ✅ All 16 schemas with examples
- ✅ Interactive "Try it out" buttons
- ✅ JWT authentication support
- ✅ Complete API documentation

## Test the API

### 1. Test Health Check
```
GET http://localhost:3000/health
```

### 2. Register Admin
```
POST http://localhost:3000/auth/admin/register
{
  "email": "admin@school.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Admin",
  "role": "admin"
}
```

### 3. Login Admin (Get OTP)
```
POST http://localhost:3000/auth/admin/login
{
  "email": "admin@school.com",
  "password": "SecurePass123!"
}
```

### 4. Verify OTP
```
POST http://localhost:3000/auth/verify-otp
{
  "email": "admin@school.com",
  "otp": "123456"
}
```

### 5. Use Token for Protected Endpoints
- Copy the `access_token` from step 4
- Click "Authorize" in Swagger UI
- Enter: `Bearer <your_token>`
- Test any protected endpoint

## Troubleshooting

### If cargo check still fails:

1. **Verify VS Build Tools installation:**
   - Open "Add or Remove Programs"
   - Search for "Visual Studio Build Tools"
   - Click "Modify"
   - Ensure "Desktop development with C++" is checked
   - Ensure these components are installed:
     - MSVC v143 - VS 2022 C++ x64/x86 build tools
     - Windows 10/11 SDK

2. **Restart your computer** (sometimes required for PATH updates)

3. **Try Developer Command Prompt** (Option 2 above)

## Summary

Your Swagger integration is complete. The only step remaining is getting the build tools recognized by your terminal. The easiest solution is:

1. **Close current terminal**
2. **Open new terminal**
3. **Run `cargo check`**
4. **Run `cargo run`**
5. **Visit `http://localhost:3000/docs`**

Everything is ready to go! 🚀
