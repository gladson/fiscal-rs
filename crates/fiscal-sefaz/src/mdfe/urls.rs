//! MDF-e endpoint URL resolution.
//!
//! Unlike NF-e — where each UF may run its own authorizer — **every** Brazilian
//! state authorizes MDF-e through **SVRS** (Sefaz Virtual do RS). The ACBr
//! `ACBrMDFeServicos.ini` config makes all 27 UF sections a plain
//! `Usar=MDFe_SVRS` redirect, so there are no per-state exceptions.
//!
//! The path casing below is reproduced **exactly** from the SVRS config —
//! the load balancer is case-sensitive on path segments (note that
//! `MDFeRecepcaoSinc` keeps mixed case while the others are lowercase).

use fiscal_core::FiscalError;
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::MdfeService;

/// Production host for SVRS MDF-e services.
const HOST_PROD: &str = "https://mdfe.svrs.rs.gov.br";
/// Homologation host for SVRS MDF-e services.
const HOST_HOMOLOG: &str = "https://mdfe-homologacao.svrs.rs.gov.br";

/// Path segment (after the host) for each service. Casing is significant.
fn service_path(service: MdfeService) -> &'static str {
    match service {
        MdfeService::StatusServico => "/ws/mdfestatusservico/MDFeStatusServico.asmx",
        MdfeService::Consulta => "/ws/mdfeconsulta/MDFeConsulta.asmx",
        MdfeService::RecepcaoSinc => "/ws/MDFeRecepcaoSinc/MDFeRecepcaoSinc.asmx",
        MdfeService::Recepcao => "/ws/mdferecepcao/MDFeRecepcao.asmx",
        MdfeService::RetRecepcao => "/ws/mdferetrecepcao/MDFeRetRecepcao.asmx",
        MdfeService::RecepcaoEvento => "/ws/mdferecepcaoevento/MDFeRecepcaoEvento.asmx",
        MdfeService::ConsNaoEnc => "/ws/mdfeconsnaoenc/MDFeConsNaoEnc.asmx",
    }
}

/// Resolve the SVRS MDF-e service URL for a state and environment.
///
/// `uf` is validated as a real Brazilian state abbreviation (the authorizer is
/// always SVRS regardless of which state it is), so callers get an early,
/// uniform error for typos.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation, or if `environment` is a production-contingency variant
/// that SVRS MDF-e does not serve.
pub fn get_mdfe_url(
    uf: &str,
    environment: SefazEnvironment,
    service: MdfeService,
) -> Result<String, FiscalError> {
    // Validate the UF even though SVRS serves them all — keeps the error
    // surface identical to the NF-e path and rejects garbage early.
    get_state_code(uf)?;

    let host = match environment {
        SefazEnvironment::Production => HOST_PROD,
        SefazEnvironment::Homologation => HOST_HOMOLOG,
        _ => return Err(FiscalError::InvalidStateCode(uf.to_string())),
    };

    Ok(format!("{host}{}", service_path(service)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_url_production() {
        let url = get_mdfe_url(
            "SP",
            SefazEnvironment::Production,
            MdfeService::StatusServico,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://mdfe.svrs.rs.gov.br/ws/mdfestatusservico/MDFeStatusServico.asmx"
        );
    }

    #[test]
    fn status_url_homologation() {
        let url = get_mdfe_url(
            "RS",
            SefazEnvironment::Homologation,
            MdfeService::StatusServico,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://mdfe-homologacao.svrs.rs.gov.br/ws/mdfestatusservico/MDFeStatusServico.asmx"
        );
    }

    #[test]
    fn recepcao_sinc_keeps_mixed_case_path() {
        let url = get_mdfe_url(
            "MG",
            SefazEnvironment::Production,
            MdfeService::RecepcaoSinc,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://mdfe.svrs.rs.gov.br/ws/MDFeRecepcaoSinc/MDFeRecepcaoSinc.asmx"
        );
    }

    #[test]
    fn every_uf_resolves_to_svrs() {
        // All 27 states authorize through the same SVRS host.
        for uf in [
            "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MG", "MS", "MT", "PA",
            "PB", "PE", "PI", "PR", "RJ", "RN", "RO", "RS", "RR", "SC", "SE", "SP", "TO",
        ] {
            let url =
                get_mdfe_url(uf, SefazEnvironment::Production, MdfeService::Consulta).unwrap();
            assert!(
                url.starts_with("https://mdfe.svrs.rs.gov.br/"),
                "{uf} did not resolve to SVRS: {url}"
            );
        }
    }

    #[test]
    fn rejects_invalid_uf() {
        let err = get_mdfe_url(
            "XX",
            SefazEnvironment::Production,
            MdfeService::StatusServico,
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }
}
