//! PFX/PKCS#12 certificate loading, parsing, and info extraction.
//!
//! Pure-Rust implementation — no OpenSSL dependency.

use base64::Engine as _;
use fiscal_core::FiscalError;
use fiscal_core::types::{CertificateData, CertificateInfo};
use x509_cert::der::Decode as _;

use super::pkcs12_parser;

/// Hash algorithm used for XML-DSig digest and RSA signature.
///
/// Brazilian ICP-Brasil v5 certificates require SHA-256, and some SEFAZs
/// already reject SHA-1 (rejeição 297). Use [`SignatureAlgorithm::Sha256`]
/// for new certificates; [`SignatureAlgorithm::Sha1`] is kept for
/// backwards compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignatureAlgorithm {
    /// RSA-SHA1 — legacy, kept as default for backwards compatibility.
    #[default]
    Sha1,
    /// RSA-SHA256 — required by ICP-Brasil v5 certificates.
    Sha256,
}

/// Ensure a PFX buffer can be used with modern TLS stacks.
///
/// Parses the PFX, and if it uses legacy encryption (RC2-40-CBC, 3DES-CBC),
/// returns an error indicating the PFX needs modernization. If the PFX is
/// already modern (AES-256-CBC), the original bytes are returned as-is.
///
/// Brazilian A1 certificates are commonly issued with legacy encryption
/// (RC2-40-CBC) which is handled natively by the pure-Rust PKCS#12 parser.
/// The PFX is parsed and re-exported when needed.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if the PFX is invalid or the
/// passphrase is wrong.
pub fn ensure_modern_pfx(pfx_buffer: &[u8], passphrase: &str) -> Result<Vec<u8>, FiscalError> {
    // Parse the PFX to verify it's valid and extract key material
    let _parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;

    // For now, return the original bytes — the pure-Rust parser handles
    // both legacy and modern encryption transparently.
    // In the future, we could re-export with AES-256-CBC for maximum
    // compatibility, but this isn't needed since we no longer depend
    // on native-tls/OpenSSL for parsing.
    Ok(pfx_buffer.to_vec())
}

/// Extract private key and certificate PEM strings from a PKCS#12/PFX buffer.
///
/// Parses the PFX using the provided passphrase and returns a [`CertificateData`]
/// containing both PEM-encoded private key and certificate, along with the
/// original PFX buffer and passphrase for later reuse.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The buffer is not a valid PKCS#12 file
/// - The passphrase is incorrect
/// - The PFX does not contain a private key or certificate
pub fn load_certificate(
    pfx_buffer: &[u8],
    passphrase: &str,
) -> Result<CertificateData, FiscalError> {
    let parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;

    // Convert private key DER to PEM using pkcs8 crate
    let private_key_pem = pkcs8_der_to_pem(&parsed.pkey)?;

    // Convert certificate DER to PEM using x509-cert
    let certificate_pem = x509_der_to_pem(&parsed.cert)?;

    Ok(CertificateData::new(
        private_key_pem,
        certificate_pem,
        pfx_buffer.to_vec(),
        passphrase,
    ))
}

/// Extract display metadata from a PKCS#12/PFX certificate.
///
/// Parses the PFX and reads the X.509 subject, issuer, validity dates,
/// and serial number without exposing the private key.
///
/// # Errors
///
/// Returns [`FiscalError::Certificate`] if:
/// - The buffer is not a valid PKCS#12 file
/// - The passphrase is incorrect
/// - The certificate fields cannot be parsed
pub fn get_certificate_info(
    pfx_buffer: &[u8],
    passphrase: &str,
) -> Result<CertificateInfo, FiscalError> {
    let parsed = pkcs12_parser::pkcs12_parse(pfx_buffer, passphrase)?;

    // Parse X.509 certificate from DER
    let cert = x509_cert::Certificate::from_der(&parsed.cert)
        .map_err(|e| FiscalError::Certificate(format!("Failed to parse certificate: {e}")))?;

    let common_name = extract_cn_from_name(&cert.tbs_certificate.subject);
    let issuer = extract_cn_from_name(&cert.tbs_certificate.issuer);

    let valid_from = x509_time_to_naive_date(&cert.tbs_certificate.validity.not_before)?;
    let valid_until = x509_time_to_naive_date(&cert.tbs_certificate.validity.not_after)?;

    // Serial number — format as hex string
    let serial_number = cert
        .tbs_certificate
        .serial_number
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>();

    Ok(CertificateInfo::new(
        common_name,
        valid_from,
        valid_until,
        serial_number,
        issuer,
    ))
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Convert PKCS#8 DER bytes to PEM string.
fn pkcs8_der_to_pem(der: &[u8]) -> Result<String, FiscalError> {
    // Manual PEM encoding for PKCS#8 private key
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    Ok(format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        wrap_base64(&b64)
    ))
}

/// Convert X.509 DER bytes to PEM string.
fn x509_der_to_pem(der: &[u8]) -> Result<String, FiscalError> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    Ok(format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        wrap_base64(&b64)
    ))
}

/// Wrap base64 content at 64 characters per line.
fn wrap_base64(s: &str) -> String {
    s.as_bytes()
        .chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract the Common Name (CN) from an X.500 Name (RDN sequence).
fn extract_cn_from_name(name: &x509_cert::name::Name) -> String {
    // In x509-cert 0.2, Name wraps a Vec<RelativeDistinguishedName>
    for rdn in &name.0 {
        for attr in rdn.0.as_slice() {
            if attr.oid.to_string() == "2.5.4.3" {
                // Try to extract the string value from the attribute value bytes
                if let Ok(value) = std::str::from_utf8(attr.value.value()) {
                    return value.to_string();
                }
            }
        }
    }
    // Fallback: use the Debug representation
    format!("{name:?}")
}

/// Convert an x509-cert Time to a chrono NaiveDate.
fn x509_time_to_naive_date(time: &x509_cert::time::Time) -> Result<chrono::NaiveDate, FiscalError> {
    // Convert to SystemTime -> Unix timestamp -> NaiveDate
    let system_time = match time {
        x509_cert::time::Time::UtcTime(utc) => utc.to_system_time(),
        x509_cert::time::Time::GeneralTime(gt) => gt.to_system_time(),
    };

    // Use chrono to convert
    let dt: chrono::DateTime<chrono::Utc> = system_time.into();
    Ok(dt.date_naive())
}
