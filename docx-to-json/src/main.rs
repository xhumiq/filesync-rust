use docx_rs::*;
use std::fs::File;
use std::io::Write;

mod model;
use model::Record;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -----------------------------------------------------------------
    // 1. Open the .docx file (change the path if needed)
    // -----------------------------------------------------------------
    let path = std::path::Path::new("zsv251110-List.docx");
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buf)?;
    let docx = docx_rs::read_docx(&buf)?;
    //let docx = Docx::read(file)?;

    // -----------------------------------------------------------------
    // 2. Find the first table (your list is the only table)
    // -----------------------------------------------------------------
    let table = docx
        .document
        .children
        .iter()
        .find_map(|child| match child {
            DocumentChild::Table(t) => Some(t),
            _ => None,
        })
        .ok_or("No table found in the document")?;

    // -----------------------------------------------------------------
    // 3. Skip the header row (順序 | 錄影內容 | 檔案數量)
    // -----------------------------------------------------------------
    let rows: Vec<_> = table.rows.iter().filter_map(|child| match child {
        docx_rs::TableChild::TableRow(r) => Some(r),
        _ => None,
    }).collect();
    let data_rows = &rows[1..]; // everything after the header

    // -----------------------------------------------------------------
    // 4. Parse each row
    // -----------------------------------------------------------------
    let mut records = Vec::new();

    for row in data_rows {
        let cell_strings: Vec<String> = row
            .cells
            .iter()
            .filter_map(|child| match child {
                docx_rs::TableRowChild::TableCell(c) => Some(extract_text_from_cell(c)),
                _ => None,
            })
            .collect();
        let cells: Vec<&str> = cell_strings.iter().map(|s| s.trim()).collect();

        // Expected layout: [seq, name+desc, file_count]
        if cells.len() != 3 {
            eprintln!("Skipping malformed row: {:?}", cells);
            continue;
        }

        let seq: u32 = cells[0].parse()?;
        let file_count: u32 = cells[2].parse()?;

        // The second column contains BOTH the code name and the Chinese description,
        // separated by the transition from ASCII to Chinese characters.
        let full = cells[1];

        let first_non_ascii = full.chars().position(|c| !c.is_ascii());
        let (name, description) = if let Some(pos) = first_non_ascii {
            let name_part = full[..pos].trim();
            let desc_part = full[pos..].trim();
            (name_part.to_owned(), desc_part.to_owned())
        } else {
            (full.trim().to_owned(), String::new())
        };

        println!("{} {} {}", name, description, file_count);

        records.push(Record {
            seq,
            name,
            description,
            file_count,
        });
    }

    // -----------------------------------------------------------------
    // 5. Write JSON (pretty-printed) to stdout or a file
    // -----------------------------------------------------------------
    let json = serde_json::to_string_pretty(&records)?;
    let mut out_file = File::create("output.json")?;
    out_file.write_all(json.as_bytes())?;
    println!("Wrote {} records → output.json", records.len());

    Ok(())
}

// ---------------------------------------------------------------------
// Helper: pull plain text out of a table cell (handles paragraphs, runs…)
// ---------------------------------------------------------------------
fn extract_text_from_cell(cell: &TableCell) -> String {
    let mut text = String::new();
    for content in &cell.children {
        if let docx_rs::TableCellContent::Paragraph(p) = content {
            for run in &p.children {
                if let docx_rs::ParagraphChild::Run(r) = run {
                    for run_child in &r.children {
                        if let docx_rs::RunChild::Text(t) = run_child {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
        }
    }
    text
}