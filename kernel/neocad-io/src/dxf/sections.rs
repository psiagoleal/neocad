// Caminho relativo: kernel/neocad-io/src/dxf/sections.rs
//! \file kernel/neocad-io/src/dxf/sections.rs
//! \brief Percurso das seções de um arquivo DXF.
//! \author Iago Leal
//! \date 2026-08-12

use core::fmt;

use super::pairs::{pairs, DxfPair, DxfPairError, DxfPairs};

/// Seção conhecida do formato.
///
/// A lista cobre o que a especificação define. Seção fora dela chega como
/// [`SectionKind::Other`] em vez de derrubar a leitura: o DXF é extensível, e
/// recusar um arquivo por causa de uma seção que não conhecemos seria trocar um
/// desenho inteiro por uma parte que provavelmente nem usaríamos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// Variáveis de cabeçalho.
    Header,
    /// Classes de objeto definidas pela aplicação.
    Classes,
    /// Tabelas de símbolos — camadas, estilos, tipos de linha, blocos.
    Tables,
    /// Definições de bloco.
    Blocks,
    /// Entidades do desenho, de espaço-modelo **e** de espaço-papel.
    Entities,
    /// Objetos não gráficos — inclui os `LAYOUT`, que descrevem as pranchas.
    Objects,
    /// Miniatura de visualização.
    ThumbnailImage,
    /// Dados de componentes da aplicação.
    AcdsData,
    /// Seção que a especificação não define.
    Other,
}

impl SectionKind {
    /// Classifica pelo nome que segue o marcador `0/SECTION`.
    fn from_name(name: &str) -> Self {
        match name {
            "HEADER" => Self::Header,
            "CLASSES" => Self::Classes,
            "TABLES" => Self::Tables,
            "BLOCKS" => Self::Blocks,
            "ENTITIES" => Self::Entities,
            "OBJECTS" => Self::Objects,
            "THUMBNAILIMAGE" => Self::ThumbnailImage,
            "ACDSDATA" => Self::AcdsData,
            _ => Self::Other,
        }
    }
}

/// Uma seção do arquivo, com os pares que estão dentro dela.
///
/// Os marcadores `0/SECTION`, `2/<nome>` e `0/ENDSEC` **não** entram em
/// [`Section::pairs`]: são a moldura, não o conteúdo. Quem consome a seção
/// recebe só o que precisa interpretar.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Classificação da seção.
    pub kind: SectionKind,
    /// Nome como apareceu no arquivo, sem espaços de borda.
    pub name: String,
    /// Pares internos, na ordem do arquivo.
    pub pairs: Vec<DxfPair>,
}

/// Falha ao percorrer as seções.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DxfSectionError {
    /// O fluxo de pares falhou. Encerra a leitura.
    Pair(DxfPairError),
    /// `0/SECTION` não foi seguido do `2/<nome>` que a especificação exige.
    MissingName {
        /// Código do par encontrado no lugar do nome, se houver algum.
        found: Option<u16>,
    },
    /// A seção terminou por fim de arquivo ou por um novo `0/SECTION`, sem
    /// `0/ENDSEC`.
    UnterminatedSection {
        /// Nome da seção que ficou aberta.
        name: String,
    },
    /// Par encontrado fora de qualquer seção.
    StrayPair {
        /// Código do par.
        code: u16,
    },
}

impl fmt::Display for DxfSectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pair(erro) => write!(formatter, "{erro}"),
            Self::MissingName { found: Some(code) } => write!(
                formatter,
                "a seção começa com o código {code} no lugar do nome (código 2)"
            ),
            Self::MissingName { found: None } => {
                write!(formatter, "o arquivo termina logo após abrir uma seção")
            }
            Self::UnterminatedSection { name } => {
                write!(formatter, "a seção {name} não é fechada por ENDSEC")
            }
            Self::StrayPair { code } => {
                write!(formatter, "código {code} fora de qualquer seção")
            }
        }
    }
}

impl core::error::Error for DxfSectionError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Pair(erro) => Some(erro),
            _ => None,
        }
    }
}

/// Percorre as seções de um arquivo DXF.
///
/// # Erro que encerra e erro que não encerra
///
/// A distinção é o que separa um arquivo ilegível de um arquivo malformado, e
/// vale a pena ser explícita. Falha no fluxo de pares ([`DxfSectionError::Pair`])
/// **encerra**: o leitor perdeu o sincronismo e não sabe mais o que é código e o
/// que é valor. As demais falhas são locais — o percurso volta a procurar o
/// próximo `0/SECTION` e continua entregando seções.
///
/// É a diferença entre "não dá para ler este arquivo" e "esta parte do arquivo
/// está torta". Recusar o desenho inteiro pela segunda seria perder trabalho
/// alheio por preciosismo.
#[derive(Debug)]
pub struct Sections<'a> {
    pares: DxfPairs<'a>,
    encerrado: bool,
    /// Verdadeiro quando o `0/SECTION` da próxima seção já foi consumido — o que
    /// acontece ao detectar uma seção sem `ENDSEC`. Sem isso, relatar o erro
    /// custaria a seção seguinte.
    marcador_pendente: bool,
}

impl<'a> Sections<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            pares: pairs(input),
            encerrado: false,
            marcador_pendente: false,
        }
    }

    /// Texto de um par, aparado, quando o par é textual.
    ///
    /// Os marcadores aparecem com espaço à direita em arquivo de origem real, e
    /// comparar sem aparar recusaria arquivos que todo mundo abre.
    fn marcador(par: &DxfPair) -> Option<&str> {
        par.value.as_text().map(str::trim)
    }

    /// Avança até o `0/SECTION` que abre a próxima seção.
    ///
    /// Devolve `Ok(false)` no fim do arquivo — inclusive quando falta o `0/EOF`,
    /// que é ausência tolerada: um arquivo truncado no fim ainda entrega tudo o
    /// que veio antes.
    fn procurar_secao(&mut self) -> Result<bool, DxfSectionError> {
        loop {
            let par = match self.pares.next() {
                None => return Ok(false),
                Some(Err(erro)) => {
                    self.encerrado = true;
                    return Err(DxfSectionError::Pair(erro));
                }
                Some(Ok(par)) => par,
            };

            if par.is_comment() {
                continue;
            }

            if par.code == 0 {
                match Self::marcador(&par) {
                    Some("SECTION") => return Ok(true),
                    Some("EOF") => {
                        self.encerrado = true;
                        return Ok(false);
                    }
                    _ => {}
                }
            }

            return Err(DxfSectionError::StrayPair { code: par.code });
        }
    }

    /// Lê o `2/<nome>` que a especificação exige logo após `0/SECTION`.
    fn ler_nome(&mut self) -> Result<String, DxfSectionError> {
        match self.pares.next() {
            None => Err(DxfSectionError::MissingName { found: None }),
            Some(Err(erro)) => {
                self.encerrado = true;
                Err(DxfSectionError::Pair(erro))
            }
            Some(Ok(par)) if par.code == 2 => {
                Ok(Self::marcador(&par).unwrap_or_default().to_owned())
            }
            Some(Ok(par)) => Err(DxfSectionError::MissingName {
                found: Some(par.code),
            }),
        }
    }

    /// Acumula os pares até `0/ENDSEC`.
    fn ler_conteudo(&mut self, name: String) -> Result<Section, DxfSectionError> {
        let mut conteudo = Vec::new();

        loop {
            let par = match self.pares.next() {
                None => return Err(DxfSectionError::UnterminatedSection { name }),
                Some(Err(erro)) => {
                    self.encerrado = true;
                    return Err(DxfSectionError::Pair(erro));
                }
                Some(Ok(par)) => par,
            };

            if par.code == 0 {
                match Self::marcador(&par) {
                    Some("ENDSEC") => {
                        return Ok(Section {
                            kind: SectionKind::from_name(&name),
                            name,
                            pairs: conteudo,
                        })
                    }
                    // Uma seção nova abrindo dentro de outra só pode significar
                    // `ENDSEC` faltando. O marcador fica pendente para que a
                    // seção seguinte não se perca junto com o erro.
                    Some("SECTION") => {
                        self.marcador_pendente = true;
                        return Err(DxfSectionError::UnterminatedSection { name });
                    }
                    Some("EOF") => {
                        self.encerrado = true;
                        return Err(DxfSectionError::UnterminatedSection { name });
                    }
                    _ => {}
                }
            }

            conteudo.push(par);
        }
    }
}

impl Iterator for Sections<'_> {
    type Item = Result<Section, DxfSectionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encerrado {
            return None;
        }

        if self.marcador_pendente {
            self.marcador_pendente = false;
        } else {
            match self.procurar_secao() {
                Ok(false) => return None,
                Ok(true) => {}
                Err(erro) => return Some(Err(erro)),
            }
        }

        let name = match self.ler_nome() {
            Ok(name) => name,
            Err(erro) => return Some(Err(erro)),
        };

        Some(self.ler_conteudo(name))
    }
}

/// Percorre as seções de um arquivo DXF.
///
/// # Exemplo
///
/// ```
/// use neocad_io::{sections, SectionKind};
///
/// let arquivo = b"  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n  1\nAC1015\n  0\nENDSEC\n  0\nEOF\n";
/// let lidas: Result<Vec<_>, _> = sections(arquivo).collect();
/// let lidas = lidas?;
///
/// assert_eq!(lidas.len(), 1);
/// assert_eq!(lidas[0].kind, SectionKind::Header);
/// // Os marcadores ficam de fora: sobram os dois pares de conteúdo.
/// assert_eq!(lidas[0].pairs.len(), 2);
/// # Ok::<(), neocad_io::DxfSectionError>(())
/// ```
#[must_use]
pub fn sections(input: &[u8]) -> Sections<'_> {
    Sections::new(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DxfValue;

    /// Monta um arquivo a partir de pares `(código, valor)`, para os testes não
    /// virarem paredes de `\n`.
    fn arquivo(pares: &[(u16, &str)]) -> Vec<u8> {
        let mut saida = String::new();

        for (codigo, valor) in pares {
            saida.push_str(&format!("{codigo:>3}\n{valor}\n"));
        }

        saida.into_bytes()
    }

    fn ler(bytes: &[u8]) -> Vec<Result<Section, DxfSectionError>> {
        sections(bytes).collect()
    }

    fn nomes(lidas: &[Result<Section, DxfSectionError>]) -> Vec<&str> {
        lidas
            .iter()
            .filter_map(|resultado| resultado.as_ref().ok())
            .map(|secao| secao.name.as_str())
            .collect()
    }

    #[test]
    fn le_as_secoes_de_um_arquivo_bem_formado() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (1, "AC1015"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "LINE"),
            (8, "0"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 2);
        assert_eq!(nomes(&lidas), ["HEADER", "ENTITIES"]);

        let header = lidas[0].as_ref().expect("bem formada");
        assert_eq!(header.kind, SectionKind::Header);
        assert_eq!(header.pairs.len(), 2);
        assert_eq!(header.pairs[0].code, 9);

        let entidades = lidas[1].as_ref().expect("bem formada");
        assert_eq!(entidades.kind, SectionKind::Entities);
        assert_eq!(entidades.pairs.len(), 2);
    }

    #[test]
    fn marcadores_ficam_fora_do_conteudo() {
        let bytes = arquivo(&[(0, "SECTION"), (2, "HEADER"), (0, "ENDSEC"), (0, "EOF")]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 1);
        assert!(lidas[0].as_ref().expect("bem formada").pairs.is_empty());
    }

    #[test]
    fn secao_desconhecida_e_entregue_e_nao_interrompe() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "ALGO_QUE_NAO_CONHECEMOS"),
            (1, "conteúdo"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 2);
        let desconhecida = lidas[0].as_ref().expect("entregue, não recusada");
        assert_eq!(desconhecida.kind, SectionKind::Other);
        assert_eq!(desconhecida.name, "ALGO_QUE_NAO_CONHECEMOS");
        assert_eq!(desconhecida.pairs.len(), 1);
        assert_eq!(
            lidas[1].as_ref().expect("bem formada").kind,
            SectionKind::Entities
        );
    }

    #[test]
    fn endsec_ausente_vira_erro_nomeado_sem_perder_a_secao_seguinte() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "HEADER"),
            (9, "$ACADVER"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 2);
        assert_eq!(
            lidas[0],
            Err(DxfSectionError::UnterminatedSection {
                name: String::from("HEADER")
            })
        );
        // O ponto do `marcador_pendente`: o erro não leva a próxima junto.
        assert_eq!(
            lidas[1].as_ref().expect("ainda legível").kind,
            SectionKind::Entities
        );
    }

    #[test]
    fn endsec_ausente_no_fim_do_arquivo_tambem_e_erro_nomeado() {
        let bytes = arquivo(&[(0, "SECTION"), (2, "ENTITIES"), (0, "LINE")]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 1);
        assert_eq!(
            lidas[0],
            Err(DxfSectionError::UnterminatedSection {
                name: String::from("ENTITIES")
            })
        );
        assert!(lidas[0]
            .as_ref()
            .expect_err("sem ENDSEC")
            .to_string()
            .contains("ENTITIES"));
    }

    #[test]
    fn eof_dentro_de_secao_fecha_a_leitura_com_erro() {
        let bytes = arquivo(&[(0, "SECTION"), (2, "ENTITIES"), (0, "EOF")]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 1);
        assert!(matches!(
            lidas[0],
            Err(DxfSectionError::UnterminatedSection { .. })
        ));
    }

    #[test]
    fn ordem_incomum_de_secoes_e_respeitada() {
        // Nada na leitura pode supor a ordem canônica: arquivo gerado por
        // ferramenta de terceiro costuma trocar a ordem, e recusar por isso
        // seria inventar uma regra que o formato não tem.
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "OBJECTS"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "HEADER"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "BLOCKS"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(nomes(&lidas), ["ENTITIES", "OBJECTS", "HEADER", "BLOCKS"]);
    }

    #[test]
    fn secao_repetida_e_entregue_duas_vezes() {
        // O formato não proíbe, e concatenação de arquivos produz isso. Cabe a
        // quem consome decidir se junta ou se fica com a última.
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "LINE"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "CIRCLE"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(nomes(&lidas), ["ENTITIES", "ENTITIES"]);
    }

    #[test]
    fn arquivo_sem_eof_entrega_o_que_veio_antes() {
        let bytes = arquivo(&[(0, "SECTION"), (2, "HEADER"), (0, "ENDSEC")]);

        let lidas = ler(&bytes);

        assert_eq!(nomes(&lidas), ["HEADER"]);
    }

    #[test]
    fn pares_depois_do_eof_sao_ignorados() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "HEADER"),
            (0, "ENDSEC"),
            (0, "EOF"),
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "ENDSEC"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(nomes(&lidas), ["HEADER"]);
    }

    #[test]
    fn comentario_fora_de_secao_e_ignorado() {
        let bytes = arquivo(&[
            (999, "gerado por alguma ferramenta"),
            (0, "SECTION"),
            (2, "HEADER"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(nomes(&lidas), ["HEADER"]);
    }

    #[test]
    fn par_solto_fora_de_secao_e_relatado_sem_encerrar() {
        let bytes = arquivo(&[
            (8, "camada perdida"),
            (0, "SECTION"),
            (2, "HEADER"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 2);
        assert_eq!(lidas[0], Err(DxfSectionError::StrayPair { code: 8 }));
        assert_eq!(nomes(&lidas), ["HEADER"]);
    }

    #[test]
    fn nome_ausente_e_relatado_com_o_codigo_encontrado() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (1, "não é o nome"),
            (0, "ENDSEC"),
            (0, "SECTION"),
            (2, "HEADER"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(
            lidas[0],
            Err(DxfSectionError::MissingName { found: Some(1) })
        );
        assert_eq!(nomes(&lidas), ["HEADER"]);
    }

    #[test]
    fn arquivo_que_acaba_ao_abrir_secao_nao_entra_em_panico() {
        let bytes = arquivo(&[(0, "SECTION")]);

        let lidas = ler(&bytes);

        assert_eq!(lidas, [Err(DxfSectionError::MissingName { found: None })]);
    }

    #[test]
    fn marcador_com_espaco_a_direita_e_reconhecido() {
        // Gravador de origem real deixa espaço depois do marcador. Comparar sem
        // aparar recusaria arquivo que todo mundo abre.
        let bytes = arquivo(&[
            (0, "SECTION  "),
            (2, "HEADER "),
            (0, "ENDSEC  "),
            (0, "EOF "),
        ]);

        let lidas = ler(&bytes);

        assert_eq!(lidas.len(), 1);
        assert_eq!(
            lidas[0].as_ref().expect("marcador válido").kind,
            SectionKind::Header
        );
    }

    #[test]
    fn erro_do_fluxo_de_pares_encerra_a_leitura() {
        // Código não numérico: o fluxo perde o sincronismo e não há como seguir.
        let bytes = b"  0\nSECTION\n  2\nHEADER\nabc\nqualquer\n  0\nENDSEC\n  0\nEOF\n";

        let lidas = ler(bytes);

        assert_eq!(lidas.len(), 1);
        assert!(matches!(lidas[0], Err(DxfSectionError::Pair(_))));
        assert!(core::error::Error::source(lidas[0].as_ref().expect_err("erro de par")).is_some());
    }

    #[test]
    fn entradas_vazias_nao_produzem_secao() {
        assert!(ler(b"").is_empty());
        assert!(ler(b"  0\nEOF\n").is_empty());
    }

    #[test]
    fn conteudo_preserva_ordem_e_valor_dos_pares() {
        let bytes = arquivo(&[
            (0, "SECTION"),
            (2, "ENTITIES"),
            (0, "LINE"),
            (8, "Fiação"),
            (10, "1.5"),
            (20, "-2"),
            (0, "ENDSEC"),
            (0, "EOF"),
        ]);

        let lidas = ler(&bytes);
        let secao = lidas[0].as_ref().expect("bem formada");

        assert_eq!(
            secao.pairs.iter().map(|par| par.code).collect::<Vec<_>>(),
            [0, 8, 10, 20]
        );
        assert_eq!(secao.pairs[1].value, DxfValue::Text(String::from("Fiação")));
        assert_eq!(secao.pairs[2].value, DxfValue::Real(1.5));
        assert_eq!(secao.pairs[3].value, DxfValue::Real(-2.0));
    }
}
