# PDFSnap CLI - Interactive PDF Generator

The CLI tool allows you to test PDF generation interactively without needing a web form or API client.

## Building the CLI

```bash
cargo build --bin pdfsnap-cli
```

Or run it directly:

```bash
cargo run --bin pdfsnap-cli
```

## Usage

1. **Start the CLI:**
   ```bash
   cargo run --bin pdfsnap-cli
   ```

2. **Provide PDF Template:**
   - Enter a local file path (e.g., `./template.pdf`)
   - Or enter a URL (e.g., `https://example.com/template.pdf`)

3. **Add Variables:**
   - Specify how many variables you want to add
   - For each variable, you'll be prompted for:
     - **Type**: text, signature, or image
     - **Page number**: Which page to place it on (1-indexed)
     - **Position**: X and Y coordinates
     - **Size**: Width and height
     - **Field name**: An identifier for the field
     - **Value**: The actual content
       - For text: the text string
       - For signature: the signature text (rendered in cursive)
       - For image: the image URL

4. **Text Variables Additional Options:**
   - Font size (optional, uses default if not specified)
   - Text alignment: left, center, or right
   - Color: Hex color code (e.g., `#000000` for black)

5. **Verification Hash:**
   - Choose whether to include a verification hash in the PDF header

6. **Output:**
   - Specify the output file path (default: `output.pdf`)

## Example Session

```
=== PDFSnap CLI - Interactive PDF Generator ===

Enter PDF template path or URL: ./template.pdf
Loading template from file...

Template loaded successfully!
Most used font size: 12.00
Number of pages: 1

How many variables do you want to add? [1]: 2

--- Variable 1 ---
Variable type
> text
  signature
  image
Page number (1-indexed) [1]: 1
X position [100.0]: 50
Y position [100.0]: 200
Width [200.0]: 300
Height [50.0]: 30
Field name (identifier) [field_1]: name
Text value: John Doe
Font size (press Enter for default): 14
Text alignment
> left
  center
  right
Color (hex format, e.g., #000000 for black, or press Enter for default) [#000000]: 

--- Variable 2 ---
Variable type
  text
> signature
  image
Page number (1-indexed) [1]: 1
X position [100.0]: 50
Y position [100.0]: 250
Width [200.0]: 200
Height [50.0]: 30
Field name (identifier) [field_2]: signature
Signature text: John Doe
Font size (press Enter for default): 

Processing variables...
Include verification hash in header?
> No
  Yes
Output file path [output.pdf]: generated.pdf

✅ PDF generated successfully!
📄 Saved to: generated.pdf
```

## Tips

- **Coordinates**: PDF coordinates start from the bottom-left corner (0,0)
- **Page Numbers**: Use 1-indexed page numbers (first page is 1)
- **Font Sizes**: If you don't specify a font size, it will use the most common font size from the template
- **Colors**: Use hex format like `#FF0000` for red, `#00FF00` for green, etc.
- **Images**: Must be accessible via HTTP/HTTPS URL

## Troubleshooting

- If you get "Page not found", make sure your page number is within the range of pages in your PDF
- For images, ensure the URL is publicly accessible
- Make sure you have write permissions for the output directory

