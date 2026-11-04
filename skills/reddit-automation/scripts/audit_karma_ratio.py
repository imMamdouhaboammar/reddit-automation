#!/usr/bin/env python3
"""
audit_karma_ratio.py - Reddit 90/10 Rule Auditor

Evaluates an account's recent submissions and comments to verify that:
- At least 90% of activity is genuine organic community contribution.
- No more than 10% is promotional, product-mentioning, or link-carrying.
"""

import sys
import json
import argparse

def evaluate_ratio(total_posts: int, promo_posts: int, comment_karma: int, link_karma: int) -> dict:
    if total_posts == 0:
        ratio = 100.0
    else:
        organic_posts = total_posts - promo_posts
        ratio = (organic_posts / total_posts) * 100.0

    compliant = ratio >= 90.0 and comment_karma >= 50
    recommendations = []
    
    if ratio < 90.0:
        needed_organic = int((promo_posts * 9) - (total_posts - promo_posts))
        if needed_organic < 0:
            needed_organic = 0
        recommendations.append(
            f"Ratio is {ratio:.1f}% organic. Post at least {needed_organic} helpful organic comments before posting any more product links."
        )
    if comment_karma < 50:
        recommendations.append(
            f"Comment karma is {comment_karma} (< 50 threshold). Build up comment karma in technical discussion threads."
        )

    return {
        "compliant": compliant,
        "organic_ratio_pct": round(ratio, 1),
        "total_posts": total_posts,
        "promo_posts": promo_posts,
        "comment_karma": comment_karma,
        "link_karma": link_karma,
        "recommendations": recommendations
    }

def main():
    parser = argparse.ArgumentParser(description="Audit Reddit 90/10 community ratio.")
    parser.add_argument("--total", type=int, default=20, help="Total recent posts & comments analyzed")
    parser.add_argument("--promo", type=int, default=1, help="Number of promotional / product mentions")
    parser.add_argument("--comment-karma", type=int, default=120, help="Total comment karma")
    parser.add_argument("--link-karma", type=int, default=30, help="Total link karma")
    parser.add_argument("--json", action="store_true", help="Output JSON format")
    args = parser.parse_args()

    result = evaluate_ratio(args.total, args.promo, args.comment_karma, args.link_karma)

    if args.json:
        print(json.dumps(result, indent=2))
        sys.exit(0 if result["compliant"] else 1)

    print("=== Reddit 90/10 Community Ratio Audit ===")
    print(f"Organic Ratio: {result['organic_ratio_pct']}% (Threshold: >= 90%)")
    print(f"Total Evaluated Posts: {result['total_posts']} | Promotional Posts: {result['promo_posts']}")
    print(f"Comment Karma: {result['comment_karma']} | Link Karma: {result['link_karma']}")
    print(f"Status: {'COMPLIANT' if result['compliant'] else 'NON-COMPLIANT'}")

    if result["recommendations"]:
        print("\nAction Items:")
        for rec in result["recommendations"]:
            print(f"- {rec}")

    sys.exit(0 if result["compliant"] else 1)

if __name__ == "__main__":
    main()
