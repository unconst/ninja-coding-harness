#!/usr/bin/env python3
import os
import subprocess

# Test the exact path that should be checked
swe_forge_path = "/Arbos/swe-forge/target/release/swe-forge"

print(f"Checking SWE-Forge binary path: {swe_forge_path}")
print(f"Path exists: {os.path.exists(swe_forge_path)}")
print(f"Is file: {os.path.isfile(swe_forge_path)}")
print(f"Is executable: {os.access(swe_forge_path, os.X_OK)}")

if os.path.exists(swe_forge_path):
    stat = os.stat(swe_forge_path)
    print(f"File size: {stat.st_size} bytes")
    print(f"Permissions: {oct(stat.st_mode)}")

    # Try to run it
    try:
        result = subprocess.run([swe_forge_path, "--help"], capture_output=True, text=True, timeout=5)
        print(f"Binary works: {result.returncode == 0}")
        if result.returncode != 0:
            print(f"Error output: {result.stderr}")
    except subprocess.TimeoutExpired:
        print("Binary test timed out")
    except Exception as e:
        print(f"Error running binary: {e}")

# Check the swe-forge directory
swe_forge_dir = "/Arbos/swe-forge"
print(f"\nChecking SWE-Forge directory: {swe_forge_dir}")
print(f"Directory exists: {os.path.exists(swe_forge_dir)}")

if os.path.exists(swe_forge_dir):
    print("Directory contents:")
    for item in sorted(os.listdir(swe_forge_dir)):
        item_path = os.path.join(swe_forge_dir, item)
        if os.path.isdir(item_path):
            print(f"  📁 {item}/")
        else:
            print(f"  📄 {item}")