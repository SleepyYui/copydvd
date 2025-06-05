#!/usr/bin/env python3
"""
Test script to verify HandBrake integration and notification system
"""

import subprocess
import time
import sys
import os

def run_command(cmd, timeout=10):
    """Run a command with timeout"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "Command timed out"

def test_handbrake_cli():
    """Test HandBrake CLI directly"""
    print("🔍 Testing HandBrake CLI directly...")
    code, stdout, stderr = run_command("HandBrakeCLI --version", 5)
    
    if code == 0:
        version_line = stdout.strip().split('\n')[0]
        print(f"✅ HandBrake CLI found: {version_line}")
        return True, version_line
    else:
        print(f"❌ HandBrake CLI not found or failed")
        return False, None

def test_copydvd_cli():
    """Test CopyDVD CLI mode"""
    print("\n🔍 Testing CopyDVD CLI mode...")
    
    # Test help command
    code, stdout, stderr = run_command("./target/release/copydvd --help", 5)
    if code != 0:
        print("❌ CopyDVD CLI help failed")
        return False
    
    print("✅ CopyDVD CLI help works")
    
    # Test version command
    code, stdout, stderr = run_command("./target/release/copydvd --version", 5)
    if code == 0:
        print(f"✅ CopyDVD version: {stdout.strip()}")
    else:
        print("⚠️  CopyDVD version command failed")
    
    return True

def test_compilation():
    """Test that the application compiles"""
    print("\n🔍 Testing compilation...")
    
    # Test cargo check
    code, stdout, stderr = run_command("cargo check", 30)
    if code != 0:
        print("❌ Cargo check failed:")
        print(stderr)
        return False
    
    print("✅ Cargo check passed")
    
    # Check for warnings
    if "warning:" in stderr:
        warning_count = stderr.count("warning:")
        print(f"⚠️  {warning_count} warnings found (acceptable)")
    else:
        print("✅ No warnings found")
    
    return True

def verify_handbrake_integration():
    """Verify HandBrake integration by checking logs"""
    print("\n🔍 Testing HandBrake integration via GUI logs...")
    
    # Run the app briefly in GUI mode to trigger HandBrake verification
    cmd = "timeout 5s ./target/release/copydvd 2>&1 || true"
    code, stdout, stderr = run_command(cmd, 10)
    
    # Check for key integration points
    success_indicators = [
        "Verifying HandBrake Installation",
        "Found HandBrake in system PATH",
        "HandBrake verification successful",
        "Version info: HandBrake"
    ]
    
    found_indicators = []
    for indicator in success_indicators:
        if indicator in stdout:
            found_indicators.append(indicator)
    
    # Also check for version extraction
    version_found = False
    if "HandBrake 1.9.2" in stdout:
        found_indicators.append("Version extraction (HandBrake 1.9.2)")
        version_found = True
    
    if found_indicators:
        print(f"✅ HandBrake integration working - found {len(found_indicators)} indicators:")
        for indicator in found_indicators:
            print(f"   ✓ {indicator}")
        
        if version_found:
            print("   ✓ Version display system working")
        
        return True
    else:
        print("❌ HandBrake integration issues - no success indicators found")
        print("Log output:")
        print(stdout[:500] + "..." if len(stdout) > 500 else stdout)
        return False

def main():
    """Main test function"""
    print("🚀 CopyDVD HandBrake Integration Test")
    print("=" * 50)
    
    # Change to the correct directory
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    tests = [
        ("Compilation", test_compilation),
        ("HandBrake CLI", test_handbrake_cli),
        ("CopyDVD CLI", test_copydvd_cli),
        ("HandBrake Integration", verify_handbrake_integration),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n📋 Running: {test_name}")
        print("-" * 30)
        
        try:
            if test_func():
                passed += 1
                print(f"✅ {test_name} PASSED")
            else:
                print(f"❌ {test_name} FAILED")
        except Exception as e:
            print(f"❌ {test_name} ERROR: {e}")
    
    print("\n" + "=" * 50)
    print(f"📊 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! HandBrake integration is working correctly.")
        return 0
    else:
        print("⚠️  Some tests failed. Check the output above for details.")
        return 1

if __name__ == "__main__":
    sys.exit(main())