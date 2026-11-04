#!/usr/bin/env python3
"""
verify_experience_grammar.py - Experience Grammar & Anti-Slop Linter

Validates draft responses for Reddit according to:
1. Length discipline: 25 - 55 words.
2. Experience Grammar anchors ("I ran into this", "When we built", "In my experience").
3. Honest engineering hedges ("YMMV", "depends on", "in our setup").
4. Mandatory affiliation disclosure ("Full disclosure:", "Disclosure:", "I work on", "I built").
5. Anti-slop prohibition: bans sales hype ("game changer", "revolutionary", "best tool", "look no further").
"""

import sys
import re
import argparse

EXPERIENCE_ANCHORS = [
    r"\bi ran into\b",
    r"\bwe ran into\b",
    r"\bin my experience\b",
    r"\bwhen we built\b",
    r"\bwe had this exact\b",
    r"\bi've seen\b",
    r"\bwe tested\b",
    r"\bi noticed that\b"
]

HEDGES = [
    r"\bymmv\b",
    r"\bdepends on\b",
    r"\bin our case\b",
    r"\bmight work\b",
    r"\btrade-off\b",
    r"\bworth testing\b",
    r"\bfor what it's worth\b"
]

DISCLOSURES = [
    r"\bfull disclosure\b",
    r"\bdisclosure:\b",
    r"\bi work on\b",
    r"\bi built\b",
    r"\bdisclaimer:\b",
    r"\bi'm affiliated with\b"
]

PROHIBITED_SLOP = [
    r"\bgame changer\b",
    r"\brevolutionary\b",
    r"\blook no further\b",
    r"\bbest tool out there\b",
    r"\bcheck out my\b",
    r"\bguaranteed to\b",
    r"\bseamlessly integrates\b",
    r"\bworld-class\b",
    r"\bsilver bullet\b"
]

def audit_text(text: str) -> dict:
    words = text.strip().split()
    word_count = len(words)
    lower = text.lower()
    
    issues = []
    
    # 1. Word count constraint
    if word_count < 25:
        issues.append(f"Too short ({word_count} words). Minimum required is 25 words.")
    elif word_count > 55:
        issues.append(f"Too long ({word_count} words). Maximum allowed is 55 words.")
        
    # 2. Experience anchor check
    has_anchor = any(re.search(pat, lower) for pat in EXPERIENCE_ANCHORS)
    if not has_anchor:
        issues.append("Missing Experience Grammar anchor (e.g. 'I ran into this...', 'When we built...').")
        
    # 3. Hedge check
    has_hedge = any(re.search(pat, lower) for pat in HEDGES)
    if not has_hedge:
        issues.append("Missing honest hedge or caveat (e.g. 'YMMV', 'depends on your setup', 'in our case').")
        
    # 4. Mandatory disclosure check
    has_disclosure = any(re.search(pat, lower) for pat in DISCLOSURES)
    if not has_disclosure:
        issues.append("Missing mandatory affiliation disclosure (e.g. 'Full disclosure: I work on X').")
        
    # 5. Anti-slop / sales hype check
    slop_matches = [pat for pat in PROHIBITED_SLOP if re.search(pat, lower)]
    if slop_matches:
        issues.append(f"Contains prohibited promotional slop: {slop_matches}")
        
    return {
        "valid": len(issues) == 0,
        "word_count": word_count,
        "has_anchor": has_anchor,
        "has_hedge": has_hedge,
        "has_disclosure": has_disclosure,
        "issues": issues
    }

def main():
    parser = argparse.ArgumentParser(description="Audit Reddit response against Experience Grammar.")
    parser.add_argument("text", nargs="?", help="Text string to evaluate")
    parser.add_argument("--file", "-f", help="Path to file containing draft response")
    args = parser.parse_args()

    content = ""
    if args.file:
        with open(args.file, "r", encoding="utf-8") as f:
            content = f.read()
    elif args.text:
        content = args.text
    else:
        if not sys.stdin.isatty():
            content = sys.stdin.read()
        else:
            parser.print_help()
            sys.exit(1)

    result = audit_text(content)
    
    print(f"Word count: {result['word_count']} words")
    print(f"Experience Anchor: {'PASS' if result['has_anchor'] else 'FAIL'}")
    print(f"Hedge / Trade-off: {'PASS' if result['has_hedge'] else 'FAIL'}")
    print(f"Disclosure: {'PASS' if result['has_disclosure'] else 'FAIL'}")
    
    if result["valid"]:
        print("\nRESULT: PASS (Clean Experience Grammar)")
        sys.exit(0)
    else:
        print("\nRESULT: FAIL")
        for issue in result["issues"]:
            print(f"- {issue}")
        sys.exit(1)

if __name__ == "__main__":
    main()
