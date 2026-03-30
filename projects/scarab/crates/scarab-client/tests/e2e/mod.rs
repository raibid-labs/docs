//! End-to-End Integration Tests for Scarab Terminal Emulator
//!
//! This module provides comprehensive E2E testing for the full Scarab stack:
//! - Daemon process management
//! - Client process lifecycle
//! - Shared memory IPC
//! - Unix socket communication
//! - PTY interaction
//! - Terminal state management
//!
//! ## Test Scenarios
//!
//! 1. **Basic Workflow** - Simple echo commands and basic terminal operations
//! 2. **Vim Editing** - Full vim editor interaction (marked `#[ignore]`)
//! 3. **Color Rendering** - ANSI color parsing and display
//! 4. **Scrollback** - Large output handling and buffer management
//! 5. **Session Persistence** - Client disconnect/reconnect with state preservation
//! 6. **Input Forwarding** - Keyboard input and control sequences
//! 7. **Resize Handling** - Terminal dimension changes
//! 8. **Stress Testing** - Long-running stability tests (marked `#[ignore]`)
//!
//! ## Running Tests
//!
//! Run all E2E tests (excluding ignored tests):
//! ```bash
//! cargo test --test e2e
//! ```
//!
//! Run a specific test scenario:
//! ```bash
//! cargo test --test e2e basic_echo
//! ```
//!
//! Run ignored tests (vim, stress tests):
//! ```bash
//! cargo test --test e2e -- --ignored
//! ```
//!
//! Run a specific ignored test:
//! ```bash
//! cargo test --test e2e stress_1_hour -- --ignored
//! ```
//!
//! ## Test Environment
//!
//! These tests require:
//! - Linux or macOS (Unix socket support)
//! - Bash shell available
//! - Write permissions for /tmp
//! - Compiled daemon and client binaries
//!
//! Optional (for some tests):
//! - vim (for vim_editing tests)
//! - htop/top (for application interaction tests)
//!
//! ## Architecture
//!
//! The test harness (`harness.rs`) provides:
//! - Process spawning and lifecycle management
//! - Shared memory access
//! - IPC communication helpers
//! - Output verification utilities
//! - Automatic cleanup on drop
//!
//! Each test scenario is in its own module for clarity and organization.

pub mod harness;

// Test scenario modules
mod basic_workflow;
mod color_rendering;
mod input_forward;
mod resize_handling;
mod scrollback;
mod session_persist;
mod stress_test;
mod vim_editing;
