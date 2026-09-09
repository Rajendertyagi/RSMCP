<p align="center">
  <img width="96" src="./docs/_media/rust-mcp-filesystem.png" alt="RSMCP Logo" width="300">
</p>

# RSMCP — Rust MCP Server

RSMCP is a comprehensive, production-grade MCP (Model Context Protocol) server built in Rust. It extends the original `rust-mcp-filesystem` with **process management**, **PDF reading**, **Excel read/write**, and **DOCX extraction** — delivering **38 tools** in a single native binary.

This is a fork of [rust-mcp-filesystem](https://github.com/rust-mcp-stack/rust-mcp-filesystem) with added document and process capabilities from [DCMCP](https://github.com/wonderwhy-er/DesktopCommanderMCP).

📝 Full documentation: https://github.com/Rajendertyagi/RSMCP

## Features

- **⚡ High Performance** — Pure Rust, async I/O, compiled to a single native binary (~10 MB)
- **🔒 Root Isolation** — cap-std based confined directory access; symlinks cannot escape allowed roots
- **🔍 Advanced Search** — Glob file search + regex/literal content search via grep crate
- **🔄 MCP Roots Support** — Clients can dynamically update allowed directories (disabled by default)
- **📦 ZIP Archive Support** — Create/extract ZIPs, compress directories with glob patterns
- **🧰 Process Management** — Start, read, write, kill, and list background processes with session tracking
- **📄 PDF Tools** — Read text, list pages with dimensions, extract embedded images
- **📊 Excel Tools** — Read sheets as tabular data, list sheets, write 2D string arrays
- **📝 DOCX Tools** — Extract plain text from Word documents, list XML parts
- **✏️ Fuzzy Editing** — Surgical text replacement with Levenshtein/Jaro-Winkler fuzzy fallback
- **🪶 Lightweight** — No Node.js, Python, or other runtimes required
- **🎛️ Tool Disabling** — Disable specific tools to reduce token usage

## Tool Catalog (38 tools)

### Filesystem (24 tools — inherited from rust-mcp-filesystem)
`read_text_file`, `write_file`, `edit_file`, `list_directory`, `list_directory_with_sizes`, `list_allowed_directories`, `directory_tree`, `file_info`, `apply_patch`, `create_directory`, `remove`, `hash_file`, `move`, `search_files`, `search_files_content`, `read_file_lines`, `head_file`, `tail_file`, `read_media_file`, `read_multiple_media_files`, `zip_files`, `unzip_file`, `zip_directory`, `find_empty_directories`, `calculate_directory_size`, `find_duplicate_files`

### Document Readers (8 tools)
| Tool | Description |
|---|---|
| `read_pdf` | Extract text from PDF (all or specific page) |
| `list_pdf_pages` | Get page count and dimensions |
| `extract_images_from_pdf` | Extract embedded images as base64 |
| `read_excel` | Read sheet data as tabular text |
| `list_excel_sheets` | List sheet names and dimensions |
| `write_excel` | Write 2D string array to Excel |
| `read_docx` | Extract plain text from Word docs |
| `list_docx_parts` | List XML parts in DOCX |

### Process Management (6 tools)
| Tool | Description |
|---|---|
| `process_start` | Spawn a background process, return PID |
| `process_read` | Paginated stdout read (offset/length) |
| `process_write` | Send input to process stdin |
| `process_kill` | Terminate a process by PID |
| `process_list` | List active process sessions |
| `process_ps` | List all system processes |

### Enhanced Editing (2 tools)
| Tool | Description |
|---|---|
| `edit_block` | Exact or fuzzy text replacement |
| `search_and_replace` | Find/replace with regex support |

## 🔧 Installation

### From Source (GitHub Actions builds only — no local install required)
```bash
cargo install rsmcp --git https://github.com/Rajendertyagi/RSMCP
```

### Using cargo-dist (after release)
```bash
curl -LsSf https://github.com/Rajendertyagi/RSMCP/releases/latest/download/rsmcp-installer.sh | sh
```

## Usage

```bash
# Read-only mode (default)
rsmcp /path/to/allowed/dir1 /path/to/dir2

# Read/write mode
rsmcp --allow-write /path/to/allowed/dir

# With MCP Roots support (client provides directories)
rsmcp --enable-roots

# Disable specific tools to save tokens
rsmcp --disable-tools read_pdf,write_excel /path/to/dir
```

## GitHub Actions Integration

Builds are automatically triggered on push/PR to `main`:

```yaml
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@stable
- run: cargo build --release
```

Release artifacts are published to GitHub Releases on tagged releases.

## Security

- All filesystem operations are confined to allowed roots via `cap-std`
- Symlink escapes are rejected at the OS layer
- Process management runs in isolated sessions with bounded output buffers
- Read-only by default; write operations require `--allow-write`

## License

MIT (same as upstream rust-mcp-filesystem)
