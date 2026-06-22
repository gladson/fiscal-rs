//! Public data structures for the MDF-e (model 58) XML blocks.
//!
//! These mirror the leiaute 3.00 grouping: `Ide`, `Emit`, `Modal`
//! (road via `Rodo`), `InfDoc`, `Tot`, and `InfAdic`, assembled into
//! `MdfeBuildData`. Fields use the SEFAZ tag names in their documentation so
//! the mapping back to the XSD stays obvious.

use serde::{Deserialize, Serialize};

#[cfg(feature = "ts")]
use ts_rs::TS;

/// Top-level input for building an MDF-e document.
///
/// Carries every block the road-modal builder needs. The access key is derived
/// from `Ide` + `Emit` at build time, so it is not stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct MdfeBuildData {
    /// `<ide>` — identification block.
    pub ide: Ide,
    /// `<emit>` — issuer block.
    pub emit: Emit,
    /// `<infModal>` — transport modal (road implemented; others stubbed).
    pub modal: Modal,
    /// `<infDoc>` — linked fiscal documents, grouped by unload municipality.
    pub inf_doc: InfDoc,
    /// `<tot>` — document totals.
    pub tot: Tot,
    /// `<infAdic>` — optional additional information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_adic: Option<InfAdic>,
    /// Optional explicit 8-digit `cMDF` random code. When `None`, a code is
    /// generated at build time. Provided mainly for deterministic tests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_code: Option<String>,
}

// ── ide ──────────────────────────────────────────────────────────────────────

/// `<ide>` — MDF-e identification block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Ide {
    /// `cUF` — issuer state IBGE code (2 digits).
    pub c_uf: String,
    /// `tpAmb` — environment: `1` = production, `2` = homologation.
    pub tp_amb: String,
    /// `tpEmit` — issuer type: `1` = transport provider, `2` = own cargo,
    /// `3` = globalized.
    pub tp_emit: String,
    /// `serie` — document series.
    pub serie: u32,
    /// `nMDF` — sequential document number.
    pub n_mdf: u32,
    /// `modal` — transport modal: `1` road, `2` air, `3` waterway, `4` rail.
    pub modal: String,
    /// `dhEmi` — emission timestamp.
    pub dh_emi: chrono::DateTime<chrono::FixedOffset>,
    /// `tpEmis` — emission type: `1` = normal, `2` = contingency.
    pub tp_emis: String,
    /// `procEmi` — emission process code (usually `0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proc_emi: Option<String>,
    /// `verProc` — emitting-application version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ver_proc: Option<String>,
    /// `UFIni` — trip start state (UF abbreviation).
    pub uf_ini: String,
    /// `UFFim` — trip end state (UF abbreviation).
    pub uf_fim: String,
    /// `infMunCarrega` — loading municipalities (at least one).
    pub inf_mun_carrega: Vec<MunCarrega>,
    /// `infPercurso` — states crossed during the route, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_percurso: Vec<String>,
    /// `dhIniViagem` — optional trip-start timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dh_ini_viagem: Option<chrono::DateTime<chrono::FixedOffset>>,
}

/// `infMunCarrega` — a single loading municipality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct MunCarrega {
    /// `cMunCarrega` — IBGE municipality code (7 digits).
    pub c_mun: String,
    /// `xMunCarrega` — municipality name.
    pub x_mun: String,
}

// ── emit ─────────────────────────────────────────────────────────────────────

/// `<emit>` — issuer block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Emit {
    /// `CNPJ` — issuer CNPJ (14 digits).
    pub cnpj: String,
    /// `IE` — state registration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    /// `xNome` — corporate name.
    pub x_nome: String,
    /// `xFant` — trade name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    /// `enderEmit` — issuer address.
    pub ender_emit: EnderEmit,
}

/// `enderEmit` — issuer address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct EnderEmit {
    /// `xLgr` — street.
    pub x_lgr: String,
    /// `nro` — number.
    pub nro: String,
    /// `xCpl` — complement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_cpl: Option<String>,
    /// `xBairro` — neighbourhood.
    pub x_bairro: String,
    /// `cMun` — IBGE municipality code (7 digits).
    pub c_mun: String,
    /// `xMun` — municipality name.
    pub x_mun: String,
    /// `CEP` — postal code (8 digits).
    pub cep: String,
    /// `UF` — state abbreviation.
    pub uf: String,
    /// `fone` — phone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
    /// `email` — email address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ── infModal ─────────────────────────────────────────────────────────────────

/// `<infModal>` — transport modal. Exactly one modal block is emitted inside
/// `<infModal versaoModal="3.00">`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Modal {
    /// Road modal (`rodo`).
    Rodo(Rodo),
    /// Air modal (`aereo`).
    Aereo(Aereo),
    /// Waterway modal (`aquav`).
    Aquav(Aquav),
    /// Rail modal (`ferrov`).
    Ferrov(Ferrov),
}

// ── aereo ─────────────────────────────────────────────────────────────────────

/// `aereo` — air modal block. All fields are required by the MDF-e 3.00 schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Aereo {
    /// `nac` — aircraft nationality mark (1–4 chars).
    pub nac: String,
    /// `matr` — aircraft registration mark (1–6 chars).
    pub matr: String,
    /// `nVoo` — flight number (5–9 chars, e.g. `AB1234`).
    pub n_voo: String,
    /// `cAerEmb` — boarding aerodrome code (IATA/OACI, 3–4 chars).
    pub c_aer_emb: String,
    /// `cAerDes` — destination aerodrome code (3–4 chars).
    pub c_aer_des: String,
    /// `dVoo` — flight date (`AAAA-MM-DD`).
    pub d_voo: String,
}

// ── aquav ─────────────────────────────────────────────────────────────────────

/// `aquav` — waterway modal block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Aquav {
    /// `irin` — IRIN of the vessel (1–10 chars). Always required.
    pub irin: String,
    /// `tpEmb` — vessel type code (2 digits).
    pub tp_emb: String,
    /// `cEmbar` — vessel code (1–10 chars).
    pub c_embar: String,
    /// `xEmbar` — vessel name (1–60 chars).
    pub x_embar: String,
    /// `nViag` — voyage number.
    pub n_viag: String,
    /// `cPrtEmb` — boarding port code (1–5 chars).
    pub c_prt_emb: String,
    /// `cPrtDest` — destination port code (1–5 chars).
    pub c_prt_dest: String,
    /// `prtTrans` — transshipment port (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prt_trans: Option<String>,
    /// `tpNav` — navigation type: `0` inland, `1` cabotage (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_nav: Option<String>,
    /// `infTermCarreg` — loading terminals (0–5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_term_carreg: Vec<TermCarreg>,
    /// `infTermDescarreg` — unloading terminals (0–5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_term_descarreg: Vec<TermDescarreg>,
    /// `infEmbComb` — convoy vessels (0–30).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_emb_comb: Vec<EmbComb>,
    /// `infUnidCargaVazia` — empty cargo units.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_unid_carga_vazia: Vec<UnidCargaVazia>,
    /// `infUnidTranspVazia` — empty transport units.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_unid_transp_vazia: Vec<UnidTranspVazia>,
    /// `MMSI` — Maritime Mobile Service Identity (9 digits, optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mmsi: Option<String>,
}

/// `infTermCarreg` — a loading terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct TermCarreg {
    /// `cTermCarreg` — loading terminal code (1–8 chars).
    pub c_term_carreg: String,
    /// `xTermCarreg` — loading terminal name (1–60 chars).
    pub x_term_carreg: String,
}

/// `infTermDescarreg` — an unloading terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct TermDescarreg {
    /// `cTermDescarreg` — unloading terminal code (1–8 chars).
    pub c_term_descarreg: String,
    /// `xTermDescarreg` — unloading terminal name (1–60 chars).
    pub x_term_descarreg: String,
}

/// `infEmbComb` — a convoy (pushed/towed) vessel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct EmbComb {
    /// `cEmbComb` — convoy vessel code (1–10 chars).
    pub c_emb_comb: String,
    /// `xBalsa` — barge identifier (1–60 chars).
    pub x_balsa: String,
}

/// `infUnidCargaVazia` — an empty cargo unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct UnidCargaVazia {
    /// `idUnidCargaVazia` — empty cargo unit identifier (container).
    pub id_unid_carga_vazia: String,
    /// `tpUnidCargaVazia` — unit type: `1` container, `2` ULD, `3` pallet, `4` other.
    pub tp_unid_carga_vazia: String,
}

/// `infUnidTranspVazia` — an empty transport unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct UnidTranspVazia {
    /// `idUnidTranspVazia` — empty transport unit identifier.
    pub id_unid_transp_vazia: String,
    /// `tpUnidTranspVazia` — unit type: `1` truck tractor, `2` trailer.
    pub tp_unid_transp_vazia: String,
}

// ── ferrov ────────────────────────────────────────────────────────────────────

/// `ferrov` — rail modal block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Ferrov {
    /// `trem` — train composition information.
    pub trem: Trem,
    /// `vag` — wagons (at least one).
    pub vag: Vec<Vag>,
}

/// `trem` — train composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Trem {
    /// `xPref` — train prefix (1–10 chars).
    pub x_pref: String,
    /// `dhTrem` — origin release datetime (UTC offset; optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dh_trem: Option<String>,
    /// `xOri` — origin station abbreviation (1–3 chars).
    pub x_ori: String,
    /// `xDest` — destination station abbreviation (1–3 chars).
    pub x_dest: String,
    /// `qVag` — number of loaded wagons.
    pub q_vag: String,
}

/// `vag` — a single wagon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Vag {
    /// `pesoBC` — freight calculation-base weight, in tonnes (decimal string).
    pub peso_bc: String,
    /// `pesoR` — real weight, in tonnes (decimal string).
    pub peso_r: String,
    /// `tpVag` — wagon type (3 chars; optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tp_vag: Option<String>,
    /// `serie` — wagon identification series (3 chars).
    pub serie: String,
    /// `nVag` — wagon identification number.
    pub n_vag: String,
    /// `nSeq` — sequence within the composition (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_seq: Option<String>,
    /// `TU` — useful tonnage (decimal string).
    pub tu: String,
}

/// `rodo` — road modal block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Rodo {
    /// `infANTT` — ANTT (highway authority) information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_antt: Option<InfAntt>,
    /// `veicTracao` — traction (tractor) vehicle.
    pub veic_tracao: VeicTracao,
    /// `veicReboque` — towed (trailer) vehicles.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub veic_reboque: Vec<VeicReboque>,
}

/// `infANTT` — ANTT registration and freight information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfAntt {
    /// `RNTRC` — national road-cargo transporter registry (8 chars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rntrc: Option<String>,
    /// `infCIOT` — CIOT (cargo transport operation) entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_ciot: Vec<InfCiot>,
    /// `valePed` — vale pedágio obrigatório (0..N). Lei 10.209/2001.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vale_ped: Vec<ValePed>,
}

/// `valePed` — Vale Pedágio Obrigatório no MDF-e.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct ValePed {
    /// `CNPJForn` — CNPJ do FVPO habilitado pela ANTT.
    pub cnpj_forn: String,
    /// `nCompra` — IDVPO gerado pela ANTT.
    pub n_compra: String,
    /// `vValePed` — valor do vale pedágio em reais.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v_vale_ped: Option<String>,
    /// `tpValePed` — tipo: `01` TAG, `04` OCR/leitura de placa. Obrigatório desde 31/01/2025.
    pub tp_vale_ped: String,
    /// `categCombVeic` — categoria combinação veicular (02..14 eixos). Obrigatório quando valePed presente (rejeição 731).
    pub categ_comb_veic: String,
}

/// `infCIOT` — a single CIOT entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfCiot {
    /// `CIOT` — 12-digit CIOT code.
    pub ciot: String,
    /// Responsible party tax id (CNPJ or CPF). The correct tag is chosen by length.
    pub tax_id: String,
}

/// `veicTracao` — traction (tractor) vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct VeicTracao {
    /// `cInt` — internal vehicle code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_int: Option<String>,
    /// `placa` — license plate.
    pub placa: String,
    /// `RENAVAM` — vehicle registration number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renavam: Option<String>,
    /// `tara` — tare weight in kg.
    pub tara: u32,
    /// `capKG` — load capacity in kg.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_kg: Option<u32>,
    /// `capM3` — load capacity in m³.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_m3: Option<u32>,
    /// `prop` — vehicle owner, when other than the issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prop: Option<Prop>,
    /// `condutor` — drivers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub condutor: Vec<Condutor>,
    /// `tpRod` — wheel type (`01`–`06`).
    pub tp_rod: String,
    /// `tpCar` — body type (`00`–`05`).
    pub tp_car: String,
    /// `UF` — vehicle licensing state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
}

/// `prop` — vehicle owner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Prop {
    /// Owner tax id (CNPJ or CPF). The correct tag is chosen by length.
    pub tax_id: String,
    /// `RNTRC` — owner road-cargo registry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rntrc: Option<String>,
    /// `xNome` — owner name.
    pub x_nome: String,
    /// `IE` — owner state registration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    /// `UF` — owner state.
    pub uf: String,
    /// `tpProp` — owner type (`0`–`3`).
    pub tp_prop: String,
}

/// `condutor` — a single driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Condutor {
    /// `xNome` — driver name.
    pub x_nome: String,
    /// `CPF` — driver CPF (11 digits).
    pub cpf: String,
}

/// `veicReboque` — a towed (trailer) vehicle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct VeicReboque {
    /// `cInt` — internal vehicle code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_int: Option<String>,
    /// `placa` — license plate.
    pub placa: String,
    /// `RENAVAM` — vehicle registration number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renavam: Option<String>,
    /// `tara` — tare weight in kg.
    pub tara: u32,
    /// `capKG` — load capacity in kg.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_kg: Option<u32>,
    /// `capM3` — load capacity in m³.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cap_m3: Option<u32>,
    /// `prop` — trailer owner, when other than the issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prop: Option<Prop>,
    /// `tpCar` — body type (`00`–`05`).
    pub tp_car: String,
    /// `UF` — vehicle licensing state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uf: Option<String>,
}

// ── infDoc ───────────────────────────────────────────────────────────────────

/// `<infDoc>` — linked fiscal documents grouped by unload municipality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfDoc {
    /// `infMunDescarga` — one group per unload municipality.
    pub inf_mun_descarga: Vec<MunDescarga>,
}

/// `infMunDescarga` — documents unloaded at one municipality.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct MunDescarga {
    /// `cMunDescarga` — IBGE municipality code (7 digits).
    pub c_mun: String,
    /// `xMunDescarga` — municipality name.
    pub x_mun: String,
    /// `infNFe` — linked NF-e access keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_nfe: Vec<String>,
    /// `infCTe` — linked CT-e access keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_cte: Vec<String>,
    /// `infMDFeTransp` — linked transported MDF-e access keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inf_mdfe: Vec<String>,
}

// ── tot ──────────────────────────────────────────────────────────────────────

/// `<tot>` — document totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct Tot {
    /// `qCTe` — number of linked CT-e.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q_cte: Option<u32>,
    /// `qNFe` — number of linked NF-e.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q_nfe: Option<u32>,
    /// `qMDFe` — number of transported MDF-e.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q_mdfe: Option<u32>,
    /// `vCarga` — total cargo value (BRL).
    pub v_carga: f64,
    /// `cUnid` — weight unit: `01` = KG, `02` = TON.
    pub c_unid: String,
    /// `qCarga` — total cargo weight in the unit given by `cUnid`.
    pub q_carga: f64,
}

// ── infAdic ──────────────────────────────────────────────────────────────────

/// `<infAdic>` — optional additional information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(TS), ts(export))]
pub struct InfAdic {
    /// `infAdFisco` — additional fiscal-interest information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_ad_fisco: Option<String>,
    /// `infCpl` — additional taxpayer-interest information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inf_cpl: Option<String>,
}
