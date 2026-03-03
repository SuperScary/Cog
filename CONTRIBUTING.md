# Contributing to Cog

Thank you for your interest in contributing to Cog! This document provides guidelines and instructions for contributing to the project.

## Table of Contents
1. [RFC Process](#rfc-process)
2. [Contributor Expectations](#contributor-expectations)
3. [How to Add a Language](#how-to-add-a-language)
4. [Getting Started](#getting-started)
5. [Code Style & Standards](#code-style--standards)

---

## RFC Process

For significant changes, new features, or architectural adjustments, we follow a Request for Comments (RFC) process. This ensures that the community has a chance to discuss and refine major changes before they are implemented.

1.  **Open an Issue**: Start by opening an issue on GitHub to describe the proposed change. Use the `RFC` prefix in the title.
2.  **Draft a Proposal**: In the issue, provide a detailed description of:
    -   The problem being solved.
    -   The proposed solution.
    -   Potential drawbacks or alternatives.
3.  **Community Feedback**: Wait for feedback from maintainers and the community. This process may involve several iterations.
4.  **Approval**: Once a consensus is reached, a maintainer will mark the RFC as approved, and you can proceed with the implementation.

Minor bug fixes and small improvements do not require an RFC and can be submitted directly as Pull Requests.

---

## Contributor Expectations

To ensure the quality and consistency of the Cog project, we expect contributors to:

-   **Write Clean Code**: Follow Rust's idiomatic patterns and use `cargo fmt` to maintain consistent formatting.
-   **Include Tests**: Add unit tests or integration tests for new features and bug fixes whenever possible.
-   **Update Documentation**: Ensure that any changes to features or configuration are reflected in the documentation (e.g., `README.md`).
-   **Be Respectful**: Adhere to a professional and inclusive code of conduct in all interactions.
-   **Small PRs**: Prefer multiple small, focused Pull Requests over one massive change.

---

## How to Add a Language

Cog supports syntax highlighting and language-specific configurations through JSON files. To add support for a new language, follow these steps:

### 1. Create the Language Directory
Create a new directory under `languages/` named after the language (e.g., `languages/rust/`).

### 2. Define the Syntax (`syntax.json`)
Create a `syntax.json` file in your new directory. This file defines how the code should be highlighted using regular expressions.

**Structure of `syntax.json`:**
```json
{
  "name": "Language Name",
  "file_extensions": ["ext1", "ext2"],
  "rules": [
    { "scope": "comment.line", "pattern": "//.*$" },
    { "scope": "keyword.control", "pattern": "\\b(if|else|for|while)\\b" },
    { "scope": "string.double", "begin": "\"", "end": "\"" }
  ]
}
```
-   `name`: The display name of the language.
-   `file_extensions`: A list of file extensions associated with this language.
-   `rules`: An array of highlighting rules:
    -   `scope`: The token type (e.g., `keyword.control`, `comment.line`, `string.double`, `constant.numeric`).
    -   `pattern`: A regex pattern for single-line tokens.
    -   `begin`/`end`: Used for multi-line tokens like block comments or strings.

### 3. Define Language Configuration (`language-configuration.json`)
Create a `language-configuration.json` file to define editor behavior for the language.

**Structure of `language-configuration.json`:**
```json
{
  "comments": {
    "lineComment": "//",
    "blockComment": ["/*", "*/"]
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"]
  ],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "\"", "close": "\"" }
  ]
}
```
-   `comments`: Defines how the editor should handle commenting out lines.
-   `brackets`: Pairs that define logical blocks.
-   `autoClosingPairs`: Pairs that the editor should automatically close when the opening character is typed.

### 4. Verify Your Changes
Restart Cog and open a file with one of the extensions you defined. Ensure that the syntax highlighting and auto-closing features work as expected.

---

## Getting Started

1.  **Fork the Repository**: Create your own fork of Cog on GitHub.
2.  **Clone Locally**: `git clone https://github.com/your-username/Cog.git`
3.  **Create a Branch**: `git checkout -b feature/your-feature-name`
4.  **Make Changes**: Implement your feature or fix.
5.  **Run Tests**: `cargo test`
6.  **Submit a PR**: Push your branch to your fork and open a Pull Request against the `main` branch of the original repository.

---

## Code Style & Standards

-   Use `cargo fmt` before committing.
-   Use `cargo clippy` to check for common mistakes and idiomatic improvements.
-   Write descriptive commit messages.
-   Keep the dependency count low to maintain Cog's lightweight nature.
