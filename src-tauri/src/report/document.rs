mod docx;
mod export;
mod html;
mod model;
mod pdf;
mod utils;

pub(crate) use export::export_file_meta;
pub(crate) use model::ReportDocument;

impl ReportDocument {
    pub(crate) fn render_html(&self) -> Vec<u8> {
        html::render_html(self)
    }

    pub(crate) fn render_pdf(&self) -> Vec<u8> {
        pdf::render_pdf(self)
    }

    pub(crate) fn render_docx(&self) -> Vec<u8> {
        docx::render_docx(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        docx::build_docx_package,
        model::{DocumentSection, ReportTemplate},
        pdf::build_simple_pdf,
    };
    #[test]
    fn simple_pdf_has_pdf_header_and_eof_marker() {
        let bytes = build_simple_pdf(&["MyLLMBenchmark 测试报告".to_string()]);

        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF"));
    }

    #[test]
    fn simple_pdf_creates_multiple_page_objects_for_long_reports() {
        let lines = (0..120)
            .map(|index| format!("第 {index} 行容量证据"))
            .collect::<Vec<_>>();
        let bytes = build_simple_pdf(&lines);
        let page_count = bytes
            .windows(b"/Type /Page /Parent".len())
            .filter(|window| *window == b"/Type /Page /Parent")
            .count();

        assert!(page_count >= 2);
    }

    #[test]
    fn docx_package_is_a_zip_with_word_document_part() {
        let bytes = build_docx_package(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>MyLLMBenchmark</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#,
        );

        assert!(bytes.starts_with(b"PK\x03\x04"));
        assert!(bytes
            .windows("word/document.xml".len())
            .any(|window| { window == "word/document.xml".as_bytes() }));
        assert!(bytes
            .windows("<w:tbl>".len())
            .any(|window| { window == "<w:tbl>".as_bytes() }));
    }

    #[test]
    fn report_templates_filter_sections_differently() {
        let sections = vec![
            section("执行摘要"),
            section("测试配置"),
            section("阶段证据"),
            section("错误分布"),
            section("上线建议"),
            section("附录"),
        ];

        let summary = ReportTemplate::DeliverySummary.filter_sections(sections.clone());
        let audit = ReportTemplate::DetailedAudit.filter_sections(sections);

        assert_eq!(summary.len(), 3);
        assert_eq!(audit.len(), 6);
    }

    fn section(title: &str) -> DocumentSection {
        DocumentSection {
            title: title.to_string(),
            paragraphs: Vec::new(),
            rows: Vec::new(),
        }
    }
}
