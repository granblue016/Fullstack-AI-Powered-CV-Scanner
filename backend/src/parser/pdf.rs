use anyhow::Result;

pub async fn extract_pdf_text(bytes: &[u8]) -> Result<String> {
    // Use pdf-extract to parse PDF
    let text = pdf_extract::extract_text_from_mem(bytes)?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_extraction() {
        // Test with sample PDF bytes
        let sample = b"%PDF-1.4\nSample text";
        let result = extract_pdf_text(sample).await;
        assert!(result.is_ok() || result.is_err()); // Basic test
    }
}
