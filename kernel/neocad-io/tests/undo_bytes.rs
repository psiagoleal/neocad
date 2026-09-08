// Caminho relativo: kernel/neocad-io/tests/undo_bytes.rs
//! \file kernel/neocad-io/tests/undo_bytes.rs
//! \brief Desfazer uma edição devolve o desenho aos mesmos bytes.
//! \author Iago Leal
//! \date 2026-09-08
//!
//! O critério de aceite do MT-K3-08 é este arquivo: para cada operação de
//! edição, desfazer tem de restaurar o documento **byte a byte**.
//!
//! Comparar bytes é mais forte que comparar campo a campo porque não depende de
//! alguém lembrar de comparar o campo novo. A gravação é determinística por
//! decisão registrada (ADR 0004), então dois documentos iguais produzem
//! necessariamente os mesmos bytes, e dois documentos diferentes em qualquer
//! coisa que o formato carregue produzem bytes diferentes.

use neocad_geometry::{Arc, Circle, Line, Point2, Polyline, PolylineVertex};
use neocad_io::{write_dxf, DxfContents, EntitySpace, ReadEntity};
use neocad_model::{Document, Entity, EntityId, Geometry, Text};
use neocad_transaction::{commands, CommandStack};

/// Um desenho com uma de cada forma que o modelo representa.
fn desenho() -> (Document, Vec<EntityId>) {
    let mut document = Document::new();
    let camada = document.layers().default_layer();
    let formas = [
        Geometry::Line(Line {
            start: Point2::new(1.0, 2.0),
            end: Point2::new(9.0, 6.0),
        }),
        Geometry::Circle(Circle {
            center: Point2::new(4.0, 4.0),
            radius: 2.5,
        }),
        Geometry::Arc(Arc {
            center: Point2::new(-3.0, 1.0),
            radius: 1.5,
            start_angle: 0.3,
            end_angle: 2.1,
        }),
        Geometry::Polyline(Polyline {
            vertices: vec![
                PolylineVertex::new(Point2::ORIGIN, 0.75),
                PolylineVertex::straight(Point2::new(5.0, 0.0)),
                PolylineVertex::new(Point2::new(5.0, 5.0), -0.4),
                PolylineVertex::straight(Point2::new(0.0, 5.0)),
            ],
            closed: true,
        }),
        Geometry::Text(Text {
            position: Point2::new(2.0, 8.0),
            content: "CORTE A-A".to_owned(),
            height: 2.5,
            rotation: 0.6,
        }),
    ];

    let mut editor = document.edit();
    let mut ids = Vec::new();

    for forma in formas {
        ids.push(
            editor
                .insert_in_model_space(Entity::new(camada, forma))
                .expect("o espaço-modelo aceita a entidade"),
        );
    }

    let _ = editor.finish();

    (document, ids)
}

/// Grava o espaço-modelo do documento.
fn bytes(document: &Document) -> Vec<u8> {
    let entidades: Vec<ReadEntity> = document
        .entities_in_block(document.model_space())
        .map(|(_, entidade)| ReadEntity {
            space: EntitySpace::Model,
            entity: entidade.clone(),
        })
        .collect();

    write_dxf(&DxfContents {
        layers: document.layers(),
        entities: &entidades,
        blocks: &[],
        layouts: &[],
    })
}

/// As cinco operações, cada uma com um nome e um efeito.
#[allow(clippy::type_complexity)]
fn operacoes() -> Vec<(
    &'static str,
    Box<
        dyn Fn(
            &mut neocad_model::DocumentEditor<'_>,
            &[EntityId],
        ) -> Result<(), neocad_model::DocumentError>,
    >,
)> {
    vec![
        (
            "Mover",
            Box::new(|editor, ids| commands::move_by(editor, ids, 12.5, -3.25)),
        ),
        (
            "Girar",
            Box::new(|editor, ids| commands::rotate(editor, ids, Point2::new(1.0, 1.0), 0.7)),
        ),
        (
            "Espelhar",
            Box::new(|editor, ids| commands::mirror(editor, ids, Point2::new(2.0, 0.0), 0.4)),
        ),
        (
            "Escalar",
            Box::new(|editor, ids| commands::scale(editor, ids, Point2::new(-1.0, 2.0), 1.75)),
        ),
        (
            "Copiar",
            Box::new(|editor, ids| commands::copy(editor, ids, 20.0, 0.0).map(|_| ())),
        ),
    ]
}

#[test]
fn cada_operacao_desfeita_devolve_os_mesmos_bytes() {
    for (nome, operacao) in operacoes() {
        let (mut documento, ids) = desenho();
        let antes = bytes(&documento);
        let mut pilha = CommandStack::new();

        pilha
            .edit(&mut documento, nome, |editor| operacao(editor, &ids))
            .unwrap_or_else(|erro| panic!("{nome} falhou: {erro}"));

        assert_ne!(
            bytes(&documento),
            antes,
            "{nome} não alterou o desenho, então o teste não prova nada"
        );

        assert!(
            pilha.undo(&mut documento).expect("desfazer não falha"),
            "{nome} não deixou nada para desfazer"
        );

        assert_eq!(
            bytes(&documento),
            antes,
            "{nome} desfeito não devolveu os mesmos bytes"
        );
    }
}

#[test]
fn refazer_devolve_os_bytes_da_edicao() {
    let (mut documento, ids) = desenho();
    let mut pilha = CommandStack::new();

    pilha
        .edit(&mut documento, "Mover", |editor| {
            commands::move_by(editor, &ids, 3.0, 4.0)
        })
        .expect("mover não falha");

    let editado = bytes(&documento);

    pilha.undo(&mut documento).expect("desfazer não falha");
    pilha.redo(&mut documento).expect("refazer não falha");

    assert_eq!(bytes(&documento), editado);
}

#[test]
fn a_operacao_recusa_por_inteiro_quando_uma_entidade_nao_existe() {
    let (mut documento, ids) = desenho();
    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();

    // Um identificador válido seguido de um inválido: se a operação fosse
    // aplicada à medida que percorre, metade do desenho ficaria movida.
    let removida = ids[0];

    pilha
        .edit(&mut documento, "Apagar", |editor| {
            editor.remove_entity(removida).map(|_| ())
        })
        .expect("apagar não falha");

    let depois_de_apagar = bytes(&documento);

    let resultado = pilha.edit(&mut documento, "Mover", |editor| {
        commands::move_by(editor, &[ids[1], removida], 5.0, 5.0)
    });

    assert!(resultado.is_err(), "mover o inexistente tinha de falhar");
    assert_eq!(
        bytes(&documento),
        depois_de_apagar,
        "a recusa não pode deixar metade do desenho movida"
    );

    pilha.undo(&mut documento).expect("desfazer não falha");

    assert_eq!(bytes(&documento), antes);
}
