To split each module into separate workspaces in Rust, you'll need to follow these steps:

### 1. **Create a Workspace Structure**

First, restructure your project into a Cargo workspace. A workspace allows you to manage multiple related packages (crates) in a single repository. Here's how to do it:

#### **Step 1: Create a `Cargo.toml` at the Root**

This file defines the workspace and lists the member crates.

```toml Cargo.toml
[workspace]
members = [
    "app",
    "docsplitter",
    "embedder",
    "lancevectordb",
    "chat",
    "tests",
]
resolver = "2"  # Ensures consistent dependency resolution
```

#### **Step 2: Move Each Module into Its Own Crate**

Each module (`app`, `docsplitter`, `embedder`, etc.) should become a separate crate with its own `Cargo.toml`.

Example for `docsplitter`:

```toml docsplitter/Cargo.toml
[package]
name = "docsplitter"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

#### **Step 3: Update Dependencies**

Ensure each crate's `Cargo.toml` lists its dependencies correctly. If crates depend on each other, specify them like this:

```toml app/Cargo.toml
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
docsplitter = { path = "../docsplitter" }
```

#### **Step 4: Adjust Module Paths**

Update `mod` declarations in `lib.rs` (or equivalent) to reflect the new crate structure. For example, if `docsplitter` is now a separate crate, you might import it as:

```rust src/lib.rs
// Instead of `pub mod docsplitter;`
use docsplitter;
```

### 2. **Update Imports in Code**

Ensure all imports in your modules point to the correct crate paths. For example:

```rust src/docsplitter/code_loader.rs
// If `FileChunk` is defined in `docsplitter`, no changes are needed.
// If it was in another module, update the import:
use crate::types::FileChunk;  // Might become `use shared_types::FileChunk;`
```

### 3. **Handle Shared Code**

If multiple crates share types or utilities, create a `shared` or `core` crate:

```toml shared/Cargo.toml
[package]
name = "shared"
version = "0.1.0"
edition = "2021"
```

Then, other crates can depend on it:

```toml docsplitter/Cargo.toml
[dependencies]
shared = { path = "../shared" }
```

### 4. **Update Build and Test Workflows**

- Run tests for all crates: `cargo test --workspace`
- Build all crates: `cargo build --workspace`

### 5. **Verify the Structure**

Ensure all crates compile and tests pass. Fix any dependency issues that arise.

### Example Final Structure

```
your_project/
├── Cargo.toml          # Workspace root
├── app/
│   ├── Cargo.toml
│   └── src/
│       └── ...
├── docsplitter/
│   ├── Cargo.toml
│   └── src/
│       └── ...
├── embedder/
│   ├── Cargo.toml
│   └── src/
│       └── ...
└── shared/             # Optional for shared code
    ├── Cargo.toml
    └── src/
        └── ...
```

### Key Benefits

- **Isolation**: Each module is a separate crate, reducing compile times for unrelated changes.
- **Reusability**: Crates can be published independently.
- **Clear Dependencies**: Explicit dependencies between crates.

Would you like help with any specific part of this migration?
