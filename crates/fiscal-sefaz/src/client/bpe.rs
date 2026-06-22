//! BP-e (model 63) client methods on `SefazClient`.
//!
//! BP-e is authorized by **SVRS** for almost every UF (MG self-hosts). The
//! recepção service is synchronous (`BPeRecepcao`, body `<bpeDadosMsg>`).

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;
use fiscal_core::xml_utils::extract_xml_tag_value;

use super::SefazClient;

const BPE_NAMESPACE: &str = "http://www.portalfiscal.inf.br/bpe";
const SOAP_NS: &str = "http://www.w3.org/2003/05/soap-envelope";

/// Resultado da recepção de um BP-e.
#[derive(Debug, Clone)]
pub struct BpeAuthResponse {
    pub status_code: String,
    pub status_message: String,
    pub protocol_number: Option<String>,
    /// `<protBPe>` completo (para arquivar no bpeProc), quando autorizado.
    pub protocol_xml: Option<String>,
}

fn bpe_recepcao_url(uf: &str, env: SefazEnvironment) -> &'static str {
    let prod = env == SefazEnvironment::Production;
    match uf {
        "MG" => {
            if prod {
                "https://bpe.fazenda.mg.gov.br/bpe/services/BPeRecepcao"
            } else {
                "https://hbpe.fazenda.mg.gov.br/bpe/services/BPeRecepcao"
            }
        }
        _ => {
            if prod {
                "https://bpe.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
            } else {
                "https://bpe-homologacao.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
            }
        }
    }
}

fn extract_element(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let end = xml.find(&close)? + close.len();
    Some(xml[start..end].to_string())
}

impl SefazClient {
    /// Submit a signed BP-e (`BPeRecepcao`, síncrono).
    ///
    /// `signed_bpe_xml` é o `<BPe>` já assinado (com `<infBPeSupl>`). Retorna o
    /// `cStat`/`xMotivo`/`nProt` e o `<protBPe>` para arquivamento.
    ///
    /// # Errors
    ///
    /// [`FiscalError::Network`] em falha de transporte; `FiscalError::XmlParsing`
    /// se a resposta não tiver `cStat`.
    pub async fn bpe_recepcao(
        &self,
        uf: &str,
        signed_bpe_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<BpeAuthResponse, FiscalError> {
        let url = bpe_recepcao_url(uf, environment);
        let namespace = format!("{BPE_NAMESPACE}/wsdl/BPeRecepcao");
        let envelope = format!(
            "<soap:Envelope xmlns:soap=\"{SOAP_NS}\"><soap:Body><bpeDadosMsg xmlns=\"{namespace}\">{signed_bpe_xml}</bpeDadosMsg></soap:Body></soap:Envelope>"
        );
        let action = format!("{namespace}/bpeRecepcao");
        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(url)
            .header("Content-Type", &content_type)
            .body(envelope)
            .send()
            .await
            .map_err(|e| FiscalError::Network(format!("{e}")))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FiscalError::Network(format!("Failed to read response body: {e}")))?;
        if !status.is_success() {
            return Err(FiscalError::Network(format!(
                "SEFAZ returned HTTP {status}: {body}"
            )));
        }

        let status_code = extract_xml_tag_value(&body, "cStat")
            .ok_or_else(|| FiscalError::XmlParsing("missing <cStat> in BPe response".into()))?;
        let status_message =
            extract_xml_tag_value(&body, "xMotivo").unwrap_or_else(|| "Unknown".into());
        let protocol_number = extract_xml_tag_value(&body, "nProt");
        let protocol_xml = extract_element(&body, "protBPe");

        Ok(BpeAuthResponse {
            status_code,
            status_message,
            protocol_number,
            protocol_xml,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pfx() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("test PFX not found")
    }

    const TEST_PASSWORD: &str = "minhasenha";

    // ── bpe_recepcao_url ──────────────────────────────────────────────

    #[test]
    fn bpe_recepcao_url_mg_production() {
        assert_eq!(
            super::bpe_recepcao_url("MG", SefazEnvironment::Production),
            "https://bpe.fazenda.mg.gov.br/bpe/services/BPeRecepcao"
        );
    }

    #[test]
    fn bpe_recepcao_url_mg_homologation() {
        assert_eq!(
            super::bpe_recepcao_url("MG", SefazEnvironment::Homologation),
            "https://hbpe.fazenda.mg.gov.br/bpe/services/BPeRecepcao"
        );
    }

    #[test]
    fn bpe_recepcao_url_svrs_production() {
        // Non-MG UFs route to SVRS.
        assert_eq!(
            super::bpe_recepcao_url("SP", SefazEnvironment::Production),
            "https://bpe.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
        );
        assert_eq!(
            super::bpe_recepcao_url("RJ", SefazEnvironment::Production),
            "https://bpe.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
        );
    }

    #[test]
    fn bpe_recepcao_url_svrs_homologation() {
        assert_eq!(
            super::bpe_recepcao_url("SP", SefazEnvironment::Homologation),
            "https://bpe-homologacao.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
        );
    }

    // ── extract_element ───────────────────────────────────────────────

    #[test]
    fn extract_element_extracts_tag_with_content() {
        let xml = "<root><protBPe><infProt>123</infProt></protBPe></root>";
        let result = super::extract_element(xml, "protBPe");
        assert_eq!(
            result.as_deref(),
            Some("<protBPe><infProt>123</infProt></protBPe>")
        );
    }

    #[test]
    fn extract_element_returns_none_when_tag_absent() {
        let xml = "<root><other>data</other></root>";
        assert_eq!(super::extract_element(xml, "protBPe"), None);
    }

    #[test]
    fn extract_element_handles_empty_xml() {
        assert_eq!(super::extract_element("", "protBPe"), None);
    }

    // ── BpeAuthResponse ───────────────────────────────────────────────

    #[test]
    fn bpe_auth_response_debug_contains_fields() {
        let resp = BpeAuthResponse {
            status_code: "100".into(),
            status_message: "Autorizado o uso".into(),
            protocol_number: Some("123456789".into()),
            protocol_xml: Some("<protBPe/>".into()),
        };
        let debug = format!("{resp:?}");
        assert!(debug.contains("100"));
        assert!(debug.contains("Autorizado o uso"));
    }

    // ── bpe_recepcao (unknown UF falls through to SVRS) ─────────────
    // Unlike other services, bpe_recepcao does NOT validate the UF — unknown
    // UFs fall through to the SVRS URL. The error would be a network failure,
    // not InvalidStateCode.

    #[test]
    fn bpe_recepcao_url_unknown_uf_falls_through_to_svrs() {
        // The match has `_ =>` arm, so any unknown UF routes to SVRS.
        assert_eq!(
            super::bpe_recepcao_url("XX", SefazEnvironment::Production),
            "https://bpe.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
        );
        assert_eq!(
            super::bpe_recepcao_url("ZZ", SefazEnvironment::Homologation),
            "https://bpe-homologacao.svrs.rs.gov.br/ws/bpeRecepcao/bpeRecepcao.asmx"
        );
    }
}
