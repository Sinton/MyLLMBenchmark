use super::model::ReportDocument;

pub(crate) fn render_pdf(document: &ReportDocument) -> Vec<u8> {
    let lines = document.as_plain_lines();
    build_simple_pdf(&lines)
}

pub(crate) fn build_simple_pdf(lines: &[String]) -> Vec<u8> {
    let wrapped = wrap_pdf_lines(lines, 56);
    let pages = wrapped
        .chunks(42)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let pages = if pages.is_empty() {
        vec![vec!["MyLLMBenchmark 压测报告".to_string()]]
    } else {
        pages
    };
    let page_count = pages.len();
    let font_id = 3 + page_count * 2;
    let descendant_font_id = font_id + 1;
    let page_refs = (0..page_count)
        .map(|index| format!("{} 0 R", 3 + index))
        .collect::<Vec<_>>()
        .join(" ");

    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{page_refs}] /Count {page_count} >>"),
    ];
    for index in 0..page_count {
        let content_id = 3 + page_count + index;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
        ));
    }
    for (index, page_lines) in pages.iter().enumerate() {
        let content = pdf_page_content(page_lines, index + 1, page_count);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        ));
    }
    objects.push(format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /STSong-Light /Encoding /UniGB-UCS2-H /DescendantFonts [{descendant_font_id} 0 R] >>"
    ));
    objects.push(
        "<< /Type /Font /Subtype /CIDFontType0 /BaseFont /STSong-Light /CIDSystemInfo << /Registry (Adobe) /Ordering (GB1) /Supplement 2 >> >>"
            .to_string(),
    );

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }
    let xref_offset = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    pdf
}

fn wrap_pdf_lines(lines: &[String], max_chars: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            wrapped.push(String::new());
            continue;
        }
        let chars = line.chars().collect::<Vec<_>>();
        for chunk in chars.chunks(max_chars) {
            wrapped.push(chunk.iter().collect());
        }
    }
    wrapped
}

fn pdf_page_content(lines: &[String], page: usize, total: usize) -> String {
    let mut content = String::from("BT /F1 9 Tf 50 805 Td 13 TL ");
    content.push('<');
    content.push_str(&utf16be_hex("MyLLMBenchmark 压测报告"));
    content.push_str("> Tj T* ");
    content.push_str("0 -8 Td ");
    for line in lines {
        content.push('<');
        content.push_str(&utf16be_hex(line));
        content.push_str("> Tj T* ");
    }
    content.push_str("ET BT /F1 8 Tf 50 34 Td ");
    content.push('<');
    content.push_str(&utf16be_hex(&format!("第 {page} / {total} 页")));
    content.push_str("> Tj ET");
    content
}

fn utf16be_hex(value: &str) -> String {
    let mut hex = String::from("FEFF");
    for unit in value.encode_utf16() {
        hex.push_str(&format!("{unit:04X}"));
    }
    hex
}
