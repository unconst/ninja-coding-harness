#!/usr/bin/env python3
import asyncio
import subprocess
import time
import signal
import os

async def test_async_hang():
    """Test to identify where the continuous loop hangs"""
    print("🔍 Starting hang detection test...")

    # Change to ninja directory
    os.chdir("/Arbos/ninja")

    # Start the process
    proc = subprocess.Popen([
        "cargo", "run", "--bin", "continuous-loop"
    ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

    start_time = time.time()
    timeout_seconds = 60  # 1 minute timeout

    last_output_time = start_time
    lines_seen = []

    try:
        while True:
            # Non-blocking read with timeout
            try:
                # Use select to check if output is available
                import select
                if select.select([proc.stdout], [], [], 0.1)[0]:
                    line = proc.stdout.readline()
                    if line:
                        current_time = time.time()
                        elapsed = current_time - start_time
                        since_last = current_time - last_output_time

                        print(f"[{elapsed:.1f}s] (+{since_last:.1f}s) {line.strip()}")
                        lines_seen.append((elapsed, line.strip()))
                        last_output_time = current_time
                    else:
                        # Process finished
                        break

                # Check for timeout
                if time.time() - last_output_time > 30:  # 30 seconds of no output
                    print(f"🚨 HANG DETECTED! No output for 30 seconds")
                    print(f"Last line seen: {lines_seen[-1] if lines_seen else 'None'}")
                    break

                # Check for total timeout
                if time.time() - start_time > timeout_seconds:
                    print(f"🚨 TOTAL TIMEOUT! Process ran for {timeout_seconds} seconds")
                    break

            except Exception as e:
                print(f"Error reading output: {e}")
                break

    finally:
        # Terminate the process
        try:
            proc.kill()
            proc.wait(timeout=5)
        except:
            pass

    print("\n📊 HANG ANALYSIS COMPLETE")
    print(f"Total lines of output: {len(lines_seen)}")
    if lines_seen:
        print(f"First line: {lines_seen[0]}")
        print(f"Last line: {lines_seen[-1]}")

        # Look for specific patterns
        for elapsed, line in lines_seen:
            if "Starting autonomous operation" in line:
                print(f"🎯 Autonomous operation started at {elapsed:.1f}s")
            elif "generate_and_solve_batch" in line:
                print(f"🎯 generate_and_solve_batch called at {elapsed:.1f}s")
            elif "load_existing_challenges" in line:
                print(f"🎯 load_existing_challenges called at {elapsed:.1f}s")

if __name__ == "__main__":
    asyncio.run(test_async_hang())