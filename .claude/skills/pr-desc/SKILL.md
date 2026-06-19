---
name: pr-desc
description: Generate a PR description and save it as a markdown file. Use when the user asks for a PR description, pull request summary, or wants to document branch changes.
---

# PR Description Generator

## Instructions

1. Run `git branch --show-current` to get the current branch name.
2. Run `git log main..HEAD --oneline` and `git diff main...HEAD` to understand all changes on this branch.
3. Write a PR description with:
   - **Summary**: 2-4 bullet points on what changed and why.
   - **Test plan**: bulleted checklist of how to verify the changes.
4. Create the directory `target/docs/` if it doesn't exist.
5. Save the description to `target/docs/PR_<branch_name>.md`.
6. Confirm the file path when done.

## Important
Do not add any attribution, "Generated with Claude Code", or similar lines to the output file.