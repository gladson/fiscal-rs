//! SOAP 1.2 envelope construction for CT-e web services.
//!
//! Targets the CT-e portal namespace and the `<cteDadosMsg>` body element.
//! Internal to the crate — [`crate::client::SefazClient`] wraps requests
//! automatically.

use fiscal_core::FiscalError;

use super::{CTE_NAMESPACE, CteServiceMeta};

const SOAP_NS: &str = "http://www.w3.org/2003/05/soap-envelope";

/// WSDL operation namespace (`http://www.portalfiscal.inf.br/cte/wsdl/{operation}`).
fn wsdl_namespace(meta: &CteServiceMeta) -> String {
    format!("{CTE_NAMESPACE}/wsdl/{}", meta.operation)
}

/// Build the SOAP 1.2 envelope wrapping a CT-e request body in `<cteDadosMsg>`.
pub(crate) fn build_envelope(request_xml: &str, meta: &CteServiceMeta) -> String {
    build_envelope_named(request_xml, meta, "cteDadosMsg")
}

/// Like [`build_envelope`] but with a custom body element (e.g. `gtveDadosMsg`
/// para a GTV-e).
pub(crate) fn build_envelope_named(
    request_xml: &str,
    meta: &CteServiceMeta,
    body_elem: &str,
) -> String {
    let namespace = wsdl_namespace(meta);
    let mut s = String::with_capacity(request_xml.len() + 400);

    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");
    s.push_str("<soap:Body>");
    s.push('<');
    s.push_str(body_elem);
    s.push_str(" xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(request_xml);
    s.push_str("</");
    s.push_str(body_elem);
    s.push('>');
    s.push_str("</soap:Body>");
    s.push_str("</soap:Envelope>");

    s
}

/// Build the SOAP 1.2 envelope for the **synchronous** reception service, whose
/// payload is gzip-compressed and Base64-encoded inside `<cteDadosMsg>`.
///
/// Matches sped-cte `CteRecepcao`: `EncodeBase64(GZipCompress(<CTe>))`.
///
/// # Errors
///
/// Returns [`FiscalError::XmlGeneration`] if gzip compression fails.
pub(crate) fn build_envelope_compressed(
    request_xml: &str,
    meta: &CteServiceMeta,
) -> Result<String, FiscalError> {
    use base64::Engine as _;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(request_xml.as_bytes())
        .map_err(|e| FiscalError::XmlGeneration(format!("Gzip compression failed: {e}")))?;
    let compressed = encoder
        .finish()
        .map_err(|e| FiscalError::XmlGeneration(format!("Gzip compression failed: {e}")))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&compressed);

    let namespace = wsdl_namespace(meta);
    let mut s = String::with_capacity(b64.len() + 400);

    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");
    s.push_str("<soap:Body>");
    s.push_str("<cteDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(&b64);
    s.push_str("</cteDadosMsg>");
    s.push_str("</soap:Body>");
    s.push_str("</soap:Envelope>");

    Ok(s)
}

/// Build the `SoapAction` URI for the HTTP `Content-Type` header.
///
/// Format: `http://www.portalfiscal.inf.br/cte/wsdl/{operation}/{method}`.
pub(crate) fn build_action(meta: &CteServiceMeta) -> String {
    format!("{CTE_NAMESPACE}/wsdl/{}/{}", meta.operation, meta.method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cte::CteService;

    #[test]
    fn envelope_uses_cte_dados_msg_and_cte_namespace() {
        let meta = CteService::StatusServico.meta();
        let body = "<consStatServCte><tpAmb>2</tpAmb></consStatServCte>";
        let env = build_envelope(body, &meta);

        assert!(env.starts_with("<soap:Envelope"));
        assert!(env.ends_with("</soap:Envelope>"));
        assert!(env.contains(
            "<cteDadosMsg xmlns=\"http://www.portalfiscal.inf.br/cte/wsdl/CTeStatusServicoV4\">"
        ));
        assert!(!env.contains("mdfeDadosMsg"), "must not leak MDF-e element");
        assert!(!env.contains("nfeDadosMsg"), "must not leak NF-e element");
        assert!(env.contains(body));
    }

    #[test]
    fn action_uri_format() {
        assert_eq!(
            build_action(&CteService::RecepcaoSinc.meta()),
            "http://www.portalfiscal.inf.br/cte/wsdl/CTeRecepcaoSincV4/cteRecepcao"
        );
        assert_eq!(
            build_action(&CteService::StatusServico.meta()),
            "http://www.portalfiscal.inf.br/cte/wsdl/CTeStatusServicoV4/cteStatusServicoCT"
        );
    }

    #[test]
    fn compressed_envelope_roundtrips_to_original_xml() {
        use base64::Engine as _;
        use flate2::read::GzDecoder;
        use std::io::Read;

        let meta = CteService::RecepcaoSinc.meta();
        let body = "<CTe xmlns=\"http://www.portalfiscal.inf.br/cte\"><infCte/></CTe>";
        let env = build_envelope_compressed(body, &meta).unwrap();

        let start_tag =
            "<cteDadosMsg xmlns=\"http://www.portalfiscal.inf.br/cte/wsdl/CTeRecepcaoSincV4\">";
        assert!(env.contains(start_tag));
        let start = env.find(start_tag).unwrap() + start_tag.len();
        let end = env.find("</cteDadosMsg>").unwrap();
        let b64 = &env[start..end];

        let compressed = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .expect("valid base64");
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut out = String::new();
        decoder.read_to_string(&mut out).expect("valid gzip");
        assert_eq!(out, body);
    }
}
