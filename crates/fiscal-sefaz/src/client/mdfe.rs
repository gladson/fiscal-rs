//! MDF-e client methods on `SefazClient`.
//!
//! These mirror the NF-e convenience methods (`status`, `consult`,
//! `authorize`) but target the MDF-e SVRS services and the `<mdfeDadosMsg>`
//! SOAP envelope. The synchronous reception (`MDFeRecepcaoSinc`) gzip-compresses
//! its payload.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use super::SefazClient;
use crate::mdfe::events::{MdfeEventResponse, parse_mdfe_event_response};
use crate::mdfe::request_builders::{
    build_mdfe_consulta_request, build_mdfe_recepcao_sinc_payload, build_mdfe_status_request,
};
use crate::mdfe::response_parsers::{
    MdfeAuthorizationResponse, MdfeConsultaResponse, StatusResponse,
    parse_mdfe_authorization_response, parse_mdfe_consulta_response, parse_mdfe_status_response,
};
use crate::mdfe::{MdfeService, get_mdfe_url, soap};

impl SefazClient {
    /// POST a built MDF-e request body to the resolved SVRS endpoint and return
    /// the raw response XML.
    ///
    /// When `compressed` is set, the body is gzip-compressed and Base64-encoded
    /// inside `<mdfeDadosMsg>` (required by `MDFeRecepcaoSinc`).
    async fn send_mdfe(
        &self,
        service: MdfeService,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
        compressed: bool,
    ) -> Result<String, FiscalError> {
        let url = get_mdfe_url(uf, environment, service)?;
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

    /// Check the MDF-e service status (`MDFeStatusServico`) for a UF.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn mdfe_status(
        &self,
        uf: &str,
        environment: SefazEnvironment,
    ) -> Result<StatusResponse, FiscalError> {
        let request_xml = build_mdfe_status_request(environment);
        let raw = self
            .send_mdfe(
                MdfeService::StatusServico,
                uf,
                environment,
                &request_xml,
                false,
            )
            .await?;
        parse_mdfe_status_response(&raw)
    }

    /// Consult an MDF-e by its 44-digit access key (`MDFeConsulta`).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn mdfe_consult(
        &self,
        uf: &str,
        access_key: &str,
        environment: SefazEnvironment,
    ) -> Result<MdfeConsultaResponse, FiscalError> {
        let request_xml = build_mdfe_consulta_request(access_key, environment);
        let raw = self
            .send_mdfe(MdfeService::Consulta, uf, environment, &request_xml, false)
            .await?;
        parse_mdfe_consulta_response(&raw)
    }

    /// Authorize a signed MDF-e synchronously (`MDFeRecepcaoSinc`).
    ///
    /// `signed_mdfe_xml` must be the complete, already-signed `<MDFe>` document
    /// (signing happens upstream in `fiscal-mdfe`). The XML is wrapped in
    /// `<enviMDFe>`, gzip-compressed, and submitted; the parsed protocol is
    /// returned.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::XmlGeneration`] if compression fails,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn mdfe_authorize(
        &self,
        uf: &str,
        signed_mdfe_xml: &str,
        lot_id: &str,
        environment: SefazEnvironment,
    ) -> Result<MdfeAuthorizationResponse, FiscalError> {
        let payload = build_mdfe_recepcao_sinc_payload(signed_mdfe_xml, lot_id);
        let raw = self
            .send_mdfe(MdfeService::RecepcaoSinc, uf, environment, &payload, true)
            .await?;
        parse_mdfe_authorization_response(&raw)
    }

    /// Sign and submit an MDF-e event (`MDFeRecepcaoEvento`).
    ///
    /// `event_xml` is an unsigned `<eventoMDFe>` built by one of the
    /// [`crate::mdfe::events`] builders (encerramento, cancelamento, …). The
    /// `<infEvento>` is signed in place (RSA-SHA1) before transmission; the
    /// returned [`MdfeEventResponse`] carries the signed event XML and the raw
    /// SEFAZ response for archival.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Certificate`] if signing fails,
    /// [`FiscalError::InvalidStateCode`] for an unknown UF,
    /// [`FiscalError::Network`] on transport failure, or
    /// `FiscalError::XmlParsing` if the response is malformed.
    pub async fn mdfe_recepcao_evento(
        &self,
        uf: &str,
        event_xml: &str,
        environment: SefazEnvironment,
    ) -> Result<MdfeEventResponse, FiscalError> {
        let signed = fiscal_crypto::certificate::sign_mdfe_event_xml(
            event_xml,
            &self.private_key,
            &self.certificate,
        )?;
        let raw = self
            .send_mdfe(MdfeService::RecepcaoEvento, uf, environment, &signed, false)
            .await?;
        let mut parsed = parse_mdfe_event_response(&raw)?;
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

    // Transmission itself needs a live SEFAZ endpoint; here we only assert the
    // request-building / URL-resolution paths are wired (invalid UF rejected
    // before any network I/O).
    #[tokio::test]
    async fn mdfe_status_rejects_invalid_uf() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");
        let err = client
            .mdfe_status("XX", SefazEnvironment::Homologation)
            .await
            .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    #[tokio::test]
    async fn mdfe_consult_rejects_invalid_uf() {
        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");
        let err = client
            .mdfe_consult(
                "ZZ",
                "43250612345678000190580010000000011000000017",
                SefazEnvironment::Homologation,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }

    // Event signing is local (no network), so we can verify the full
    // build → sign path: the `<Signature>` must land inside `<eventoMDFe>`,
    // after `</infEvento>`, referencing the event Id.
    #[test]
    fn signs_encerramento_event_inside_evento_mdfe() {
        use crate::mdfe::events::build_mdfe_encerramento;

        let client = SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds");
        let key = "43250612345678000190580010000000011000000017";
        let event = build_mdfe_encerramento(
            key,
            "143250000000123",
            "2026-06-05",
            "43",
            "4314902",
            1,
            "06157250000116",
            SefazEnvironment::Homologation,
        );
        assert!(!event.contains("<Signature"));

        let signed = fiscal_crypto::certificate::sign_mdfe_event_xml(
            &event,
            &client.private_key,
            &client.certificate,
        )
        .expect("event signs");

        assert!(signed.contains("<Signature"));
        assert!(signed.contains(&format!("Reference URI=\"#ID110112{key}01\"")));
        let inf_close = signed.find("</infEvento>").expect("infEvento closes");
        let sig = signed.find("<Signature").expect("Signature present");
        let evt_close = signed.find("</eventoMDFe>").expect("eventoMDFe closes");
        assert!(
            inf_close < sig && sig < evt_close,
            "<Signature> must sit after </infEvento> and before </eventoMDFe>"
        );
        assert!(signed.contains("<X509Certificate>"));
    }
}
