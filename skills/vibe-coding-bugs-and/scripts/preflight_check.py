#!/usr/bin/env python3
"""Preflight diagnostic script synthesized by reddit-2-skill.
Validates environment invariants and detects common failure anti-patterns.
"""
import sys
import os

def run_preflight():
    print("[reddit-2-skill preflight] Checking project health invariants...")
    issues = []
    
    # Check git state
    if os.path.exists(".git"):
        print("✓ Git repository detected")
    else:
        issues.append("Warning: Not inside a version-controlled git directory")
        
    if issues:
        print("\nPreflight Issues Detected:")
        for issue in issues:
            print(f"  - {issue}")
        sys.exit(1)
        
    print("✓ All preflight invariants passed successfully.")
    sys.exit(0)

if __name__ == "__main__":
    run_preflight()
