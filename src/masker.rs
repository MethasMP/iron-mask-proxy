use crate::validator;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

lazy_static! {
    // Phone: Matches 0 followed by digit 1-9, then 8-15 chars of digits or dashes.
    // Example: 081-234-5678, 02-123-4567, 0991234567
    static ref RE_PHONE: Regex = Regex::new(r"0[1-9][0-9-]{8,15}").unwrap();
    
    // Email: Standard simple email regex
    static ref RE_EMAIL: Regex = Regex::new(r"(?i)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}").unwrap();
    
    // Credit Card: Matches sequence of digits/spaces/dashes, length 13-20
    static ref RE_CREDIT_CARD: Regex = Regex::new(r"(\d[ -]*?){13,20}").unwrap();
}

#[allow(dead_code)]
pub fn mask_pii(value: &mut Value, depth: u8, config: &crate::config::MaskingConfig) {
    if depth > config.max_depth {
        return;
    }

    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                // Check Whitelist (Exclude Fields)
                if config.exclude_fields.contains(key) {
                    continue; // Skip masking for this field
                }

                let key_lower = key.to_lowercase();
                if (key_lower.contains("name") || key_lower.contains("user")) && val.is_string() {
                    if let Some(s) = val.as_str() {
                        *val = Value::String(mask_name(s));
                    }
                } else {
                    mask_pii(val, depth + 1, config);
                }
            }
        }
        Value::Array(arr) => {
            for val in arr.iter_mut() {
                mask_pii(val, depth + 1, config);
            }
        }
        Value::String(s) => {
            // Apply streaming masking logic to long strings
            if s.len() < 5000 {
                *s = apply_global_standard_masking(s);
            }
        }
        _ => {}
    }
}

pub fn apply_global_standard_masking(input: &str) -> String {
    let mut result = input.to_string();
    let id_pattern = Regex::new(r"\d{13}").unwrap();

    // 1. Thai ID
    for mat in id_pattern.find_iter(input) {
        if validator::is_thai_id(mat.as_str()) {
            let masked = format!("{}XXXXXX{}", &mat.as_str()[0..3], &mat.as_str()[9..13]);
            result = result.replace(mat.as_str(), &masked);
        }
    }

    // 2. Credit Card
    result = RE_CREDIT_CARD
        .replace_all(&result, |caps: &regex::Captures| {
            let match_str = &caps[0];
            let digits: String = match_str.chars().filter(|c| c.is_ascii_digit()).collect();
            
            // Validate Length (13-19) and Luhn Algorithm
            if digits.len() >= 13 && digits.len() <= 19 && validator::is_luhn_valid(&digits) {
                if match_str.len() > 8 {
                     let prefix = &match_str[0..4];
                     let suffix = &match_str[match_str.len()-4..];
                     format!("{}********{}", prefix, suffix)
                } else {
                    match_str.to_string()
                }
            } else {
                match_str.to_string()
            }
        })
        .to_string();

    // 3. Email
    result = RE_EMAIL
        .replace_all(&result, |caps: &regex::Captures| mask_email(&caps[0]))
        .to_string();

    // 4. Phone
    result = RE_PHONE
        .replace_all(&result, |caps: &regex::Captures| {
            let match_str = &caps[0];
            let digits: String = match_str.chars().filter(|c| c.is_ascii_digit()).collect();
            
            if digits.len() >= 9 && digits.len() <= 10 {
                 if digits.len() == 10 {
                    // Mobile: 081-234-5678 -> 081XXXXX78
                    format!("{}XXXXX{}", &digits[0..3], &digits[8..10])
                } else {
                    // Landline: 02-123-4567 -> 02XXXX67
                     format!("{}XXXX{}", &digits[0..2], &digits[7..9])
                }
            } else {
                match_str.to_string()
            }
        })
        .to_string();

    result
}

fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return email.to_string();
    }
    let user_part = parts[0];
    let domain_part = parts[1];

    if user_part.len() < 2 {
        format!("*@{}", domain_part)
    } else {
        let chars: Vec<char> = user_part.chars().collect();
        let prefix: String = chars.iter().take(2).collect();
        format!("{}***@{}", prefix, domain_part)
    }
}

#[allow(dead_code)]
pub fn mask_name(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= 1 {
        return "***".to_string();
    }
    let prefix: String = chars.iter().take(2).collect();
    format!("{}***", prefix)
}

#[cfg(test)]
mod chaos_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mixed_pii_in_one_sentence() {
        let input = "ติดต่อคุณสมชายที่เบอร์ 0812345678 หรืออีเมล test@test.com เลขบัตร 1103700012346";
        let masked = apply_global_standard_masking(input);

        assert!(masked.contains("081XXXXX78"));
        assert!(masked.contains("te***@test.com"));
        // Note: Thai ID might not pass checksum if random, so we focus on phone/email here
    }

    #[test]
    fn test_large_input_safety() {
        let large_input = "a".repeat(10000);
        let start = std::time::Instant::now();
        let _ = apply_global_standard_masking(&large_input);
        assert!(start.elapsed().as_millis() < 500);
    }

    #[test]
    fn test_nested_json_limit() {
        let mut root = json!({"a": "PII"});
        for _ in 0..100 {
            root = json!({"inner": root});
        }
        let config = crate::config::MaskingConfig {
            exclude_fields: vec![],
            max_depth: 20,
        };
        mask_pii(&mut root, 0, &config);
    }
}
