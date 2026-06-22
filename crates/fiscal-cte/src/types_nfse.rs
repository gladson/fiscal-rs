//! Types for the NFS-e Nacional DPS (Declaração de Prestação de Serviço),
//! leiaute **1.01** (RTC — com grupo IBS/CBS da reforma).
//!
//! Documento separado (namespace `http://www.sped.fazenda.gov.br/nfse`),
//! REST (não SOAP). O emitente monta a `<DPS>`, assina `<infDPS>`, comprime e
//! envia ao SEFIN Nacional, que devolve a NFS-e (chave de 50 dígitos).
//!
//! Reusa `Documento` de [`crate::types`].

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::types::Documento;

/// Root build data for a DPS document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct DpsBuildData {
    pub ide: IdeDps,
    pub prest: Prestador,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toma: Option<Pessoa>,
    pub serv: Servico,
    pub valores: Valores,
    /// Grupo `IBSCBS` da reforma tributária (irmão de `valores` em `infDPS`).
    /// Opcional na transição; obrigatório quando o RTC entra em vigor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibscbs: Option<Ibscbs>,
}

/// `<infDPS>` identification block. O `Id` (`DPS` + 42 dígitos) é derivado de
/// cLocEmi + tpInsc + inscrição + serie + nDPS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct IdeDps {
    pub tp_amb: String,
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    #[serde(default = "ver_aplic_default")]
    pub ver_aplic: String,
    pub serie: String,
    pub n_dps: u64,
    /// `dCompet` — competência (AAAA-MM-DD).
    pub d_compet: String,
    /// `tpEmit` — `1` Prestador, `2` Tomador, `3` Intermediário.
    pub tp_emit: String,
    /// `cLocEmi` — código IBGE do município emitente (7 dígitos).
    pub c_loc_emi: String,
}

fn ver_aplic_default() -> String {
    "dfehub-1.0".into()
}

/// `<prest>` — prestador do serviço.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Prestador {
    pub doc: Documento,
    /// `IM` — inscrição municipal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<EnderNac>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub reg_trib: RegTrib,
}

/// `<regTrib>` — regime tributário do prestador.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct RegTrib {
    /// `opSimpNac` — `1` não optante, `2` MEI, `3` ME/EPP.
    pub op_simp_nac: String,
    /// `regEspTrib` — regime especial de tributação (`0` nenhum, ...).
    pub reg_esp_trib: String,
}

/// `<toma>`/`<interm>` — pessoa (tomador/intermediário).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Pessoa {
    pub doc: Documento,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    pub x_nome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<EnderNac>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// `<endNac>` — endereço nacional.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct EnderNac {
    pub x_lgr: String,
    pub nro: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_cpl: Option<String>,
    pub x_bairro: String,
    /// `cMun` — código IBGE do município (7 dígitos).
    pub c_mun: String,
    pub cep: String,
}

/// `<serv>` — dados do serviço prestado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Servico {
    /// `cLocPrestacao` — município de prestação (IBGE).
    pub c_loc_prestacao: String,
    /// `cTribNac` — código de tributação nacional (lista nacional de serviços).
    pub c_trib_nac: String,
    /// `cTribMun` — código de tributação municipal (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_trib_mun: Option<String>,
    pub x_desc_serv: String,
}

/// `<valores>` — valores e tributos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Valores {
    /// `vServ` — valor do serviço.
    pub v_serv: String,
    pub trib: Trib,
}

/// `<trib>` — tributação (ISSQN municipal + federal). O grupo IBS/CBS fica
/// fora de `trib` (é irmão de `valores` em `infDPS` — ver [`DpsBuildData::ibscbs`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Trib {
    pub trib_mun: TribMun,
    /// `tribFed` — PIS/COFINS/IR/CSLL (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trib_fed: Option<TribFed>,
}

/// `<tribMun>` — ISSQN.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct TribMun {
    /// `tribISSQN` — `1` operação tributável, `2` imunidade, `3` exportação,
    /// `4` não incidência.
    pub trib_issqn: String,
    /// `pAliq` — alíquota ISSQN (%). Obrigatória quando tribISSQN=1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq: Option<String>,
    /// `tpRetISSQN` — `1` não retido, `2` retido pelo tomador, `3` intermediário.
    pub tp_ret_issqn: String,
}

/// `<tribFed>` — tributos federais (PIS/COFINS + retenções CP/IRRF/CSLL).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct TribFed {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub piscofins: Option<PisCofins>,
    /// `vRetCP` — valor retido de contribuição previdenciária.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_cp: Option<String>,
    /// `vRetIRRF` — valor retido de IRRF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_irrf: Option<String>,
    /// `vRetCSLL` — valor retido de CSLL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_ret_csll: Option<String>,
}

/// `<piscofins>` — grupo PIS/COFINS dentro de `tribFed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct PisCofins {
    /// `CST` do PIS/COFINS (2 dígitos).
    pub cst: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_bc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq_pis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_aliq_cofins: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_pis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_cofins: Option<String>,
    /// `tpRetPisCofins` — tipo de retenção (0..9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_ret: Option<String>,
}

/// `<IBSCBS>` (TCRTCInfoIBSCBS) — grupo declarado pelo emitente para IBS/CBS.
/// Irmão de `valores` em `infDPS`. A SEFIN calcula alíquotas/valores e os
/// devolve na NFS-e; a DPS apenas **declara** CST, classificação e contexto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Ibscbs {
    /// `finNFSe` — finalidade de emissão (`0` normal, ...).
    pub fin_nfse: String,
    /// `indFinal` — operação de uso/consumo pessoal (`0`/`1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ind_final: Option<String>,
    /// `cIndOp` — código indicador da operação de fornecimento (6 dígitos).
    pub c_ind_op: String,
    /// `indDest` — a respeito do destinatário (`0`/`1`/`2`).
    pub ind_dest: String,
    /// `CST` do IBS/CBS (3 dígitos).
    pub cst: String,
    /// `cClassTrib` — classificação tributária IBS/CBS (6 dígitos).
    pub c_class_trib: String,
    /// `cCredPres` — código de crédito presumido (opcional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_cred_pres: Option<String>,
}
