use chrono::{NaiveDate, Weekday, Datelike};
use std::fs;
use webfs::models::files::parse_file_name;
use webfs::models::auth::{SigningKeys, SignUrlRequest, SignUrlResponse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut signing_keys = SigningKeys::new(3600, 3600);
    let req = SignUrlRequest::new("GET","https://example.com/files/report.pdf?user=alice");

    // Generate
    let signed_resp = signing_keys.generate_signed_url(&req)?;
    println!("Signed URL: {}", signed_resp.url);

    // Verify (should succeed)
    let verified_resp = SignUrlResponse::from_url("GET", &signed_resp.url)?;
    let verified = signing_keys.verify_signed_url(&verified_resp)?;
    println!("Verified: {}", verified);

    // Tamper with it (should fail)
    let tampered_url = signed_resp.url.replace("alice", "bob");
    let tampered_resp = SignUrlResponse::from_url("GET", &tampered_url)?;
    match signing_keys.verify_signed_url(&tampered_resp) {
        Ok(_) => println!("❌ Verification failed!"),
        Err(e) => println!("✅ Tampered URL rejected: {}", e),
    }
    Ok(())
}
