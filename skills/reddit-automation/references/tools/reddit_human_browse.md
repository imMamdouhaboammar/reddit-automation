# reddit_human_browse

Browse Reddit URLs using the Obscura stealth anti-detection engine and natural human behavior simulation.

## Overview
- **Obscura Stealth Engine**: Neutralizes bot detection by overriding `navigator.webdriver`, spoofing `window.chrome`, emulating Apple M-Series WebGL rendering contexts, and applying isolated profile sandboxing.
- **Natural Human Behavior**:
  - **Reading Hesitation**: Simulates human reading pauses (800ms - 2200ms) with jitter.
  - **Momentum Scrolling**: Implements stepped scrolling with deceleration physics and frame jitter.
  - **Humanized Typing**: Variable keystroke delays (45ms-110ms) with pauses on spaces and punctuation.
- **Semantic Text Snapshots (agent-browser inspired)**:
  - Generates token-efficient accessibility trees tagged with `@e1`, `@e2`, `@e3...` references.
  - Allows AI agents to interact with buttons, links, inputs, and posts without consuming thousands of tokens on messy DOM HTML.

---

## Tool Signature

```json
{
  "name": "reddit_human_browse",
  "description": "Stealth browser navigation with human behavior emulation (anti-detection flags, Keychain cookie restoration, humanized reading hesitation, smooth momentum scrolling) returning a compact semantic element snapshot (@e1, @e2 references) for AI agent interaction.",
  "inputSchema": {
    "type": "object",
    "required": ["url"],
    "properties": {
      "account": {
        "type": "string",
        "description": "Optional account alias. Restores Keychain cookies if provided."
      },
      "url": {
        "type": "string",
        "description": "Target Reddit URL to browse (e.g. 'https://www.reddit.com/r/rust')."
      },
      "headless": {
        "type": "boolean",
        "description": "Run in headless mode (default: true)."
      }
    }
  }
}
```

---

## Example Usage

### Request
```json
{
  "account": "personal",
  "url": "https://www.reddit.com/r/rust",
  "headless": true
}
```

### Response
```json
{
  "title": "Rust Programming Language: Announcements, Discussions, and Questions",
  "url": "https://www.reddit.com/r/rust",
  "element_count": 28,
  "tree_text": "[Page: Rust Programming Language] (URL: https://www.reddit.com/r/rust)\n  - @e1 [LINK] \"Home\" -> /r/all\n  - @e2 [INPUT] \"Search Reddit\"\n  - @e3 [SHREDDIT-POST] \"What's everyone working on this week?\"\n    - @e4 [LINK] \"Comments (84)\" -> /r/rust/comments/...\n    - @e5 [BUTTON] \"Upvote (142)\"",
  "elements": [
    { "ref_id": "@e1", "tag": "link", "label": "Home", "href": "/r/all" },
    { "ref_id": "@e2", "tag": "input", "label": "Search Reddit", "href": null },
    { "ref_id": "@e3", "tag": "shreddit-post", "label": "What's everyone working on this week?", "href": null }
  ]
}
```
