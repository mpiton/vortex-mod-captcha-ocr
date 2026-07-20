//! Tesseract OCR CAPTCHA solver for Vortex.

#[cfg(target_family = "wasm")]
mod plugin_api;

use serde::{Deserialize, Serialize};

const MAX_SOLUTION_BYTES: usize = 4_096;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptchaRequest {
    pub challenge_id: String,
    pub challenge_type: String,
    pub challenge_url: String,
    pub image_data: Option<String>,
}

#[derive(Debug, Serialize)]
struct TesseractRequest<'a> {
    image_data: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SolverResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    solution: Option<String>,
}

pub fn handle_can_solve(input: &str) -> Result<String, String> {
    let request = parse_request(input)?;
    Ok((request.challenge_type == "image" && request.image_data.is_some()).to_string())
}

pub fn build_tesseract_request(input: &str) -> Result<String, String> {
    let request = parse_request(input)?;
    if request.challenge_type != "image" {
        return Err("unsupported CAPTCHA type".into());
    }
    let image_data = request
        .image_data
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "CAPTCHA image is missing".to_string())?;
    serde_json::to_string(&TesseractRequest { image_data })
        .map_err(|_| "could not encode Tesseract request".into())
}

pub fn normalize_tesseract_response(input: &str) -> Result<String, String> {
    let mut response: SolverResponse =
        serde_json::from_str(input).map_err(|_| "invalid Tesseract response".to_string())?;
    match (response.status.as_str(), response.solution.as_mut()) {
        ("solved", Some(solution)) => {
            *solution = solution.trim().to_string();
            if solution.is_empty() || solution.len() > MAX_SOLUTION_BYTES {
                return Err("invalid OCR solution".into());
            }
        }
        ("unavailable" | "rejected", None) => {}
        _ => return Err("invalid Tesseract status payload".into()),
    }
    serde_json::to_string(&response).map_err(|_| "could not encode solver response".into())
}

fn parse_request(input: &str) -> Result<CaptchaRequest, String> {
    serde_json::from_str(input).map_err(|_| "invalid CAPTCHA request".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMAGE_REQUEST: &str = r#"{"challenge_id":"captcha-1","challenge_type":"image","challenge_url":"https://example.test","image_data":"aW1hZ2U="}"#;

    #[test]
    fn only_image_challenges_with_bytes_are_supported() {
        assert_eq!(handle_can_solve(IMAGE_REQUEST).unwrap(), "true");
        assert_eq!(
            handle_can_solve(r#"{"challenge_id":"1","challenge_type":"text_input","challenge_url":"https://example.test","image_data":null}"#).unwrap(),
            "false"
        );
    }

    #[test]
    fn broker_request_exposes_only_image_data() {
        let request: serde_json::Value =
            serde_json::from_str(&build_tesseract_request(IMAGE_REQUEST).unwrap()).unwrap();
        assert_eq!(request, serde_json::json!({ "image_data": "aW1hZ2U=" }));
    }

    #[test]
    fn response_is_strict_and_solution_is_trimmed() {
        assert_eq!(
            normalize_tesseract_response(r#"{"status":"solved","solution":"  abc  "}"#).unwrap(),
            r#"{"status":"solved","solution":"abc"}"#
        );
        assert!(normalize_tesseract_response(
            r#"{"status":"solved","solution":"abc","diagnostic":"secret"}"#
        )
        .is_err());
    }
}
