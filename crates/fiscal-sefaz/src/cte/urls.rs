//! CT-e endpoint URL resolution (leiaute 4.00).
//!
//! Unlike MDF-e (single SVRS authorizer), CT-e has three kinds of authorizer,
//! grounded in `ACBrCTeServicos.ini` / `sped-cte wscte_4.00_mod57.xml`:
//!
//! - **Own authorizers**: MG, MS, MT, PR, SP.
//! - **SVSP** (Sefaz Virtual de SP) for AP, PE, RR → routed to the SP host.
//! - **SVRS** for every other UF (the majority).

use fiscal_core::FiscalError;
use fiscal_core::state_codes::get_state_code;
use fiscal_core::types::SefazEnvironment;

use super::CteService;

/// Production + homologation URLs for one CT-e service.
struct ServiceUrls {
    production: &'static str,
    homologation: &'static str,
}

/// The four CT-e services for a single authorizer.
struct CteAuthorizer {
    status: ServiceUrls,
    consulta: ServiceUrls,
    recepcao_sinc: ServiceUrls,
    recepcao_evento: ServiceUrls,
}

impl CteAuthorizer {
    fn url(&self, service: CteService, env: SefazEnvironment) -> Option<&'static str> {
        let urls = match service {
            CteService::StatusServico => &self.status,
            CteService::Consulta => &self.consulta,
            CteService::RecepcaoSinc => &self.recepcao_sinc,
            // GTV-e compartilha o host do RecepcaoSinc; o client troca o path
            // `CTeRecepcaoSincV4` → `CTeRecepcaoGTVe`.
            CteService::RecepcaoGTVe => &self.recepcao_sinc,
            CteService::RecepcaoEvento => &self.recepcao_evento,
        };
        match env {
            SefazEnvironment::Production => Some(urls.production),
            SefazEnvironment::Homologation => Some(urls.homologation),
            _ => None,
        }
    }
}

macro_rules! authorizer {
    ($st_p:literal, $st_h:literal, $co_p:literal, $co_h:literal,
     $rs_p:literal, $rs_h:literal, $ev_p:literal, $ev_h:literal) => {
        CteAuthorizer {
            status: ServiceUrls {
                production: $st_p,
                homologation: $st_h,
            },
            consulta: ServiceUrls {
                production: $co_p,
                homologation: $co_h,
            },
            recepcao_sinc: ServiceUrls {
                production: $rs_p,
                homologation: $rs_h,
            },
            recepcao_evento: ServiceUrls {
                production: $ev_p,
                homologation: $ev_h,
            },
        }
    };
}

static SVRS: CteAuthorizer = authorizer!(
    "https://cte.svrs.rs.gov.br/ws/CTeStatusServicoV4/CTeStatusServicoV4.asmx",
    "https://cte-homologacao.svrs.rs.gov.br/ws/CTeStatusServicoV4/CTeStatusServicoV4.asmx",
    "https://cte.svrs.rs.gov.br/ws/CTeConsultaV4/CTeConsultaV4.asmx",
    "https://cte-homologacao.svrs.rs.gov.br/ws/CTeConsultaV4/CTeConsultaV4.asmx",
    "https://cte.svrs.rs.gov.br/ws/CTeRecepcaoSincV4/CTeRecepcaoSincV4.asmx",
    "https://cte-homologacao.svrs.rs.gov.br/ws/CTeRecepcaoSincV4/CTeRecepcaoSincV4.asmx",
    "https://cte.svrs.rs.gov.br/ws/CTeRecepcaoEventoV4/CTeRecepcaoEventoV4.asmx",
    "https://cte-homologacao.svrs.rs.gov.br/ws/CTeRecepcaoEventoV4/CTeRecepcaoEventoV4.asmx"
);

/// SP own authorizer — also serves AP, PE, RR as **SVSP**.
static SP: CteAuthorizer = authorizer!(
    "https://nfe.fazenda.sp.gov.br/CTeWS/WS/CTeStatusServicoV4.asmx",
    "https://homologacao.nfe.fazenda.sp.gov.br/CTeWS/WS/CTeStatusServicoV4.asmx",
    "https://nfe.fazenda.sp.gov.br/CTeWS/WS/CTeConsultaV4.asmx",
    "https://homologacao.nfe.fazenda.sp.gov.br/CTeWS/WS/CTeConsultaV4.asmx",
    "https://nfe.fazenda.sp.gov.br/CTeWS/WS/CTeRecepcaoSincV4.asmx",
    "https://homologacao.nfe.fazenda.sp.gov.br/CTeWS/WS/CTeRecepcaoSincV4.asmx",
    "https://nfe.fazenda.sp.gov.br/CTeWS/WS/CTeRecepcaoEventoV4.asmx",
    "https://homologacao.nfe.fazenda.sp.gov.br/CTeWS/WS/CTeRecepcaoEventoV4.asmx"
);

static MG: CteAuthorizer = authorizer!(
    "https://cte.fazenda.mg.gov.br/cte/services/CTeStatusServicoV4",
    "https://hcte.fazenda.mg.gov.br/cte/services/CTeStatusServicoV4",
    "https://cte.fazenda.mg.gov.br/cte/services/CTeConsultaV4",
    "https://hcte.fazenda.mg.gov.br/cte/services/CTeConsultaV4",
    "https://cte.fazenda.mg.gov.br/cte/services/CTeRecepcaoSincV4",
    "https://hcte.fazenda.mg.gov.br/cte/services/CTeRecepcaoSincV4",
    "https://cte.fazenda.mg.gov.br/cte/services/CTeRecepcaoEventoV4",
    "https://hcte.fazenda.mg.gov.br/cte/services/CTeRecepcaoEventoV4"
);

static MS: CteAuthorizer = authorizer!(
    "https://producao.cte.ms.gov.br/ws/CTeStatusServicoV4",
    "https://homologacao.cte.ms.gov.br/ws/CTeStatusServicoV4",
    "https://producao.cte.ms.gov.br/ws/CTeConsultaV4",
    "https://homologacao.cte.ms.gov.br/ws/CTeConsultaV4",
    "https://producao.cte.ms.gov.br/ws/CTeRecepcaoSincV4",
    "https://homologacao.cte.ms.gov.br/ws/CTeRecepcaoSincV4",
    "https://producao.cte.ms.gov.br/ws/CTeRecepcaoEventoV4",
    "https://homologacao.cte.ms.gov.br/ws/CTeRecepcaoEventoV4"
);

static MT: CteAuthorizer = authorizer!(
    "https://cte.sefaz.mt.gov.br/ctews2/services/CTeStatusServicoV4",
    "https://homologacao.sefaz.mt.gov.br/ctews2/services/CTeStatusServicoV4",
    "https://cte.sefaz.mt.gov.br/ctews2/services/CTeConsultaV4",
    "https://homologacao.sefaz.mt.gov.br/ctews2/services/CTeConsultaV4",
    "https://cte.sefaz.mt.gov.br/ctews2/services/CTeRecepcaoSincV4",
    "https://homologacao.sefaz.mt.gov.br/ctews2/services/CTeRecepcaoSincV4",
    "https://cte.sefaz.mt.gov.br/ctews2/services/CTeRecepcaoEventoV4",
    "https://homologacao.sefaz.mt.gov.br/ctews2/services/CTeRecepcaoEventoV4"
);

static PR: CteAuthorizer = authorizer!(
    "https://cte.fazenda.pr.gov.br/cte4/CTeStatusServicoV4",
    "https://homologacao.cte.fazenda.pr.gov.br/cte4/CTeStatusServicoV4",
    "https://cte.fazenda.pr.gov.br/cte4/CTeConsultaV4",
    "https://homologacao.cte.fazenda.pr.gov.br/cte4/CTeConsultaV4",
    "https://cte.fazenda.pr.gov.br/cte4/CTeRecepcaoSincV4",
    "https://homologacao.cte.fazenda.pr.gov.br/cte4/CTeRecepcaoSincV4",
    "https://cte.fazenda.pr.gov.br/cte4/CTeRecepcaoEventoV4",
    "https://homologacao.cte.fazenda.pr.gov.br/cte4/CTeRecepcaoEventoV4"
);

/// Resolve the CT-e authorizer for a given UF.
fn get_cte_authorizer(uf: &str) -> Option<&'static CteAuthorizer> {
    match uf {
        "MG" => Some(&MG),
        "MS" => Some(&MS),
        "MT" => Some(&MT),
        "PR" => Some(&PR),
        "SP" => Some(&SP),
        // SVSP (Sefaz Virtual de SP) — routed to the SP host.
        "AP" | "PE" | "RR" => Some(&SP),
        // SVRS — every other valid UF.
        "AC" | "AL" | "AM" | "BA" | "CE" | "DF" | "ES" | "GO" | "MA" | "PA" | "PB" | "PI"
        | "RJ" | "RN" | "RO" | "RS" | "SC" | "SE" | "TO" => Some(&SVRS),
        _ => None,
    }
}

/// Resolve the CT-e service URL for a UF, environment, and service.
///
/// # Errors
///
/// Returns [`FiscalError::InvalidStateCode`] if `uf` is not a valid Brazilian
/// state abbreviation, or if `environment` is a contingency variant that the
/// CT-e authorizers do not serve.
pub fn get_cte_url(
    uf: &str,
    environment: SefazEnvironment,
    service: CteService,
) -> Result<String, FiscalError> {
    // Validate the UF for a uniform early error on typos.
    get_state_code(uf)?;

    let authorizer =
        get_cte_authorizer(uf).ok_or_else(|| FiscalError::InvalidStateCode(uf.to_string()))?;

    authorizer
        .url(service, environment)
        .map(|s| s.to_string())
        .ok_or_else(|| FiscalError::InvalidStateCode(uf.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svrs_uf_resolves_to_svrs_host() {
        let url = get_cte_url(
            "RS",
            SefazEnvironment::Homologation,
            CteService::RecepcaoSinc,
        )
        .unwrap();
        assert_eq!(
            url,
            "https://cte-homologacao.svrs.rs.gov.br/ws/CTeRecepcaoSincV4/CTeRecepcaoSincV4.asmx"
        );
    }

    #[test]
    fn svsp_ufs_route_to_sp_host() {
        for uf in ["AP", "PE", "RR"] {
            let url = get_cte_url(uf, SefazEnvironment::Production, CteService::Consulta).unwrap();
            assert!(
                url.starts_with("https://nfe.fazenda.sp.gov.br/CTeWS/"),
                "{uf} should route to SP (SVSP): {url}"
            );
        }
    }

    #[test]
    fn own_authorizers_use_their_own_host() {
        let mt = get_cte_url(
            "MT",
            SefazEnvironment::Homologation,
            CteService::StatusServico,
        )
        .unwrap();
        assert!(mt.contains("homologacao.sefaz.mt.gov.br"));
        let pr = get_cte_url(
            "PR",
            SefazEnvironment::Production,
            CteService::RecepcaoEvento,
        )
        .unwrap();
        assert!(pr.contains("cte.fazenda.pr.gov.br/cte4"));
    }

    #[test]
    fn rejects_invalid_uf() {
        let err = get_cte_url(
            "XX",
            SefazEnvironment::Production,
            CteService::StatusServico,
        )
        .unwrap_err();
        assert!(matches!(err, FiscalError::InvalidStateCode(_)));
    }
}
