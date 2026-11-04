# Playbook: Community Building & 90/10 Ratio Auditing

> **Role:** Reputation Guardian & Community Strategist  
> **Primary Tool:** `reddit_audit_community_ratio`  
> **Core Principle:** Reddit penalizes accounts that exist solely to promote. Maintain at least 90% non-promotional, organic value contributions.

---

## 1. The 90/10 Content-to-Promotion Rule

Reddit moderators, community spam filters, and platform anti-abuse algorithms enforce the standard 90/10 rule:

$$\text{Value Percentage} = \frac{\text{Value Engagements}}{\text{Total Engagements}} \times 100 \ge 90.0\%$$

```
   ┌──────────────────────────────────────────────────────────────────┐
   │ 90% Organic Value Contributions                                  │
   │ - Answering questions in your domain without mentioning products │
   │ - Sharing open-source scripts, benchmarks, or teardowns          │
   │ - Participating in AMAs, feedback requests, and discussions      │
   ├───────────────────────────────┬──────────────────────────────────┤
   │ 10% Disclosed Product Mentions│ Must satisfy the 3-part gate:    │
   │                               │ (1) User is asking               │
   │                               │ (2) Perfect product fit          │
   │                               │ (3) Honest affiliation disclosure│
   └───────────────────────────────┴──────────────────────────────────┘
```

---

## 2. Running the Ratio Audit

Call `reddit_audit_community_ratio`:
```json
{
  "account": "personal"
}
```

### Audit Response Analysis

```json
{
  "account": "personal",
  "reddit_username": "founder_bob",
  "link_karma": 120,
  "comment_karma": 1840,
  "audit": {
    "total_engagements": 45,
    "value_add_count": 42,
    "promotional_count": 3,
    "value_percentage": 93.3,
    "is_compliant": true,
    "advisory": "Account ratio is healthy and compliant (93.3% value-add). Safe to engage."
  }
}
```

### Decision Matrix:
- If `is_compliant == true` ($\ge 90\%$): The agent may proceed with opportunity engagement following the standard drafting gate.
- If `is_compliant == false` ($< 90\%$): **Promotion Freeze!** The agent must halt all product-naming replies and switch exclusively to pure-value help mode until the ratio recovers above 90%.

---

## 3. Subreddit Sentiment Monitoring & Brand Remediation

When monitoring target subreddits, the agent tracks brand mentions:

### Sentiment Classifications:
1. `Positive`: Organic praise or recommendation from third parties.
   - *Action:* No intrusion needed; optional brief genuine thank you.
2. `Neutral`: Factual listing or feature inquiry.
   - *Action:* Provide objective technical clarification if asked.
3. `Negative`: User complaint, friction, or bug report.
   - *Action:* Implement **Non-Defensive Remediation Protocol**:
     - Acknowledge the user's frustration immediately without excuses.
     - Cite the exact root cause or fix currently implemented.
     - Offer an immediate workaround or direct resolution channel.
4. `CriticalCrisis`: Widespread controversy or security vulnerability accusation.
   - *Action:* **Halt all automation immediately!** Alert human operator with high-priority incident dossier.
