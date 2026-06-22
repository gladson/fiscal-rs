//! Delivery receipt, delivery failure, prorrogação, and actor registration methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders;
use crate::response_parsers::{self, CancellationResponse};
use crate::services::SefazService;

use super::SefazClient;

impl SefazClient {
    /// Register an interested actor for an NF-e (`RecepcaoEvento4`,
    /// tpEvento=110150).
    ///
    /// Authorizes a transporter to access the NF-e. Sent to
    /// Ambiente Nacional (AN).
    ///
    /// # Arguments
    ///
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `tp_autor` — Author type (1=emitente, 2=destinatario, 3=transportador).
    /// * `ver_aplic` — Version of the issuing application.
    /// * `authorized_cnpj` — Optional CNPJ to authorize.
    /// * `authorized_cpf` — Optional CPF to authorize.
    /// * `tp_autorizacao` — Authorization type (0=no subcontract, 1=allowed).
    /// * `issuer_uf` — UF of the issuer.
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the event sender.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn ator_interessado(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        tp_autor: u8,
        ver_aplic: &str,
        authorized_cnpj: Option<&str>,
        authorized_cpf: Option<&str>,
        tp_autorizacao: u8,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_ator_interessado_request(
            access_key,
            tp_autor,
            ver_aplic,
            authorized_cnpj,
            authorized_cpf,
            tp_autorizacao,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Register a delivery receipt for an NF-e (`RecepcaoEvento4`,
    /// tpEvento=110130).
    ///
    /// Records proof of delivery. Sent to Ambiente Nacional (AN).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn comprovante_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        delivery_date: &str,
        doc_number: &str,
        name: &str,
        lat: Option<&str>,
        long: Option<&str>,
        hash: &str,
        hash_date: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_comprovante_entrega_request(
            access_key,
            ver_aplic,
            delivery_date,
            doc_number,
            name,
            lat,
            long,
            hash,
            hash_date,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a delivery receipt event (`RecepcaoEvento4`,
    /// tpEvento=110131).
    ///
    /// Cancels a previously registered delivery receipt. Sent to AN.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_comprovante_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        event_protocol: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_comprovante_entrega_request(
            access_key,
            ver_aplic,
            event_protocol,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Register a delivery failure event (`RecepcaoEvento4`,
    /// tpEvento=110192).
    ///
    /// Records a failed delivery attempt. Sent to Ambiente Nacional (AN).
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn insucesso_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        attempt_date: &str,
        attempt_number: Option<u32>,
        reason_type: u8,
        reason_justification: Option<&str>,
        lat: Option<&str>,
        long: Option<&str>,
        hash: &str,
        hash_date: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_insucesso_entrega_request(
            access_key,
            ver_aplic,
            attempt_date,
            attempt_number,
            reason_type,
            reason_justification,
            lat,
            long,
            hash,
            hash_date,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a delivery failure event (`RecepcaoEvento4`,
    /// tpEvento=110193).
    ///
    /// Cancels a previously registered delivery failure. Sent to AN.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_insucesso_entrega(
        &self,
        environment: SefazEnvironment,
        access_key: &str,
        ver_aplic: &str,
        event_protocol: &str,
        issuer_uf: &str,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_insucesso_entrega_request(
            access_key,
            ver_aplic,
            event_protocol,
            issuer_uf,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send_an(SefazService::RecepcaoEvento, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Submit a pedido de prorrogacao ICMS event (`RecepcaoEvento4`,
    /// tpEvento=111500 or 111501).
    ///
    /// Used for NF-e of consignment for industrialization with ICMS suspension
    /// in interstate operations. First term uses 111500, second term uses 111501.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `protocol` — Authorization protocol of the original NF-e.
    /// * `items` — Items and quantities for the prorrogacao request.
    /// * `second_term` — If `true`, sends 2nd-term event (111501).
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn prorrogacao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        protocol: &str,
        items: &[request_builders::ProrrogacaoItem],
        second_term: bool,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_prorrogacao_request(
            access_key,
            protocol,
            items,
            second_term,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// Cancel a pedido de prorrogacao ICMS event (`RecepcaoEvento4`,
    /// tpEvento=111502 or 111503).
    ///
    /// First term uses 111502, second term uses 111503.
    ///
    /// # Arguments
    ///
    /// * `uf` — State abbreviation of the issuer.
    /// * `environment` — SEFAZ environment.
    /// * `access_key` — 44-digit access key of the NF-e.
    /// * `protocol` — Authorization protocol of the prorrogacao event.
    /// * `second_term` — If `true`, sends 2nd-term cancellation (111503).
    /// * `seq` — Event sequence number.
    /// * `tax_id` — CNPJ or CPF of the issuer.
    ///
    /// # Errors
    ///
    /// Returns [`FiscalError::Network`] on transport failure.
    /// Returns `FiscalError::XmlParsing` if the response is malformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn cancel_prorrogacao(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        protocol: &str,
        second_term: bool,
        seq: u32,
        tax_id: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let request_xml = request_builders::build_cancel_prorrogacao_request(
            access_key,
            protocol,
            second_term,
            seq,
            environment,
            tax_id,
        );
        let signed_xml = self.sign_event(&request_xml)?;
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }
}

#[cfg(test)]
mod tests {
    use super::SefazClient;
    use crate::request_builders;
    use fiscal_core::types::SefazEnvironment;

    fn test_pfx() -> Vec<u8> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../..",
            "/tests/fixtures/certs/novo_cert_cnpj_06157250000116_senha_minhasenha.pfx"
        );
        std::fs::read(path).expect("test PFX not found")
    }

    const TEST_PASSWORD: &str = "minhasenha";

    fn build_client() -> SefazClient {
        SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds")
    }

    const TEST_ACCESS_KEY: &str = "41250106157250000116550010000000011000000017";
    const TEST_TAX_ID: &str = "06157250000116";

    // ── ator_interessado (local signing + UF rejection) ──────────────
    // ator_interessado calls send_an which goes to AN (no UF), but the
    // request builder still needs valid inputs.

    #[test]
    fn ator_interessado_builds_and_signs() {
        let client = build_client();
        let request_xml = request_builders::build_ator_interessado_request(
            TEST_ACCESS_KEY,
            2, // tp_autor = destinatario
            "1.0",
            Some("12345678000190"),
            None,
            1,
            "SP",
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("110150"));
        assert!(request_xml.contains(TEST_ACCESS_KEY));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<X509Certificate>"));
    }

    // ── comprovante_entrega (local signing) ──────────────────────────

    #[test]
    fn comprovante_entrega_builds_and_signs() {
        let client = build_client();
        let request_xml = request_builders::build_comprovante_entrega_request(
            TEST_ACCESS_KEY,
            "1.0",
            "2025-01-06T14:00:00-02:00",
            "12345678900",
            "RECIPIENT NAME",
            None, // lat
            None, // long
            "abc123hash",
            "2025-01-06T14:05:00-02:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("110130"));
        assert!(request_xml.contains(TEST_ACCESS_KEY));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
    }

    // ── cancel_comprovante_entrega (local signing) ───────────────────

    #[test]
    fn cancel_comprovante_entrega_builds_and_signs() {
        let client = build_client();
        let request_xml = request_builders::build_cancel_comprovante_entrega_request(
            TEST_ACCESS_KEY,
            "1.0",
            "123456789012345",
            "SP",
            2,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("110131"));
        assert!(request_xml.contains("123456789012345"));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
    }

    // ── insucesso_entrega (local signing) ─────────────────────────────

    #[test]
    fn insucesso_entrega_builds_and_signs() {
        let client = build_client();
        let request_xml = request_builders::build_insucesso_entrega_request(
            TEST_ACCESS_KEY,
            "1.0",
            "2025-01-06T16:00:00-02:00",
            Some(1),
            4, // reason_type=4 requires xJustMotivo
            Some("Cliente ausente no local"),
            None, // lat
            None, // long
            "def456hash",
            "2025-01-06T16:05:00-02:00",
            "SP",
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("110192"));
        assert!(request_xml.contains("Cliente ausente no local"));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
    }

    // ── cancel_insucesso_entrega (local signing) ─────────────────────

    #[test]
    fn cancel_insucesso_entrega_builds_and_signs() {
        let client = build_client();
        let request_xml = request_builders::build_cancel_insucesso_entrega_request(
            TEST_ACCESS_KEY,
            "1.0",
            "123456789012346",
            "SP",
            2,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("110193"));
        assert!(request_xml.contains("123456789012346"));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
    }

    // ── prorrogacao (UF rejection + local signing) ───────────────────

    #[tokio::test]
    async fn prorrogacao_rejects_invalid_uf() {
        let client = build_client();
        let err = client
            .prorrogacao(
                "XX",
                SefazEnvironment::Homologation,
                TEST_ACCESS_KEY,
                "123456789",
                &[],
                false,
                1,
                TEST_TAX_ID,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, fiscal_core::FiscalError::InvalidStateCode(_)));
    }

    #[test]
    fn prorrogacao_builds_and_signs() {
        let client = build_client();
        let items = vec![request_builders::ProrrogacaoItem {
            num_item: 1,
            qtde: 10.0,
        }];
        let request_xml = request_builders::build_prorrogacao_request(
            TEST_ACCESS_KEY,
            "123456789",
            &items,
            false, // first term: tpEvento = 111500
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("111500"));

        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
    }

    #[test]
    fn prorrogacao_second_term_uses_111501() {
        let items = vec![request_builders::ProrrogacaoItem {
            num_item: 1,
            qtde: 10.0,
        }];
        let request_xml = request_builders::build_prorrogacao_request(
            TEST_ACCESS_KEY,
            "123456789",
            &items,
            true, // second term: tpEvento = 111501
            2,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("111501"));
    }

    // ── cancel_prorrogacao (UF rejection + local signing) ─────────────

    #[tokio::test]
    async fn cancel_prorrogacao_rejects_invalid_uf() {
        let client = build_client();
        let err = client
            .cancel_prorrogacao(
                "XX",
                SefazEnvironment::Homologation,
                TEST_ACCESS_KEY,
                "123456789",
                false,
                1,
                TEST_TAX_ID,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, fiscal_core::FiscalError::InvalidStateCode(_)));
    }

    #[test]
    fn cancel_prorrogacao_first_term_uses_111502() {
        let request_xml = request_builders::build_cancel_prorrogacao_request(
            TEST_ACCESS_KEY,
            "123456789",
            false, // cancelling first term
            2,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("111502"));
    }

    #[test]
    fn cancel_prorrogacao_second_term_uses_111503() {
        let request_xml = request_builders::build_cancel_prorrogacao_request(
            TEST_ACCESS_KEY,
            "123456789",
            true, // cancelling second term
            3,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
        );
        assert!(request_xml.contains("111503"));
    }
}
