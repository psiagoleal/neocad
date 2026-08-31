// Caminho relativo: kernel/neocad-io/src/dxf/report.rs
//! \file kernel/neocad-io/src/dxf/report.rs
//! \brief Relatório do que a leitura de um DXF não compreendeu.
//! \author Iago Leal
//! \date 2026-08-15

use core::fmt;
use std::collections::BTreeMap;

use super::entities::RejectedEntity;
use super::sections::DxfSectionError;
use super::tables::RejectedLayer;

/// O que a leitura encontrou e não soube representar.
///
/// # Por que um relatório, e não silêncio
///
/// O conversor DXF/DWG do upstream descarta em silêncio a entidade que não sabe
/// converter — `createEntity` devolve `null` e o chamador simplesmente não a
/// empilha. O efeito prático foi medido neste projeto: uma cobertura anunciada
/// como 85% era, contra o espaço-modelo, 61%, e ninguém tinha como saber, porque
/// o que sumia não aparecia em contagem nenhuma.
///
/// Este relatório existe para que isso não se repita na leitura própria. Toda
/// entidade lida cai numa de duas categorias — representada ou **contada aqui** —
/// e nenhuma terceira via existe.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DxfReport {
    /// Tipos de entidade que o modelo ainda não representa, com a contagem de
    /// cada um, somando espaço-modelo, espaço-papel e definições de bloco.
    pub unsupported: BTreeMap<String, usize>,
    /// Camadas citadas por entidade sem estarem na tabela, criadas na leitura.
    pub created_layers: Vec<String>,
    /// Entidades que não puderam ser criadas.
    pub rejected_entities: Vec<RejectedEntity>,
    /// Camadas que a tabela do modelo recusou.
    pub rejected_layers: Vec<RejectedLayer>,
    /// Códigos de grupo vistos em registro de camada e ainda não interpretados.
    pub unread_layer_codes: BTreeMap<u16, usize>,
    /// Seções presentes no arquivo que a leitura ainda não consome, com quantos
    /// pares cada uma trazia.
    ///
    /// Hoje inclui a `OBJECTS`, onde moram os objetos `LAYOUT` — o que a torna a
    /// medida direta do que falta para a fase KL.
    pub skipped_sections: BTreeMap<String, usize>,
    /// Falhas locais do percurso de seções, que não impediram a leitura.
    pub section_errors: Vec<DxfSectionError>,
    /// Abas cujo vínculo com o bloco não resolveu.
    ///
    /// O layout continua sendo entregue: perder a prancha porque o ponteiro está
    /// torto seria descartar o conteúdo por causa de um handle.
    pub unresolved_layouts: Vec<String>,
}

impl DxfReport {
    /// Total de entidades não representadas, somando todos os tipos.
    #[must_use]
    pub fn unsupported_count(&self) -> usize {
        self.unsupported.values().sum()
    }

    /// Indica que o arquivo foi compreendido por inteiro.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.unsupported.is_empty()
            && self.rejected_entities.is_empty()
            && self.rejected_layers.is_empty()
            && self.section_errors.is_empty()
    }

    /// Tipos não representados, do mais frequente para o menos.
    ///
    /// É a lista que orienta o que implementar em seguida: num acervo real o
    /// peso é muito desigual, e adivinhar a ordem custa trabalho no lugar errado.
    /// O desempate é alfabético, para a saída ser determinística.
    #[must_use]
    pub fn unsupported_by_frequency(&self) -> Vec<(&str, usize)> {
        let mut tipos: Vec<(&str, usize)> = self
            .unsupported
            .iter()
            .map(|(tipo, quantidade)| (tipo.as_str(), *quantidade))
            .collect();

        tipos.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        tipos
    }

    /// Acrescenta uma ocorrência de tipo não representado.
    pub(super) fn contar_nao_representado(&mut self, tipo: String, quantidade: usize) {
        *self.unsupported.entry(tipo).or_insert(0) += quantidade;
    }
}

impl fmt::Display for DxfReport {
    /// Resumo de uma linha, adequado a mensagem de interface.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_clean() {
            return write!(formatter, "arquivo compreendido por inteiro");
        }

        let mut partes = Vec::new();

        if !self.unsupported.is_empty() {
            partes.push(format!(
                "{} entidade(s) de {} tipo(s) não representada(s)",
                self.unsupported_count(),
                self.unsupported.len()
            ));
        }

        if !self.rejected_entities.is_empty() {
            partes.push(format!(
                "{} entidade(s) recusada(s)",
                self.rejected_entities.len()
            ));
        }

        if !self.rejected_layers.is_empty() {
            partes.push(format!(
                "{} camada(s) recusada(s)",
                self.rejected_layers.len()
            ));
        }

        if !self.section_errors.is_empty() {
            partes.push(format!("{} falha(s) de seção", self.section_errors.len()));
        }

        write!(formatter, "{}", partes.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relatorio_com(tipos: &[(&str, usize)]) -> DxfReport {
        let mut relatorio = DxfReport::default();

        for (tipo, quantidade) in tipos {
            relatorio.contar_nao_representado((*tipo).to_owned(), *quantidade);
        }

        relatorio
    }

    #[test]
    fn relatorio_vazio_e_limpo() {
        let relatorio = DxfReport::default();

        assert!(relatorio.is_clean());
        assert_eq!(relatorio.unsupported_count(), 0);
        assert_eq!(relatorio.to_string(), "arquivo compreendido por inteiro");
    }

    #[test]
    fn conta_por_tipo_e_no_total() {
        let relatorio = relatorio_com(&[("HATCH", 3), ("DIMENSION", 2)]);

        assert_eq!(relatorio.unsupported_count(), 5);
        assert_eq!(relatorio.unsupported.get("HATCH"), Some(&3));
        assert!(!relatorio.is_clean());
    }

    #[test]
    fn ordena_por_frequencia_com_desempate_alfabetico() {
        // O desempate existe para a saída não depender da ordem de inserção.
        let relatorio = relatorio_com(&[("SPLINE", 2), ("HATCH", 9), ("MTEXT", 2)]);

        assert_eq!(
            relatorio.unsupported_by_frequency(),
            [("HATCH", 9), ("MTEXT", 2), ("SPLINE", 2)]
        );
    }

    #[test]
    fn ocorrencias_do_mesmo_tipo_se_somam() {
        let relatorio = relatorio_com(&[("HATCH", 2), ("HATCH", 3)]);

        assert_eq!(relatorio.unsupported.get("HATCH"), Some(&5));
    }

    #[test]
    fn resumo_nomeia_o_que_houve() {
        let relatorio = relatorio_com(&[("HATCH", 4), ("SPLINE", 1)]);

        assert_eq!(
            relatorio.to_string(),
            "5 entidade(s) de 2 tipo(s) não representada(s)"
        );
    }

    #[test]
    fn secao_pulada_nao_torna_o_arquivo_incompreendido() {
        // A `OBJECTS` ainda não é consumida, e isso é lacuna conhecida, não
        // defeito do arquivo. Contar como sujeira faria todo DXF moderno
        // parecer problemático.
        let mut relatorio = DxfReport::default();
        relatorio
            .skipped_sections
            .insert(String::from("OBJECTS"), 42);

        assert!(relatorio.is_clean());
    }
}
