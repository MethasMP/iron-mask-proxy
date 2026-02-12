// เพิ่ม Luhn เพื่อเช็กบัตรเครดิตจริง
pub fn is_luhn_valid(s: &str) -> bool {
    let digits: Vec<u32> = s.chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 { return false; }

    let mut sum = 0;
    let mut double = false;
    for &digit in digits.iter().rev() {
        let mut d = digit;
        if double {
            d *= 2;
            if d > 9 { d -= 9; }
        }
        sum += d;
        double = !double;
    }
    sum % 10 == 0
}

// src/validator.rs
pub fn is_thai_id(id: &str) -> bool {
    // 1. ตรวจสอบเบื้องต้น
    if id.len() != 13 || !id.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // 2. คำนวณ Checksum แบบ Functional Style (เลี่ยงการสร้าง Vec ใหม่เพื่อประหยัด RAM)
    let sum: u32 = id.chars()
        .take(12)
        .enumerate()
        .filter_map(|(i, c)| c.to_digit(10).map(|d| d * (13 - i as u32)))
        .sum();

    let check_digit = (11 - (sum % 11)) % 10;

    // 3. ตรวจสอบหลักสุดท้าย
    id.chars().last().and_then(|c| c.to_digit(10)) == Some(check_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_thai_id() {
        assert!(is_thai_id("1103700012346")); // เลขที่มี checksum ถูกต้อง
    }

    #[test]
    fn test_invalid_thai_id() {
        assert!(!is_thai_id("1103700012345")); // เลขที่ checksum ผิด
    }
}