//! CT-e event request builders (leiaute 4.00).
//!
//! Like the MDF-e — and unlike the NF-e, which wraps events in an `<envEvento>`
//! batch — the `CTeRecepcaoEventoV4` service receives a **bare `<eventoCTe>`**
//! element. The `<infEvento>` is signed in place (`<Signature>` becomes a direct
//! child of `<eventoCTe>`), which the client handles via
//! [`crate::client::SefazClient::cte_recepcao_evento`].
//!
//! Supported events:
//! - **Cancelamento** (`110111`).
//! - **Carta de Correção / CCe** (`110110`).

use fiscal_core::types::SefazEnvironment;

use super::{CTE_NAMESPACE, CTE_VERSION};

/// `tpEvento` — Carta de Correção (CCe).
pub const EV_CCE: u32 = 110110;
/// `tpEvento` — Cancelamento.
pub const EV_CANCELAMENTO: u32 = 110111;
/// `tpEvento` — Prestação do Serviço em Desacordo (manifestado pelo tomador).
pub const EV_PREST_DESACORDO: u32 = 610110;

/// One `<infCorrecao>` group of a CCe: which field changed and the new value.
#[derive(Debug, Clone)]
pub struct CteCorrecao {
    /// Grupo do leiaute alterado (ex: `ide`, `rem`, `vPrest`).
    pub grupo_alterado: String,
    /// Campo alterado dentro do grupo (ex: `xMunIni`).
    pub campo_alterado: String,
    /// Novo valor do campo.
    pub valor_alterado: String,
    /// Número do item alterado (quando o grupo é repetível). Default `1`.
    pub nro_item_alterado: Option<String>,
}

/// Render the issuer tax-id tag (`<CNPJ>` for 14 digits, `<CPF>` for 11).
fn tax_id_tag(tax_id: &str) -> String {
    let digits: String = tax_id.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 11 {
        format!("<CPF>{digits}</CPF>")
    } else {
        format!("<CNPJ>{digits}</CNPJ>")
    }
}

/// Current timestamp with the Brazilian `-03:00` offset, in
/// `AAAA-MM-DDThh:mm:ss-03:00` form (required by `TDateTimeUTC`).
fn now_brt() -> String {
    chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::west_opt(3 * 3600).unwrap())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

/// Assemble a complete (unsigned) `<eventoCTe>` from a pre-rendered
/// `detEvento` inner element (e.g. `<evCancCTe>…</evCancCTe>`).
///
/// `c_orgao` is derived from the first two digits of `ch_cte`. The `Id` is
/// `ID{tpEvento}{chCTe}{seq:03}` — note the CT-e pads `nSeqEvento` to **3**
/// digits in the `Id` (pattern `ID[0-9]{53}`), unlike NF-e/MDF-e which use 2.
fn build_evento(
    ch_cte: &str,
    tp_evento: u32,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
    det_inner: &str,
) -> String {
    assert!(
        ch_cte.len() == 44 && ch_cte.bytes().all(|b| b.is_ascii_digit()),
        "CT-e access key must be exactly 44 digits"
    );
    let id = format!("ID{tp_evento}{ch_cte}{seq:03}");
    let c_orgao = &ch_cte[..2];
    let tp_amb = environment.as_str();
    let tax_tag = tax_id_tag(tax_id);
    let dh_evento = now_brt();

    format!(
        "<eventoCTe xmlns=\"{CTE_NAMESPACE}\" versao=\"{CTE_VERSION}\">\
<infEvento Id=\"{id}\">\
<cOrgao>{c_orgao}</cOrgao>\
<tpAmb>{tp_amb}</tpAmb>\
{tax_tag}\
<chCTe>{ch_cte}</chCTe>\
<dhEvento>{dh_evento}</dhEvento>\
<tpEvento>{tp_evento}</tpEvento>\
<nSeqEvento>{seq}</nSeqEvento>\
<detEvento versaoEvento=\"{CTE_VERSION}\">{det_inner}</detEvento>\
</infEvento></eventoCTe>"
    )
}

/// Build a **Cancelamento** (`110111`) event for an authorized CT-e.
///
/// # Panics
///
/// Panics if `ch_cte` is not 44 digits, or `justification` is outside 15–255
/// characters (SEFAZ `xJust` constraint).
pub fn build_cte_cancelamento(
    ch_cte: &str,
    protocol: &str,
    justification: &str,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    let len = justification.chars().count();
    assert!(
        (15..=255).contains(&len),
        "cancellation justification (xJust) must be 15–255 chars, got {len}"
    );
    let det = format!(
        "<evCancCTe><descEvento>Cancelamento</descEvento><nProt>{protocol}</nProt><xJust>{justification}</xJust></evCancCTe>"
    );
    build_evento(ch_cte, EV_CANCELAMENTO, seq, tax_id, environment, &det)
}

/// Build a **Carta de Correção** (`110110`) event for an authorized CT-e.
///
/// At least one [`CteCorrecao`] is required. The mandatory `xCondUso` legal text
/// is appended automatically.
///
/// # Panics
///
/// Panics if `ch_cte` is not 44 digits or `correcoes` is empty.
pub fn build_cte_cce(
    ch_cte: &str,
    correcoes: &[CteCorrecao],
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    assert!(
        !correcoes.is_empty(),
        "CCe requires at least one infCorrecao group"
    );
    let x_cond_uso = concat!(
        "A Carta de Correcao e disciplinada pelo paragrafo ",
        "1o-A do art. 7o do Convenio S/N, de 15 de dezembro de 1970 ",
        "e pode ser utilizada para regularizacao de erro ocorrido ",
        "na emissao de documento fiscal, desde que o erro nao esteja ",
        "relacionado com: I - as variaveis que determinam o valor ",
        "do imposto tais como: base de calculo, aliquota, ",
        "diferenca de preco, quantidade, valor da operacao ou da ",
        "prestacao; II - a correcao de dados cadastrais que implique ",
        "mudanca do remetente ou do destinatario; III - a data de ",
        "emissao ou de saida."
    );

    let mut grupos = String::new();
    for c in correcoes {
        let nro = c.nro_item_alterado.as_deref().unwrap_or("1");
        grupos.push_str(&format!(
            "<infCorrecao><grupoAlterado>{}</grupoAlterado><campoAlterado>{}</campoAlterado><valorAlterado>{}</valorAlterado><nroItemAlterado>{nro}</nroItemAlterado></infCorrecao>",
            c.grupo_alterado, c.campo_alterado, c.valor_alterado
        ));
    }

    let det = format!(
        "<evCCeCTe><descEvento>Carta de Correcao</descEvento>{grupos}<xCondUso>{x_cond_uso}</xCondUso></evCCeCTe>"
    );
    build_evento(ch_cte, EV_CCE, seq, tax_id, environment, &det)
}

/// Build a **Prestação do Serviço em Desacordo** (`610110`) event.
///
/// Manifestado pelo **tomador** do serviço (assinado com o certificado do
/// tomador). `x_obs` descreve o desacordo.
///
/// # Panics
///
/// Panics if `ch_cte` is not 44 digits.
pub fn build_cte_desacordo(
    ch_cte: &str,
    x_obs: &str,
    seq: u32,
    tax_id: &str,
    environment: SefazEnvironment,
) -> String {
    let det = format!(
        "<evPrestDesacordo><descEvento>Prestacao do Servico em Desacordo</descEvento><indDesacordoOper>1</indDesacordoOper><xObs>{x_obs}</xObs></evPrestDesacordo>"
    );
    build_evento(ch_cte, EV_PREST_DESACORDO, seq, tax_id, environment, &det)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chave() -> String {
        "35".to_string() + &"1".repeat(42)
    }

    #[test]
    fn cancelamento_tem_estrutura_correta() {
        let xml = build_cte_cancelamento(
            &chave(),
            "135200000000001",
            "Cancelamento por erro de digitacao no valor",
            1,
            "12345678000199",
            SefazEnvironment::Homologation,
        );
        assert!(xml.starts_with("<eventoCTe"));
        assert!(xml.contains("<tpEvento>110111</tpEvento>"));
        assert!(xml.contains("<evCancCTe>"));
        assert!(xml.contains("<nProt>135200000000001</nProt>"));
        assert!(xml.contains(&format!("Id=\"ID110111{}001\"", chave())));
        // CT-e Id pads nSeqEvento to 3 digits → total 53 digits after "ID".
        let id_digits = chave().len() + 6 + 3;
        assert_eq!(id_digits, 53);
        assert!(xml.contains("<CNPJ>12345678000199</CNPJ>"));
    }

    #[test]
    fn cce_monta_grupos_e_cond_uso() {
        let cors = vec![CteCorrecao {
            grupo_alterado: "ide".into(),
            campo_alterado: "xMunIni".into(),
            valor_alterado: "SAO PAULO".into(),
            nro_item_alterado: None,
        }];
        let xml = build_cte_cce(
            &chave(),
            &cors,
            2,
            "12345678000199",
            SefazEnvironment::Production,
        );
        assert!(xml.contains("<tpEvento>110110</tpEvento>"));
        assert!(xml.contains("<evCCeCTe>"));
        assert!(xml.contains("<grupoAlterado>ide</grupoAlterado>"));
        assert!(xml.contains("<campoAlterado>xMunIni</campoAlterado>"));
        assert!(xml.contains("<nroItemAlterado>1</nroItemAlterado>"));
        assert!(xml.contains("<xCondUso>"));
        assert!(xml.contains("<nSeqEvento>2</nSeqEvento>"));
    }

    #[test]
    fn desacordo_tem_estrutura() {
        let xml = build_cte_desacordo(
            &chave(),
            "Servico nao prestado conforme contratado",
            1,
            "12345678000199",
            SefazEnvironment::Homologation,
        );
        assert!(xml.contains("<tpEvento>610110</tpEvento>"));
        assert!(xml.contains("<evPrestDesacordo>"));
        assert!(xml.contains("<indDesacordoOper>1</indDesacordoOper>"));
    }

    #[test]
    #[should_panic(expected = "xJust")]
    fn cancelamento_rejeita_justificativa_curta() {
        build_cte_cancelamento(
            &chave(),
            "135",
            "curto",
            1,
            "12345678000199",
            SefazEnvironment::Homologation,
        );
    }
}
