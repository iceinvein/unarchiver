# Generated TypeScript Bindings

This directory contains TypeScript type definitions automatically generated from Rust structs using [ts-rs](https://github.com/Aleph-Alpha/ts-rs).

## ⚠️ Do Not Edit

**These files are auto-generated and should not be edited manually.** Any changes will be overwritten the next time types are generated.

## Regenerating Types

To regenerate the TypeScript types after modifying Rust structs:

```bash
npm run generate-types
```

Or directly:

```bash
cd src-tauri && cargo run --bin export_types
```

## How It Works

1. Rust structs in `src-tauri/src/commands.rs` and `crates/extractor/src/types.rs` are annotated with `#[derive(TS)]`
2. The `export_types` binary triggers ts-rs to generate TypeScript definitions
3. Types are exported to this directory with proper camelCase conversion

## Available Types

- `ArchiveInfo` - Metadata about an archive file
- `ExtractStats` - Statistics from a completed extraction
- `ExtractOptionsDTO` - Options for extraction operations
- `ProgressEvent` - Real-time progress updates during extraction
- `CompletionEvent` - Extraction completion notification
- `PasswordRequiredEvent` - Password prompt event
- `JobStatus` - Extraction job status enum

## Usage

Import types from the main API module:

```typescript
import type { ArchiveInfo, ProgressEvent, CompletionEvent } from '@/lib/api';
```

The types are re-exported from `src/lib/api.ts` for convenience.
