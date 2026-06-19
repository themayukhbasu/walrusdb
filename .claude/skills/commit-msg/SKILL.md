---
name: commit-msg
description: Suggest a one-liner conventional commit message based on staged changes. Use when the user asks for a commit message or wants to commit their changes.
---

# Commit Message Generator

## Instructions

1. Run `git diff --staged` and `git status` to see what's staged.
2. Suggest a single-line conventional commit message using the appropriate prefix: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
3. Output only the commit message — no explanation, no quotes.