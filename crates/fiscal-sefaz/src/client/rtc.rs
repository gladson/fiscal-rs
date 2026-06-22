//! RTC (Reforma Tributária do Consumo) event methods.

use fiscal_core::FiscalError;
use fiscal_core::types::SefazEnvironment;

use crate::request_builders::{self, RtcCredPresItem, RtcItem};
use crate::response_parsers::{self, CancellationResponse};
use crate::services::SefazService;

use super::{SefazClient, svrs_org_override};

impl SefazClient {
    // ── RTC (Reforma Tributaria) typed convenience methods ──────────────

    /// Send an RTC event via SVRS RecepcaoEvento.
    ///
    /// The built `<infEvento>` is signed before transmit — SEFAZ rejects
    /// unsigned events.
    async fn send_rtc_event(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        request_xml: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let signed_xml = self.sign_event(request_xml)?;
        let raw = self
            .send(SefazService::RecepcaoEvento, uf, environment, &signed_xml)
            .await?;
        response_parsers::parse_cancellation_response(&raw)
    }

    /// RTC: Informacao de pagamento integral (tpEvento=112110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_info_pagto_integral(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_info_pagto_integral(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Aceite de debito na apuracao (tpEvento=211128).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_aceite_debito(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_aceite_debito(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Manifestacao transferencia credito IBS (tpEvento=212110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_manif_transf_cred_ibs(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_manif_transf_cred_ibs(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Manifestacao transferencia credito CBS (tpEvento=212120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_manif_transf_cred_cbs(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        ind_aceitacao: u8,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_manif_transf_cred_cbs(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            ind_aceitacao,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Cancelamento de evento (tpEvento=110001).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_cancela_evento(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        tp_evento_aut: &str,
        n_prot_evento: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_cancela_evento(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            tp_evento_aut,
            n_prot_evento,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Atualizacao da data de previsao de entrega (tpEvento=112150).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_atualizacao_data_entrega(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        data_prevista: &str,
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_atualizacao_data_entrega(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            data_prevista,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Importacao via ZFM (tpEvento=112120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_importacao_zfm(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_importacao_zfm(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Roubo/perda em transporte pelo fornecedor (tpEvento=112130).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_roubo_perda_fornecedor(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_roubo_perda_fornecedor(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Fornecimento nao realizado (tpEvento=112140).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_fornecimento_nao_realizado(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_fornecimento_nao_realizado(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Solicitacao de apropriacao de credito presumido (tpEvento=211110).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_sol_aprop_cred_presumido(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcCredPresItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_sol_aprop_cred_presumido(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Destinacao de item para consumo pessoal (tpEvento=211120).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_destino_consumo_pessoal(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        tp_autor: u8,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_destino_consumo_pessoal(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            tp_autor,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Perecimento/roubo transporte adquirente (tpEvento=211124).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_roubo_perda_adquirente(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_roubo_perda_adquirente(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Imobilizacao de item (tpEvento=211130).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_imobilizacao_item(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_imobilizacao_item(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Apropriacao de credito combustivel (tpEvento=211140).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_apropriacao_credito_comb(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_apropriacao_credito_comb(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
    }

    /// RTC: Apropriacao de credito bens/servicos (tpEvento=211150).
    #[allow(clippy::too_many_arguments)]
    pub async fn rtc_apropriacao_credito_bens(
        &self,
        uf: &str,
        environment: SefazEnvironment,
        access_key: &str,
        seq: u32,
        tax_id: &str,
        ver_aplic: &str,
        itens: &[RtcItem],
    ) -> Result<CancellationResponse, FiscalError> {
        let org = svrs_org_override(uf);
        let xml = request_builders::build_rtc_apropriacao_credito_bens(
            access_key,
            seq,
            environment,
            tax_id,
            uf,
            ver_aplic,
            itens,
            org,
        );
        self.send_rtc_event(uf, environment, &xml).await
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
    const TEST_ACCESS_KEY: &str = "41250106157250000116550010000000011000000017";
    const TEST_TAX_ID: &str = "06157250000116";

    fn build_client() -> SefazClient {
        SefazClient::new(&test_pfx(), TEST_PASSWORD).expect("client builds")
    }

    fn sample_rtc_item() -> request_builders::RtcItem {
        request_builders::RtcItem::new(1, 10.0, 5.0)
    }

    fn sample_cred_pres_item() -> request_builders::RtcCredPresItem {
        request_builders::RtcCredPresItem {
            item: 1,
            v_bc: 100.0,
            g_ibs: Some(request_builders::RtcCredPresSub {
                c_cred_pres: "1001".into(),
                p_cred_pres: 5.0,
                v_cred_pres: 5.0,
            }),
            g_cbs: None,
        }
    }

    // ── rtc_info_pagto_integral ──────────────────────────────────────

    #[test]
    fn rtc_info_pagto_integral_builds_event_with_type_112110() {
        let request_xml = request_builders::build_rtc_info_pagto_integral(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("112110"));
        assert!(request_xml.contains(TEST_ACCESS_KEY));
    }

    // ── rtc_aceite_debito ─────────────────────────────────────────────

    #[test]
    fn rtc_aceite_debito_builds_event_with_type_211128() {
        let request_xml = request_builders::build_rtc_aceite_debito(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            1,
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211128"));
        assert!(request_xml.contains("<indAceitacao>1</indAceitacao>"));
    }

    // ── rtc_manif_transf_cred_ibs ─────────────────────────────────────

    #[test]
    fn rtc_manif_transf_cred_ibs_builds_event() {
        let request_xml = request_builders::build_rtc_manif_transf_cred_ibs(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            1,
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("212110"));
    }

    // ── rtc_manif_transf_cred_cbs ─────────────────────────────────────

    #[test]
    fn rtc_manif_transf_cred_cbs_builds_event() {
        let request_xml = request_builders::build_rtc_manif_transf_cred_cbs(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            1,
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("212120"));
    }

    // ── rtc_cancela_evento ────────────────────────────────────────────

    #[test]
    fn rtc_cancela_evento_builds_event_with_type_110001() {
        let request_xml = request_builders::build_rtc_cancela_evento(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            "112110",
            "123456789012345",
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("110001"));
        assert!(request_xml.contains("123456789012345"));
    }

    // ── rtc_atualizacao_data_entrega ──────────────────────────────────

    #[test]
    fn rtc_atualizacao_data_entrega_builds_event() {
        let request_xml = request_builders::build_rtc_atualizacao_data_entrega(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            "2025-12-31",
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("112150"));
        assert!(request_xml.contains("2025-12-31"));
    }

    // ── rtc_importacao_zfm ────────────────────────────────────────────

    #[test]
    fn rtc_importacao_zfm_builds_event_with_items() {
        let items = vec![
            request_builders::RtcItem::new(1, 10.0, 5.0),
            request_builders::RtcItem::new(2, 20.0, 10.0),
        ];
        let request_xml = request_builders::build_rtc_importacao_zfm(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &items,
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("112120"));
        assert!(request_xml.contains("10.0")); // vIBS
    }

    // ── rtc_roubo_perda_fornecedor ────────────────────────────────────

    #[test]
    fn rtc_roubo_perda_fornecedor_builds_event() {
        let request_xml = request_builders::build_rtc_roubo_perda_fornecedor(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("112130"));
    }

    // ── rtc_fornecimento_nao_realizado ────────────────────────────────

    #[test]
    fn rtc_fornecimento_nao_realizado_builds_event() {
        let request_xml = request_builders::build_rtc_fornecimento_nao_realizado(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("112140"));
    }

    // ── rtc_sol_aprop_cred_presumido ──────────────────────────────────

    #[test]
    fn rtc_sol_aprop_cred_presumido_builds_event() {
        let request_xml = request_builders::build_rtc_sol_aprop_cred_presumido(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_cred_pres_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211110"));
        assert!(request_xml.contains("1001")); // cCredPres
    }

    // ── rtc_destino_consumo_pessoal ───────────────────────────────────

    #[test]
    fn rtc_destino_consumo_pessoal_builds_event() {
        let request_xml = request_builders::build_rtc_destino_consumo_pessoal(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            1, // tp_autor
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211120"));
        assert!(request_xml.contains("<tpAutor>1</tpAutor>"));
    }

    // ── rtc_roubo_perda_adquirente ────────────────────────────────────

    #[test]
    fn rtc_roubo_perda_adquirente_builds_event() {
        let request_xml = request_builders::build_rtc_roubo_perda_adquirente(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211124"));
    }

    // ── rtc_imobilizacao_item ─────────────────────────────────────────

    #[test]
    fn rtc_imobilizacao_item_builds_event() {
        let request_xml = request_builders::build_rtc_imobilizacao_item(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211130"));
    }

    // ── rtc_apropriacao_credito_comb ──────────────────────────────────

    #[test]
    fn rtc_apropriacao_credito_comb_builds_event() {
        let request_xml = request_builders::build_rtc_apropriacao_credito_comb(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211140"));
    }

    // ── rtc_apropriacao_credito_bens ──────────────────────────────────

    #[test]
    fn rtc_apropriacao_credito_bens_builds_event() {
        let request_xml = request_builders::build_rtc_apropriacao_credito_bens(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            &[sample_rtc_item()],
            super::svrs_org_override("SP"),
        );
        assert!(request_xml.contains("211150"));
    }

    // ── send_rtc_event signs before transmit ──────────────────────────

    #[test]
    fn rtc_events_are_signed_locally() {
        let client = build_client();
        let request_xml = request_builders::build_rtc_info_pagto_integral(
            TEST_ACCESS_KEY,
            1,
            SefazEnvironment::Homologation,
            TEST_TAX_ID,
            "SP",
            "1.0",
            super::svrs_org_override("SP"),
        );
        let signed = client.sign_event(&request_xml).expect("signs");
        assert!(signed.contains("<Signature"));
        assert!(signed.contains("<X509Certificate>"));
    }

    // ── svrs_org_override returns cOrgao=92 for SVRS ──────────────────

    #[test]
    fn svrs_org_override_returns_92_for_svrs() {
        assert_eq!(super::svrs_org_override("SVRS"), Some("92"));
    }

    #[test]
    fn svrs_org_override_returns_none_for_regular_uf() {
        assert_eq!(super::svrs_org_override("SP"), None);
        assert_eq!(super::svrs_org_override("RJ"), None);
    }
}
