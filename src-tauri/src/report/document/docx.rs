use super::model::{summary_rows, ReportDocument};
use super::utils::escape_xml;

pub(crate) fn render_docx(document: &ReportDocument) -> Vec<u8> {
    let document_xml = render_docx_document_xml(document);
    build_docx_package(&document_xml)
}

fn render_docx_document_xml(document: &ReportDocument) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push_str(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
    );
    push_docx_heading(&mut xml, &document.title, "Title");
    push_docx_paragraph(&mut xml, &document.subtitle);
    push_docx_paragraph(&mut xml, &format!("模板：{}", document.template.label()));
    push_docx_paragraph(&mut xml, &format!("数据来源：{}", document.source_label));
    push_docx_table(&mut xml, &summary_rows(&document.summary));
    for section in &document.sections {
        push_docx_heading(&mut xml, &section.title, "Heading1");
        for paragraph in &section.paragraphs {
            push_docx_paragraph(&mut xml, paragraph);
        }
        if !section.rows.is_empty() {
            push_docx_table(&mut xml, &section.rows);
        }
    }
    xml.push_str(r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>"#);
    xml.push_str("</w:body></w:document>");
    xml
}

fn push_docx_heading(xml: &mut String, text: &str, style: &str) {
    xml.push_str(r#"<w:p><w:pPr><w:pStyle w:val=""#);
    xml.push_str(style);
    xml.push_str(r#""/></w:pPr><w:r><w:t>"#);
    xml.push_str(&escape_xml(text));
    xml.push_str("</w:t></w:r></w:p>");
}

fn push_docx_paragraph(xml: &mut String, text: &str) {
    xml.push_str("<w:p><w:r><w:t>");
    xml.push_str(&escape_xml(text));
    xml.push_str("</w:t></w:r></w:p>");
}

fn push_docx_table(xml: &mut String, rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    xml.push_str(
        r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top w:val="single" w:sz="4" w:color="D8CEC6"/><w:left w:val="single" w:sz="4" w:color="D8CEC6"/><w:bottom w:val="single" w:sz="4" w:color="D8CEC6"/><w:right w:val="single" w:sz="4" w:color="D8CEC6"/><w:insideH w:val="single" w:sz="4" w:color="E6DDD6"/><w:insideV w:val="single" w:sz="4" w:color="E6DDD6"/></w:tblBorders></w:tblPr>"#,
    );
    for (row_index, row) in rows.iter().enumerate() {
        xml.push_str("<w:tr>");
        for cell in row {
            xml.push_str("<w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/>");
            if row_index == 0 {
                xml.push_str(r#"<w:shd w:fill="F2EBE5"/>"#);
            }
            xml.push_str("</w:tcPr><w:p><w:r>");
            if row_index == 0 {
                xml.push_str("<w:rPr><w:b/></w:rPr>");
            }
            xml.push_str("<w:t>");
            xml.push_str(&escape_xml(cell));
            xml.push_str("</w:t></w:r></w:p></w:tc>");
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
}

pub(crate) fn build_docx_package(document_xml: &str) -> Vec<u8> {
    let files = vec![
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_bytes().to_vec(),
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_bytes().to_vec(),
        ),
        ("word/document.xml", document_xml.as_bytes().to_vec()),
    ];
    build_store_zip(files)
}

fn build_store_zip(files: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
    let mut zip = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let offset = zip.len() as u32;
        let crc = crc32(&data);
        let name_bytes = name.as_bytes();
        write_u32(&mut zip, 0x0403_4b50);
        write_u16(&mut zip, 20);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u32(&mut zip, crc);
        write_u32(&mut zip, data.len() as u32);
        write_u32(&mut zip, data.len() as u32);
        write_u16(&mut zip, name_bytes.len() as u16);
        write_u16(&mut zip, 0);
        zip.extend_from_slice(name_bytes);
        zip.extend_from_slice(&data);

        write_u32(&mut central, 0x0201_4b50);
        write_u16(&mut central, 20);
        write_u16(&mut central, 20);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, crc);
        write_u32(&mut central, data.len() as u32);
        write_u32(&mut central, data.len() as u32);
        write_u16(&mut central, name_bytes.len() as u16);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, 0);
        write_u32(&mut central, offset);
        central.extend_from_slice(name_bytes);
    }
    let central_offset = zip.len() as u32;
    let central_size = central.len() as u32;
    let file_count = count_central_entries(&central);
    zip.extend_from_slice(&central);
    write_u32(&mut zip, 0x0605_4b50);
    write_u16(&mut zip, 0);
    write_u16(&mut zip, 0);
    write_u16(&mut zip, file_count);
    write_u16(&mut zip, file_count);
    write_u32(&mut zip, central_size);
    write_u32(&mut zip, central_offset);
    write_u16(&mut zip, 0);
    zip
}

fn count_central_entries(central: &[u8]) -> u16 {
    central
        .windows(4)
        .filter(|window| *window == [0x50, 0x4b, 0x01, 0x02])
        .count() as u16
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn write_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}
