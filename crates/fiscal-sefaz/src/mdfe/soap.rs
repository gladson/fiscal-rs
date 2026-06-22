//! SOAP 1.2 envelope construction for MDF-e web services.
//!
//! Mirrors [`crate::soap`] but targets the MDF-e portal namespace and the
//! `<mdfeDadosMsg>` body element. Internal to the crate —
//! [`crate::client::SefazClient`] wraps requests automatically.

use fiscal_core::FiscalError;

use super::{MDFE_NAMESPACE, MdfeServiceMeta};

const SOAP_NS: &str = "http://www.w3.org/2003/05/soap-envelope";

/// Build the WSDL operation namespace for a service
/// (`http://www.portalfiscal.inf.br/mdfe/wsdl/{operation}`).
fn wsdl_namespace(meta: &MdfeServiceMeta) -> String {
    format!("{MDFE_NAMESPACE}/wsdl/{}", meta.operation)
}

/// Build the SOAP 1.2 envelope wrapping an MDF-e request body in
/// `<mdfeDadosMsg>`.
///
/// ```xml
/// <soap:Envelope xmlns:xsi="…" xmlns:xsd="…" xmlns:soap="…/soap-envelope">
///   <soap:Body>
///     <mdfeDadosMsg xmlns="…/mdfe/wsdl/{operation}">
///       {request_xml}
///     </mdfeDadosMsg>
///   </soap:Body>
/// </soap:Envelope>
/// ```
///
/// MDF-e 3.00 omits the `<soap:Header>`/`nfeCabecMsg` block entirely, just
/// like NF-e 4.00.
pub(crate) fn build_envelope(request_xml: &str, meta: &MdfeServiceMeta) -> String {
    let namespace = wsdl_namespace(meta);
    let mut s = String::with_capacity(request_xml.len() + 400);

    s.push_str("<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"");
    s.push_str(SOAP_NS);
    s.push_str("\">");
    s.push_str("<soap:Body>");
    s.push_str("<mdfeDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(request_xml);
    s.push_str("</mdfeDadosMsg>");
    s.push_str("</soap:Body>");
    s.push_str("</soap:Envelope>");

    s
}

/// Build the SOAP 1.2 envelope for the **synchronous** reception service,
/// whose payload is gzip-compressed and Base64-encoded inside
/// `<mdfeDadosMsg>`.
///
/// Matches ACBr `MDFeRecepcaoSinc`: `EncodeBase64(GZipCompress(enviMDFe))`.
/// Unlike NF-e (which uses a distinct `<nfeDadosMsgZip>` element), MDF-e keeps
/// the same `<mdfeDadosMsg>` element and just places the Base64 blob inside it.
///
/// # Errors
///
/// Returns [`FiscalError::XmlGeneration`] if gzip compression fails.
pub(crate) fn build_envelope_compressed(
    request_xml: &str,
    meta: &MdfeServiceMeta,
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
    s.push_str("<mdfeDadosMsg xmlns=\"");
    s.push_str(&namespace);
    s.push_str("\">");
    s.push_str(&b64);
    s.push_str("</mdfeDadosMsg>");
    s.push_str("</soap:Body>");
    s.push_str("</soap:Envelope>");

    Ok(s)
}

/// Build the `SoapAction` URI for the HTTP `Content-Type` header.
///
/// Format: `http://www.portalfiscal.inf.br/mdfe/wsdl/{operation}/{method}`.
pub(crate) fn build_action(meta: &MdfeServiceMeta) -> String {
    format!("{MDFE_NAMESPACE}/wsdl/{}/{}", meta.operation, meta.method)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mdfe::MdfeService;

    #[test]
    fn envelope_uses_mdfe_dados_msg_and_mdfe_namespace() {
        let meta = MdfeService::StatusServico.meta();
        let body = "<consStatServMDFe><tpAmb>2</tpAmb></consStatServMDFe>";
        let env = build_envelope(body, &meta);

        assert!(env.starts_with("<soap:Envelope"));
        assert!(env.ends_with("</soap:Envelope>"));
        assert!(env.contains(
            "<mdfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeStatusServico\">"
        ));
        assert!(!env.contains("nfeDadosMsg"), "must not leak NF-e element");
        assert!(!env.contains("<soap:Header"), "MDF-e 3.00 omits header");
        assert!(env.contains(body));
    }

    #[test]
    fn envelope_exact_string() {
        let meta = MdfeService::StatusServico.meta();
        let body = "<consStatServMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\"><tpAmb>2</tpAmb><xServ>STATUS</xServ></consStatServMDFe>";
        let env = build_envelope(body, &meta);

        let expected = concat!(
            "<soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ",
            "xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ",
            "xmlns:soap=\"http://www.w3.org/2003/05/soap-envelope\">",
            "<soap:Body>",
            "<mdfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeStatusServico\">",
            "<consStatServMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\">",
            "<tpAmb>2</tpAmb><xServ>STATUS</xServ></consStatServMDFe>",
            "</mdfeDadosMsg>",
            "</soap:Body>",
            "</soap:Envelope>",
        );
        assert_eq!(env, expected);
    }

    #[test]
    fn action_uri_format() {
        assert_eq!(
            build_action(&MdfeService::StatusServico.meta()),
            "http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeStatusServico/mdfeStatusServicoMDF"
        );
        assert_eq!(
            build_action(&MdfeService::RecepcaoSinc.meta()),
            "http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeRecepcaoSinc/mdfeRecepcao"
        );
    }

    #[test]
    fn compressed_envelope_roundtrips_to_original_xml() {
        use base64::Engine as _;
        use flate2::read::GzDecoder;
        use std::io::Read;

        let meta = MdfeService::RecepcaoSinc.meta();
        let body = "<enviMDFe xmlns=\"http://www.portalfiscal.inf.br/mdfe\" versao=\"3.00\"><idLote>1</idLote><MDFe/></enviMDFe>";
        let env = build_envelope_compressed(body, &meta).unwrap();

        // Same element name as the plain envelope (no Zip variant).
        assert!(env.contains(
            "<mdfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeRecepcaoSinc\">"
        ));

        let start_tag =
            "<mdfeDadosMsg xmlns=\"http://www.portalfiscal.inf.br/mdfe/wsdl/MDFeRecepcaoSinc\">";
        let start = env.find(start_tag).unwrap() + start_tag.len();
        let end = env.find("</mdfeDadosMsg>").unwrap();
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
