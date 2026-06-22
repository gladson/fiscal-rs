//! CT-e client methods on `SefazClient`.
//!
//! Mirror the MDF-e convenience methods (`status`, `consult`, `authorize`) but
//! target the CT-e authorizers and the `<cteDadosMsg>` SOAP envelope. The
//! synchronous reception (`CTeRecepcaoSincV4`) gzip-compresses its payload.
//! Event submission (`CTeRecepcaoEventoV4`) lands in a later phase together with
//! the event builders.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use super::SefazClient;
use crate::cte::request_builders::{
    build_cte_consulta_request, build_cte_recepcao_sinc_payload, build_cte_status_request,
};
use crate::cte::response_parsers::{
    CteAuthorizationResponse, CteConsultaResponse, StatusResponse,
    parse_cte_authorization_response, parse_cte_consulta_response, parse_cte_status_response,
};
use crate::cte::{CteService, get_cte_url, soap};
use crate::response_parsers::{CancellationResponse, parse_cancellation_response};

impl SefazClient {
    /// POST a built CT-e request body to the resolved endpoint and return the
    /// raw response XML. When `compressed` is set, the body is gzip-compressed
    /// and Base64-encoded inside `<cteDadosMsg>` (required by `CTeRecepcaoSincV4`).
    async fn send_cte(
        &self,
        service: CteService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        compressed: bool,
    ) -> Result<String, FiscalError> {
        let url = get_cte_url(uf, environment, service)?;
        let meta = service.meta();
        let envelope = if compressed {
            soap::build_envelope_compressed(request_xml, &meta)?
        } else {
            soap::build_envelope(request_xml, &meta)
        };
        let action = soap::build_action(&meta);
        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
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

        Ok(body)
    }

    /// Check the CT-e service status (`CTeStatusServicoV4`) for a UF.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn cte_status(
        &self,
        uf: &str,
        environment: SefazEnvironment,
    ) -> Result<StatusResponse, FiscalError> {
        let state_code = fiscal_core::state_codes::get_state_code(uf)?;
        let request_xml = build_cte_status_request(state_code, environment);
        let raw = self
            .send_cte(
                CteService::StatusServico,
                uf,
                environment,
                &request_xml,
                false,
            )
            .await?;
        parse_cte_status_response(&raw)
    }

    /// Consult a CT-e by its 44-digit access key (`CTeConsultaV4`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn cte_consult(
        &self,
        uf: &str,
        access_key: &str,
        environment: SefazEnvironment,
    ) -> Result<CteConsultaResponse, FiscalError> {
        let request_xml = build_cte_consulta_request(access_key, environment);
        let raw = self
            .send_cte(CteService::Consulta, uf, environment, &request_xml, false)
            .await?;
        parse_cte_consulta_response(&raw)
    }

    /// Authorize a signed CT-e synchronously (`CTeRecepcaoSincV4`).
    ///
    /// `signed_cte_xml` must be the complete, already-signed `<CTe>` document
    /// (signing happens upstream in `fiscal-cte`). The bare document is
    /// gzip-compressed and submitted; the parsed protocol is returned.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::XmlGeneration`] if compression fails,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn cte_authorize(
        &self,
        uf: &str,
        signed_cte_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<CteAuthorizationResponse, FiscalError> {
        let payload = build_cte_recepcao_sinc_payload(signed_cte_xml);
        let raw = self
            .send_cte(CteService::RecepcaoSinc, uf, environment, &payload, true)
            .await?;
        parse_cte_authorization_response(&raw)
    }

    /// Submit a signed GTV-e (`CTeRecepcaoGTVe`).
    ///
    /// `signed_gtve_xml` é o `<GTVe>` já assinado. O endpoint é derivado do
    /// RecepcaoSinc (troca o path por `CTeRecepcaoGTVe`); o corpo SOAP usa
    /// `<gtveDadosMsg>` (sem gzip). Retorna o protocolo parseado.
    ///
    /// # Errors
    ///
    /// Igual a [`Self::cte_authorize`].
    pub async fn cte_recepcao_gtve(
        &self,
        uf: &str,
        signed_gtve_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<CteAuthorizationResponse, FiscalError> {
        let base = get_cte_url(uf, environment, CteService::RecepcaoGTVe)?;
        let url = base.replace("CTeRecepcaoSincV4", "CTeRecepcaoGTVe");
        let meta = CteService::RecepcaoGTVe.meta();
        let envelope = soap::build_envelope_named(signed_gtve_xml, &meta, "gtveDadosMsg");
        let action = soap::build_action(&meta);
        let content_type = format!("application/soap+xml;charset=utf-8;action=\"{action}\"");

        let response = self
            .http
            .post(&url)
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
        parse_cte_authorization_response(&body)
    }

    /// Sign and submit a CT-e event (`CTeRecepcaoEventoV4`).
    ///
    /// `event_xml` is an unsigned `<eventoCTe>` built by one of the
    /// [`crate::cte::events`] builders (cancelamento, CCe). The `<infEvento>` is
    /// signed in place (RSA-SHA1) before transmission. The returned
    /// [`CancellationResponse`] carries the SEFAZ `cStat`/`xMotivo`/`nProt`, the
    /// signed event XML (`signed_event_xml`) and the raw response for archival.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Certificate`] if signing fails,
    /// [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn cte_recepcao_evento(
        &self,
        uf: &str,
        event_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<CancellationResponse, FiscalError> {
        let signed = fiscal_crypto::certificate::sign_cte_event_xml(
            event_xml,
            &self.private_key,
            &self.certificate,
        )?;
        let raw = self
            .send_cte(CteService::RecepcaoEvento, uf, environment, &signed, false)
            .await?;
        let mut parsed = parse_cancellation_response(&raw)?;
        parsed.signed_event_xml = signed;
        parsed.raw_response = raw;
        Ok(parsed)
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

    #[tokio::test]
    async fn cte_status_rejects_invalid_uf() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");
        let err = client
            .cte_status("XX", SefazEnvironment::Homologation)
            .await
            .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    #[tokio::test]
    async fn cte_authorize_rejects_invalid_uf() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");
        let err = client
            .cte_authorize(
                "ZZ",
                "<CTe xmlns=\"http://www.portalfiscal.inf.br/cte\"><infCte/></CTe>",
                SefazEnvironment::Homologation,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }
}
