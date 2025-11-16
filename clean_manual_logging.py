#!/usr/bin/env python3
"""
Remove manual file logging blocks from Rust source files.
These blocks look like:
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("rusty_renderer_debug.log")
    {
        let _ = writeln!(f, "...");
        let _ = f.flush();
    }
"""

import re
import sys

def clean_manual_logging(content):
    """Remove manual file logging blocks"""
    # Pattern to match manual file logging blocks
    # Matches from "if let Ok(mut f) = std::fs::OpenOptions" to the closing "}"
    pattern = r'\s*if let Ok\(mut f\) = std::fs::OpenOptions::new\(\)[\s\S]*?\.open\("rusty_renderer_debug\.log"\)[\s\S]*?\{[\s\S]*?\n\s*\}\n'
    
    cleaned = re.sub(pattern, '', content)
    return cleaned

def main():
    if len(sys.argv) != 2:
        print("Usage: python clean_manual_logging.py <file>")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    original_lines = len(content.split('\n'))
    cleaned = clean_manual_logging(content)
    cleaned_lines = len(cleaned.split('\n'))
    
    with open(filepath, 'w') as f:
        f.write(cleaned)
    
    print(f"Cleaned {filepath}")
    print(f"  Removed {original_lines - cleaned_lines} lines")

if __name__ == '__main__':
    main()
