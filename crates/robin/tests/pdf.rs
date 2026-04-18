use robin::pdf::{ PDFDocument };
use lopdf::{ Document };

mod pdf_document {
    use super::*;
    
    #[test]
    fn pdf_data_extracted_correctly() {
        let doc = Document::load("tests/data/multi-agent.pdf").expect("lopdf document loaded incorrectly");
        let mut pdf = PDFDocument::new(&doc, 0.0).unwrap();
        let total_bookmarks: usize = pdf.bookmarks.values().map(|v| v.len()).sum();
        let page1_text = pdf.text.get(&1).expect("page 1 is missing").replace("\n", "");
        
        assert!(pdf.bookmarks.contains_key(&1));
        assert_eq!(pdf.bookmarks[&1][0].title, "Introduction");
        assert_eq!(total_bookmarks, 25);
        assert!(page1_text.contains(
            "AI-powered development platforms are making software creation accessible to a broader audience, but this democratization has triggered a scalability crisis in security auditing. With studies showing that up to 40% of AI-generated code contains vulnerabilities, the pace of development now vastly outstrips the capacity for thorough security assessment. We present MAPTA, a multi-agent system for autonomous web application security assessment that combines large language model orchestration with tool-grounded execution and end-to-end exploit validation. On the 104-challenge XBOW benchmark, MAPTA achieves 76.9% overall success with perfect performance on SSRF and misconfiguration vulnerabilities, 83% success on broken authorization, and strong results on injection attacks including server-side template injection (85%) and SQL injection (83%). Cross-site scripting (57%) and blind SQL injection (0%) remain challenging. Our comprehensive cost analysis across all challenges totals $21.38 with a median cost of $0.073 for successful attempts versus $0.357 for failures. Success correlates strongly with resource efficiency, enabling practical early-stopping thresholds at approximately 40 tool calls or $0.30 per challenge. MAPTA's real-world findings are impactful given both the popularity of the respective scanned GitHub repositories (8K70K stars) and MAPTA's low average operating cost of $3.67 per open-source assessment: MAPTA discovered critical vulnerabilities including RCEs, command injections, secret exposure, and arbitrary file write vulnerabilities. Findings are responsibly disclosed, 10 findings are under CVE review."
        ));
    }
    
    #[test]
    fn no_outline_returns_empty_bookmarks() {
        let doc = Document::load("tests/data/smm-rootkit.pdf").expect("lopdf document loaded incorrectly");
        let pdf = PDFDocument::new(&doc, 0.0).unwrap();
        
        assert!(pdf.bookmarks.is_empty());
    }
}
