# TypeScript Type Generation

This project uses [ts-rs](https://github.com/Aleph-Alpha/ts-rs) to automatically generate TypeScript type definitions from Rust structs, ensuring type safety between the backend and frontend.

## Benefits

- **Type Safety**: Eliminates manual type definitions and prevents mismatches
- **Auto-sync**: Types automatically match Rust structs
- **CamelCase Conversion**: Rust snake_case is automatically converted to TypeScript camelCase
- **Documentation**: Rust doc comments are preserved in generated types

## Generated Types Location

All generated types are in `src/lib/bindings/`:

```
src/lib/bindings/
├── ArchiveInfo.ts
├── CompletionEvent.ts
├── ExtractOptionsDTO.ts
├── ExtractStats.ts
├── JobStatus.ts
├── PasswordRequiredEvent.ts
└── ProgressEvent.ts
```

## Regenerating Types

After modifying any Rust struct that's exported to TypeScript:

```bash
npm run generate-types
```

This runs `cargo run --bin export_types` which triggers ts-rs to regenerate all types.

## Adding New Types

To export a new Rust struct to TypeScript:

1. Add the `ts-rs` dependency (already added)
2. Derive `TS` on your struct:

```rust
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct MyNewType {
    pub my_field: String,
    #[ts(optional)]
    pub optional_field: Option<u64>,
}
```

3. Add the export call to `src-tauri/src/bin/export_types.rs`:

```rust
MyNewType::export().expect("Failed to export MyNewType");
```

4. Run `npm run generate-types`

## Type Annotations

### Optional Fields

```rust
#[ts(optional)]
pub my_field: Option<String>,
```

### Custom TypeScript Types

```rust
#[ts(type = "number")]  // u64 -> number instead of bigint
pub bytes: u64,
```

### Rename Fields

```rust
#[ts(rename = "customName")]
pub field_name: String,
```

## Common Issues

### bigint vs number

By default, Rust `u64` maps to TypeScript `bigint`. For most cases, use `number`:

```rust
#[ts(type = "number")]
pub count: u64,
```

### Case Sensitivity

Always use `#[serde(rename_all = "camelCase")]` on structs to match TypeScript conventions.

### Path Issues

Export paths are relative to the Rust file. From `src-tauri/src/commands.rs`:
- Use `../../src/lib/bindings/` to reach the frontend bindings directory

## Workflow

1. **Modify Rust struct** with `#[derive(TS)]`
2. **Run** `npm run generate-types`
3. **Import** types in TypeScript from `@/lib/api`
4. **Commit** both Rust and generated TypeScript files

## Development

The project is configured with `default-run = "unarchiver"` in `Cargo.toml`, so:
- `bun tauri dev` runs the main application
- `npm run generate-types` runs the type export utility
- Both binaries coexist without conflicts

## Example

**Rust** (`src-tauri/src/commands.rs`):
```rust
#[derive(Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: String,
    #[ts(type = "number")]
    pub bytes_written: u64,
}
```

**Generated TypeScript** (`src/lib/bindings/ProgressEvent.ts`):
```typescript
export type ProgressEvent = { 
  jobId: string, 
  bytesWritten: number,
};
```

**Usage** (`src/App.tsx`):
```typescript
import type { ProgressEvent } from '@/lib/api';

const handleProgress = (event: ProgressEvent) => {
  console.log(event.jobId, event.bytesWritten);
};
```

## Resources

- [ts-rs Documentation](https://github.com/Aleph-Alpha/ts-rs)
- [ts-rs Attributes](https://github.com/Aleph-Alpha/ts-rs#attributes)
