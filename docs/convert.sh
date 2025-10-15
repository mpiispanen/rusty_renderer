#!/usr/bin/env bash
# Convert markdown documents to HTML

set -e

DOCS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${DOCS_DIR}/html"

mkdir -p "${OUTPUT_DIR}"

# Simple HTML template function
convert_md_to_html() {
    local md_file="$1"
    local html_file="${OUTPUT_DIR}/$(basename "${md_file%.md}.html")"
    
    echo "Converting ${md_file} to ${html_file}..."
    
    # Create HTML with basic styling
    cat > "${html_file}" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rusty Renderer Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .content {
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        h2 { color: #34495e; margin-top: 30px; border-bottom: 2px solid #ecf0f1; padding-bottom: 8px; }
        h3 { color: #7f8c8d; margin-top: 20px; }
        code {
            background: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }
        pre {
            background: #2c3e50;
            color: #ecf0f1;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
        }
        pre code {
            background: none;
            color: inherit;
            padding: 0;
        }
        ul, ol { margin-left: 20px; }
        li { margin: 8px 0; }
        blockquote {
            border-left: 4px solid #3498db;
            margin: 0;
            padding-left: 20px;
            color: #7f8c8d;
        }
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 20px 0;
        }
        th, td {
            border: 1px solid #ddd;
            padding: 12px;
            text-align: left;
        }
        th {
            background: #3498db;
            color: white;
        }
        tr:nth-child(even) {
            background: #f9f9f9;
        }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .toc {
            background: #ecf0f1;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="content">
EOF

    # Use markdown_py for conversion with fenced code block support
    if command -v markdown_py &> /dev/null; then
        markdown_py -x fenced_code -x codehilite "${md_file}" >> "${html_file}"
    elif command -v cmark &> /dev/null; then
        cmark "${md_file}" >> "${html_file}"
    else
        # Fallback: basic HTML escaping and paragraph detection
        sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' \
            -e 's/^$/\<\/p\>\<p\>/' \
            -e '1s/^/<p>/' -e '$s/$/<\/p>/' "${md_file}" >> "${html_file}"
    fi
    
    cat >> "${html_file}" << 'EOF'
    </div>
</body>
</html>
EOF
    
    echo "Created ${html_file}"
}

# Convert all markdown files
for md_file in "${DOCS_DIR}"/*.md; do
    if [ -f "${md_file}" ]; then
        convert_md_to_html "${md_file}"
    fi
done

echo "Conversion complete. HTML files are in ${OUTPUT_DIR}/"
