# AGENTS.md

A comprehensive guide for AI coding agents working on the inst-upd project.

## Project Overview

This is a Rust-based industrial automation project that uses roboPLC for camera streaming, Telegram bot integration, and WebSocket communication. The project supports cross-platform compilation for ARM64 Linux targets and is designed for deployment on Raspberry Pi devices.

## Build and Development Commands

### Cross-Platform Build (Required on macOS)
- Build: `cross build --release --target aarch64-unknown-linux-gnu`
- Check: `cross check --target aarch64-unknown-linux-gnu`
- Test: `cross test --target aarch64-unknown-linux-gnu --release`
- Run: `cross run --target aarch64-unknown-linux-gnu`

### Just Commands (Available)
- Copy binary: `just c_p` (copy binary to Raspberry Pi)
- Build and flash: `just bf` (build and flash to device)
- SSH to device: `just ssh` (SSH into Raspberry Pi)

### Standard Cargo Commands
- Build: `cargo build --release --target aarch64-unknown-linux-gnu`
- Check: `cargo check --target aarch64-unknown-linux-gnu`
- Test: `cargo test --target aarch64-unknown-linux-gnu --release`
- Format: `cargo +nightly fmt` (MANDATORY before commits)

## Code Quality Guidelines

### Critical Thinking Checklist (MANDATORY)
Before proposing any solution, complete this checklist:

1. **Question the problem itself**: Is the complexity actually necessary? Can we eliminate the need entirely?
2. **Find the simplest possible solution first**: What would require the least code changes to solve this?
3. **Challenge every assumption**: Why does the current code work this way? Is it still relevant?
4. **Look at actual usage**: How is this code really used? Can we optimize for the common cases?
5. **Design the API before implementation**: What would be the cleanest interface for this functionality?

### Problem-Solving Priority Order (MUST follow in sequence):
1. **ELIMINATE** - Can we remove the problem entirely by restructuring?
2. **SIMPLIFY** - Can we solve this trivially by changing the approach?
3. **REUSE** - Does existing code already solve this?
4. **Only then: CREATE** - Write new solution only if above options fail

### Complexity Warning Signs 🚨
If you find yourself doing ANY of these, STOP and reconsider:
- Using `Box<dyn Any>` or type erasure
- Creating new trait abstractions for a single use case
- Writing more than 20 lines to work around a problem
- Adding generic parameters that propagate through multiple layers
- Introducing runtime type checking or downcasting
- Making something generic "just in case"
- Creating abstractions on top of abstractions

### Code Style Requirements
- Verify information before presenting it
- Make changes file by file
- Never use apologies
- Avoid feedback about understanding
- Don't suggest whitespace changes
- Don't summarize changes made
- Don't invent changes other than what's explicitly requested
- Don't ask for confirmation of information already provided
- Preserve existing code and structures
- Provide all edits in a single chunk
- Don't ask to verify implementations that are visible in context
- Don't suggest updates when no modifications are needed
- Provide links to real files, not x.md
- Don't show current implementation unless specifically requested

### Code Formatting Requirements
- **Always run `cargo fmt` before committing code changes**
- **Run `cargo fmt` after making any code modifications**
- **MANDATORY: Run `cargo fmt` before any git commit**
- Ensure consistent code formatting across the entire codebase
- Format all Rust files before submitting pull requests
- Verify formatting is applied to all modified files
- Use `cargo fmt` as part of the development workflow

## Documentation Guidelines

### Language Requirements
- **All documentation must be in English only**
- This includes README.md files, code comments, API documentation, inline documentation, function descriptions, and variable naming explanations
- Maintain consistent English terminology throughout the codebase
- Use proper technical English terminology
- Keep documentation clear and concise
- Write all commit messages in English

### Comment Guidelines
- All comments must be in English
- Keep comments short, laconic, and specific
- Avoid verbose or unnecessary comments
- Focus on explaining complex logic or non-obvious behavior
- Use clear, technical language
- Write brief, to-the-point explanations
- Avoid redundant comments that just repeat the code
- Prefer single-line comments for simple explanations
- Use multi-line comments only for complex algorithms or important context

### Code Examples Policy
- **Do not create standalone example files or directories**
- **Do not create an `examples/` directory**
- Avoid creating sample implementations that are not used in production
- **Always implement tests instead of examples** to demonstrate functionality
- Use test cases to show how components should be used
- Tests should serve as documentation for code usage
- Ensure tests cover all usage scenarios that would typically be shown in examples
- Document usage patterns directly in code comments or README files
- Use code snippets in documentation rather than separate example files
- Reference test cases in documentation when explaining usage

## roboPLC Integration Guidelines

### Core Requirements
- **Always use the official roboPLC repository** from https://github.com/roboplc/roboplc
- **roboPLC version 0.6** is used in this project
- **tokio is used** for async operations alongside roboPLC workers
- The project combines roboPLC workers with tokio async runtime

### Worker Implementation
When implementing workers with roboPLC:
- Follow the standard roboPLC worker patterns
- Workers can use both synchronous and asynchronous code
- Async operations are handled within workers using tokio
- Main application uses roboPLC controller with tokio runtime

### Best Practices
- Use roboPLC's native error handling mechanisms
- Ensure all roboPLC interactions are thread-safe
- Use roboPLC's built-in configuration mechanisms
- Follow roboPLC's recommended testing patterns
- Always specify and check version compatibility

## Testing Instructions

### Test Commands
TODO
### Test Requirements
- All code changes must include corresponding tests
- Tests should demonstrate functionality and serve as documentation
- Use integration tests for end-to-end functionality
- Use unit tests for component-level usage
- Ensure tests pass before committing changes

## Project Structure

### Main Components
- `src/main.rs` - Main application entry point
- `src/lib.rs` - Library crate with core functionality
- `src/core.rs` - Core application logic and configuration
- `src/workers/` - roboPLC worker implementations
  - `camera.rs` - Camera streaming worker
  - `rvideo.rs` - Video streaming functionality
  - `telegram_bot.rs` - Telegram bot integration
  - `ws_server.rs` - WebSocket server worker
  - `mod.rs` - Workers module definition

### Configuration Files
- `Cross.toml` - Cross-compilation configuration
- `Cargo.toml` - Rust project configuration
- `justfile` - Build and deployment commands
- `robo.toml` - roboPLC configuration

## Deployment and Environment

### Target Platform
- Primary target: `aarch64-unknown-linux-gnu`
- Cross-compilation required for macOS development
- Uses Docker-based build environment

## Security Considerations

- Follow roboPLC security best practices
- Use proper error handling and validation
- Avoid exposing sensitive information in logs or documentation
- Follow secure coding practices for embedded systems

## Key Questions to Ask Before Implementation

- "What is the actual problem I'm trying to solve?"
- "What would the calling code look like?"
- "Can I solve this with existing standard library functions?"
- "Am I over-engineering this?"
- "Can I eliminate this complexity entirely?"
- "Is there existing code that already solves this?"

## Before Implementation Checklist

1. Write the calling code first (outside-in design)
2. Use the simplest types that could work
3. Prefer composition over complex inheritance/traits
4. Question every generic parameter and abstraction layer
5. Verify the solution follows the ELIMINATE → SIMPLIFY → REUSE → CREATE priority order
