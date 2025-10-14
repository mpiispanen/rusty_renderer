# Documentation

This directory contains all project documentation.

## Files

- `DESIGN.md` - Main design document describing architecture, roadmap, and technical decisions
- `convert.sh` - Script to convert markdown documents to HTML

## Viewing Documentation

### Markdown
All documentation is written in Markdown and can be viewed directly in any text editor or on GitHub.

### HTML
To generate HTML versions of the documentation:

```bash
./convert.sh
```

This will create HTML files in the `html/` directory with proper styling for easier reading in a browser.

## Contributing to Documentation

When adding or updating documentation:
1. Create/edit markdown files in this directory
2. Run `./convert.sh` to generate HTML versions (optional, for local viewing)
3. Commit both markdown files (HTML files are gitignored)
