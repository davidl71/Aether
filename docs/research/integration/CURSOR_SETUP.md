# Cursor IDE Configuration

This project includes comprehensive Cursor IDE configuration to enhance your development experience.

## Configuration Files

### `.cursorrules`

Main AI assistant rules file that guides Cursor's AI when helping with this codebase. It includes:

- Code style guidelines (C++20, 2-space indentation, Allman braces)
- Build system conventions
- Testing practices
- Security guidelines
- Static analysis tools and annotations
- Project structure information

**Note**: For examples of `.cursorrules` files from other projects, see [awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules) - a
curated collection of Cursor rules for various technologies and frameworks.

### `.vscode/settings.json`

Workspace settings for Cursor/VS Code:

- **C++ Configuration**: IntelliSense, include paths, compiler settings
- **Editor Settings**: 2-space indentation, 100-character ruler, format on save
- **CMake Integration**: Auto-configuration, build directory settings
- **File Exclusions**: Hides build artifacts from file explorer
- **Python/Rust/TypeScript**: Language-specific settings

### `.vscode/tasks.json`

Pre-configured build tasks:

- **CMake: Configure (Debug)** - Configure with debug preset
- **CMake: Build** - Build the project (default build task)
- **CMake: Build (Release)** - Build release version
- **CMake: Clean** - Clean build artifacts
- **Run Tests** - Execute test suite
- **Setup Worktree** - Run worktree setup script
- **Build Universal** - Run universal build script
- **Run Linters** - Execute linting checks
- **Build Intel Decimal Library** - Build dependency
- **Build TWS API Library** - Build dependency

### `.vscode/launch.json`

Debug configurations:

- **Debug ib_box_spread** - Debug main executable with dry-run
- **Debug ib_box_spread (with config)** - Debug with config file
- **Run Tests** - Debug test suite
- **Attach to Process** - Attach debugger to running process

### `.cursor/BUGBOT.md`

Project-specific rules for Cursor Bugbot (AI-powered PR reviews):

- Security requirements (trading software safety)
- Code style guidelines
- Build system requirements
- Testing requirements
- Common issues to flag

**See**: [CURSOR_BUGBOT_INTEGRATION.md](CURSOR_BUGBOT_INTEGRATION.md) for complete setup guide.

### `.vscode/extensions.json`

Recommended extensions:

- **C++**: C/C++ extension, CMake Tools
- **Python**: Python, Pylance, Black formatter
- **Rust**: rust-analyzer (for agents/backend)
- **TypeScript**: ESLint (for web/)
- **Swift**: Swift language support (for ios/ and desktop/)
- **General**: EditorConfig, GitLens, Markdown tools, ShellCheck

Unwanted extensions (will be blocked):

- Go extension (not used in this project)
- Prettier (optional - ESLint handles formatting)
- Docker/Kubernetes extensions (not used)

### `.editorconfig`

Editor-agnostic configuration for consistent formatting across editors.

## Quick Start

1. **Open the project in Cursor**

   ```bash
   cursor /path/to/ib_box_spread_full_universal
   ```

2. **Install recommended extensions**
   - Cursor will prompt you to install recommended extensions
   - Or use: `Cmd+Shift+P` → "Extensions: Show Recommended Extensions"

3. **Configure CMake** (if needed)
   - Cursor should auto-detect CMakePresets.json
   - Or manually configure: `Cmd+Shift+P` → "CMake: Configure"

4. **Build the project**
   - Use `Cmd+Shift+B` to run default build task
   - Or `Cmd+Shift+P` → "Tasks: Run Task" → "CMake: Build"

5. **Debug**
   - Set breakpoints in your code
   - Press `F5` to start debugging
   - Or use `Cmd+Shift+P` → "Debug: Start Debugging"

## Key Features

### IntelliSense

- Full C++20 IntelliSense with clang
- Auto-completion for TWS API headers
- Go-to-definition support
- Symbol navigation

### Build Integration

- One-click build with `Cmd+Shift+B`
- Build errors shown in Problems panel
- Click errors to jump to source

### Debugging

- Full LLDB integration
- Breakpoints, watch variables, call stack
- Step through code with F10/F11
- Debug console for expressions

### Code Formatting

- Format on save enabled
- Consistent 2-space indentation
- 100-character line length guide
- Auto-trim trailing whitespace

### File Navigation

- Build artifacts hidden from explorer
- Quick file search with `Cmd+P`
- Symbol search with `Cmd+Shift+O`
- Workspace symbol search with `Cmd+T`

## Customization

### User-Specific Settings

Create `.vscode/settings.json.user` (gitignored) for personal preferences:

```json
{
  "editor.fontSize": 14,
  "editor.fontFamily": "Fira Code"
}
```

### Custom Tasks

Add project-specific tasks to `.vscode/tasks.json`:

```json
{
  "label": "My Custom Task",
  "type": "shell",
  "command": "echo 'Hello'",
  "group": "build"
}
```

### Custom Launch Configurations

Add debug configurations to `.vscode/launch.json`:

```json
{
  "name": "My Custom Debug",
  "type": "cppdbg",
  "request": "launch",
  "program": "${workspaceFolder}/build/bin/my_program"
}
```

## Troubleshooting

### IntelliSense Not Working

1. Check that `compile_commands.json` exists in build directory
2. Run CMake configure: `Cmd+Shift+P` → "CMake: Configure"
3. Reload window: `Cmd+Shift+P` → "Developer: Reload Window"

### Build Fails

1. Check CMake preset is configured: `Cmd+Shift+P` → "CMake: Configure"
2. Verify dependencies are built (Intel Decimal, TWS API)
3. Check build output: `View` → `Output` → Select "CMake" or "Tasks"

### Debugger Not Attaching

1. Ensure binary is built with debug symbols (`CMAKE_BUILD_TYPE=Debug`)
2. Check that program path in launch.json is correct
3. Verify LLDB is installed: `which lldb`

### Extensions Not Installing

1. Check internet connection
2. Try installing manually from Extensions view
3. Check Cursor/VS Code version compatibility

## Tips

### Keyboard Shortcuts

- `Cmd+Shift+B` - Build
- `F5` - Start debugging
- `F9` - Toggle breakpoint
- `F10` - Step over
- `F11` - Step into
- `Shift+F11` - Step out
- `Cmd+P` - Quick file open
- `Cmd+Shift+O` - Go to symbol in file
- `Cmd+T` - Go to symbol in workspace
- `Cmd+Shift+F` - Search in files

### Productivity Tips

1. Use `Cmd+Shift+P` for command palette - faster than menus
2. Enable "Format on Save" for consistent code style
3. Use Problems panel (`Cmd+Shift+M`) to see all errors/warnings
4. Use multi-cursor editing (`Cmd+Option+Up/Down`) for bulk edits
5. Use breadcrumbs (`View` → `Show Breadcrumbs`) for navigation

## Codebase Indexing

Cursor automatically indexes your codebase to enable semantic search and improve AI suggestions. Understanding how indexing works can help you
optimize your code for better AI assistance.

### How It Works

Cursor uses a 7-step process to index your codebase:

1. **File Sync**: Workspace files are securely synchronized with Cursor's servers
2. **Chunking**: Files are broken into meaningful chunks (functions, classes, logical blocks)
3. **Embeddings**: Each chunk is converted to vector representations using AI models
4. **Storage**: Embeddings stored in a vector database optimized for similarity search
5. **Query Processing**: Your search queries are converted to vectors using the same models
6. **Similarity Search**: System finds most similar code chunks by comparing vectors
7. **Results**: Relevant code snippets returned ranked by semantic similarity

### Benefits

- **Semantic Search**: Find code by meaning, not just exact string matches
- **Faster Agent Searches**: Pre-computed embeddings make searches faster
- **Better Accuracy**: Custom-trained models retrieve more relevant results
- **Conceptual Matching**: Find code by what it does, not just what it's named

### Privacy & Security

- File paths are encrypted before being sent to servers
- Code content is never stored in plaintext
- Code is only held in memory during indexing, then discarded
- No permanent storage of source code

### Configuration

Cursor indexes all files except those in ignore files (`.gitignore`, `.cursorignore`).

**To configure indexing:**

1. Click `Show Settings` in Cursor
2. Enable automatic indexing for new repositories
3. Configure which files to ignore

**To view indexed files:**

- `Cursor Settings` > `Indexing & Docs` > `View included files`

**Tip**: Ignoring large content files (see [CURSOR_IGNORE_SETUP.md](CURSOR_IGNORE_SETUP.md)) improves answer accuracy and indexing performance.

### Optimizing for Indexing

To get the best results from codebase indexing:

1. **Use Descriptive Names**: Function and class names that clearly describe purpose help semantic search
2. **Add Documentation Comments**: Comments explaining *why* (not just *what*) improve semantic understanding
3. **Structure Code Logically**: Well-organized code with clear boundaries (functions, classes) chunks better
4. **Use Static Analysis Annotations**: Attributes like `[[nodiscard]]` and `__attribute__((nonnull))` provide semantic hints (see
[STATIC_ANALYSIS_ANNOTATIONS.md](../../research/analysis/STATIC_ANALYSIS_ANNOTATIONS.md))
5. **Follow Naming Conventions**: Consistent naming patterns help the AI understand code relationships

### Automatic Updates

- Indexing begins automatically when you open a workspace
- Semantic search becomes available at 80% completion
- Index syncs automatically every 5 minutes
- Only changed files are updated (old embeddings removed, new ones created)
- Files processed in batches for optimal performance

### Multi-Root Workspaces

Cursor supports multi-root workspaces:

- All codebases get indexed automatically
- Each codebase's context is available to AI
- `.cursor/rules` work in all folders
- Some features (like worktrees) are disabled for multi-root workspaces

**Note**: Cursor Cloud Agents do not support multi-root workspaces.

For more details, see [Cursor's Codebase Indexing Documentation](https://cursor.com/docs/context/codebase-indexing).

## Community Resources & Best Practices

### Learning Resources

- **[Write C++ and Python Code Faster And Better With Cursor](https://supercomputingblog.com/coding-with-ai/write-c-and-python-code-faster-and-better-with-cursor/)** - Practical guide for C++ developers using Cursor,
  including examples with Z3 solver and build system integration
- **[Why I'm Back Using Cursor (And Why Their CLI Changes Everything)](https://www.ksred.com/why-im-back-using-cursor-and-why-their-cli-changes-everything/)** - Comparison of Cursor CLI vs Claude Code,
  performance considerations, and workflow optimization
- **[Cursor Community Forum](https://forum.cursor.com/)** - Community discussions, tips, and best practices from Cursor users

### Alternative AI Coding Tools

While this project is configured for Cursor, other AI coding assistants are available:

- **Claude Code** - Anthropic's terminal-based AI coding assistant (see [Builder.io's guide](https://www.builder.io/blog/claude-code))
- **Windsurf** - AI-powered editor with Cascade agent (see [windsurf.com](https://windsurf.com/))
- **GitHub Copilot** - Microsoft's AI pair programmer

**Note**: The static analysis annotations and code quality practices documented in this project work with any AI coding assistant that understands
code semantics.

### Tips from the Community

Based on community feedback and best practices:

1. **Use Git Frequently**: Commit working code often when using AI assistants - it's easier to revert if AI suggestions go wrong
2. **Add Context in Comments**: Use "Rules for AI" or file-level comments to provide context about features, constraints, and considerations
3. **Review AI Suggestions**: Don't blindly accept all suggestions - review and understand what the AI is doing
4. **Clear Chat History**: Use `/clear` or start fresh conversations when switching tasks to avoid token waste
5. **Combine Tools**: Use Cursor for quick completions and tab completions, but consider other tools for complex architectural decisions

## See Also

- [CURSOR_AI_TUTORIAL.md](../../CURSOR_AI_TUTORIAL.md) - Cursor AI tutorial and best practices
- [CURSOR_RECOMMENDATIONS.md](../../CURSOR_RECOMMENDATIONS.md) - Cursor optimization recommendations
- [CURSOR_DOCS_USAGE.md](CURSOR_DOCS_USAGE.md) - Using @docs in Cursor
- [CURSOR_IGNORE_SETUP.md](CURSOR_IGNORE_SETUP.md) - File exclusion configuration
- [STATIC_ANALYSIS_ANNOTATIONS.md](../../research/analysis/STATIC_ANALYSIS_ANNOTATIONS.md) - Static analysis annotations that help indexing
- [README.md](../../../README.md) - Main project documentation
- [WORKTREE_SETUP.md](WORKTREE_SETUP.md) - Worktree setup guide
- [QUICK_START.md](QUICK_START.md) - Quick start guide
