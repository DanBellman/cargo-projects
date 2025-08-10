## Usage

### Basic Commands

List all tracked projects:
```bash
cargo projects list
```

List projects from a specific watcher:
```bash
cargo projects list --watcher-name my-watcher
```

Scan a directory for Rust projects:
```bash
cargo projects scan /path/to/directory
```

Watch a directory for new projects:
```bash
cargo projects watch --project-path /path/to/watch --name my-watcher
```

### Project Management

Clean a specific project's target directory:
```bash
cargo projects clean <project-id>
```

Update project information:
```bash
cargo projects update
```

Refresh timing data:
```bash
cargo projects refresh
```

### Watcher Management

List all watchers:
```bash
cargo projects watchers
```

Clean inactive watchers:
```bash
cargo projects clean-watchers
```

## Project Information

The tool tracks the following information for each project:

- **Name & Path**: Project name and filesystem location
- **Size**: Total project size and target directory size
- **Dependencies**: Number of dependencies
- **Build Time**: Estimated build time (planned feature)
- **Last Modified**: When the project was last changed
- **Project Type**: Package, workspace, etc.

## Configuration

Configuration files are stored in your system's config directory:
- `~/.config/cargo-projects/registry.ron` - Project registry
- `~/.config/cargo-projects/watchers.ron` - Watcher configuration

## Examples

### Typical Workflow

1. Set up a watcher for your development directory:
   ```bash
   cargo projects watch --project-path ~/dev --name dev-projects
   ```

2. Let it discover projects, then list them:
   ```bash
   cargo projects list
   ```

3. Clean up large projects you're not using:
   ```bash
   cargo projects clean 42  # Clean project with ID 42
   ```

### Managing Multiple Workspaces

If you work with different types of projects, set up multiple watchers:

```bash
cargo projects watch --project-path ~/work --name work-projects
cargo projects watch --project-path ~/personal --name personal-projects
cargo projects watch --project-path ~/experiments --name experiments
```

Then list projects by category:
```bash
cargo projects list --watcher-name work-projects
cargo projects list --watcher-name personal-projects
```
