# Private API Feature Gate Status

## Overview

This document tracks the progress of adding `#[cfg(feature = "private-api")]` guards to fix compilation errors when building with `--no-default-features`.

## Problem

When building with `cargo build --no-default-features`, the `private-api` feature is disabled. In this mode:
- `AppState.private_api` is `Option<Arc<()>>` instead of `Option<Arc<PrivateApiService>>`
- Calling methods like `.send_action()`, `.is_connected()`, `.is_messages_ready()` on `Arc<()>` fails compilation
- Route handlers that use Private API methods need to be feature-gated

## Solution Approach

1. Add `#[cfg(feature = "private-api")]` guards to functions that require Private API
2. Add `#[cfg(not(feature = "private-api"))]` stub implementations that return errors
3. Feature-gate conditional code blocks that call Private API methods

## Completed Work

### ✅ crates/imessage-http/src/state.rs
- Already had proper feature guards for `require_private_api()`, `require_facetime_private_api()`, and `require_findmy_private_api()`
- These return `Arc<()>` when feature is disabled, causing compilation errors downstream

### ✅ crates/imessage-http/src/routes/server.rs
- Fixed lines 45-62: Added feature guards for `.is_connected()`, `.is_messages_ready()`, `.is_facetime_ready()`, `.is_findmy_ready()` calls
- Fixed line 313: Added feature guards for `.is_messages_ready()` call
- Returns `false` for all Private API status checks when feature is disabled

### ✅ crates/imessage-http/src/routes/icloud.rs
- Added `#[cfg(feature = "private-api")]` guards to functions:
  - `get_account_info` + stub
  - `change_alias` + stub
  - `get_contact_card` + stub
  - `refresh_findmy_devices` + stub
  - `refresh_findmy_friends` + stub
  - `do_refresh_findmy_friends` (private helper) + stub
  - `fetch_findmy_key` (private helper) + stub

### ✅ crates/imessage-http/src/routes/message.rs
- Fixed auto-stop typing indicator blocks (4 occurrences) - wrapped with feature guards
- Added `#[cfg(feature = "private-api")]` guards to functions:
  - `edit_message` + stub
  - `unsend_message` + stub
  - `notify_message` + stub

### ✅ crates/imessage-http/src/routes/chat.rs
- Added `#[cfg(feature = "private-api")]` guards to functions (via Python script):
  - `create` (conditional block for `method == "private-api"`) + stub for that path
  - `update` + stub
  - `delete_chat`
  - `mark_read`
  - `mark_unread`
  - `leave`
  - `start_typing`
  - `stop_typing`
  - `add_participant`
  - `remove_participant`
  - `remove_participant_delete`
  - `set_icon`
  - `remove_icon`
  - `share_contact`
  - `delete_message`
  - `share_contact_status`

### ✅ crates/imessage-http/src/routes/attachment.rs
- Added `#[cfg(feature = "private-api")]` guard to:
  - `force_download`

### ✅ crates/imessage-http/src/routes/handle.rs
- Added `#[cfg(feature = "private-api")]` guards to:
  - `get_focus_status`
  - `get_imessage_availability`
  - `post_imessage_availability`
  - `get_facetime_availability`
  - `post_facetime_availability`

### ✅ crates/imessage-http/src/routes/facetime.rs
- Added `#[cfg(feature = "private-api")]` guards to:
  - `create_session` + stub
  - `answer_call`
  - `leave_call`

## Remaining Work

###  ❌ Missing Stub Implementations

Most functions that were marked with `#[cfg(feature = "private-api")]` still need corresponding `#[cfg(not(feature = "private-api"))]` stub implementations. Without these stubs:
- The route registrations in `server.rs` will fail to compile because the functions don't exist
- Users calling these endpoints when Private API is disabled will get confusing errors

**Pattern for adding stubs:**

```rust
#[cfg(feature = "private-api")]
pub async fn some_function(
    State(state): State<AppState>,
    Path(param): Path<String>,
) -> Result<Json<Value>, AppError> {
    // ... implementation
}

#[cfg(not(feature = "private-api"))]
pub async fn some_function(
    State(_state): State<AppState>,
    Path(_param): Path<String>,
) -> Result<Json<Value>, AppError> {
    Err(AppError::imessage_error("Private API is not compiled (feature disabled)"))
}
```

### Files Needing Stubs

1. **chat.rs** - 13 functions need stubs:
   - `delete_chat`, `mark_read`, `mark_unread`, `leave`
   - `start_typing`, `stop_typing`
   - `add_participant`, `remove_participant`, `remove_participant_delete`
   - `set_icon`, `remove_icon`
   - `share_contact`, `share_contact_status`, `delete_message`

2. **attachment.rs** - 1 function needs stub:
   - `force_download`

3. **handle.rs** - 5 functions need stubs:
   - `get_focus_status`
   - `get_imessage_availability`, `post_imessage_availability`
   - `get_facetime_availability`, `post_facetime_availability`

4. **facetime.rs** - 2 functions need stubs:
   - `answer_call`
   - `leave_call`

## Testing

Once all stubs are added:

```bash
# Test that compilation succeeds without private-api feature
cargo build --no-default-features

# Test that compilation succeeds with private-api feature (default)
cargo build

# Run tests
cargo test
```

## Notes

- The `#[cfg(feature = "private-api")]` guards ensure type safety - code that calls Private API methods only compiles when the feature is enabled
- Stub implementations provide better UX by returning clear error messages instead of 404s when routes are called without Private API
- Alternative approach: Feature-gate the route registrations themselves in `server.rs`, but this is messier and provides worse UX
