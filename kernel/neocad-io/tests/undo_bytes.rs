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

use neocad_geometry::{
    point_on_circle, Arc, Circle, CornerError, Line, Point2, Polyline, PolylineVertex, TrimError,
};
use neocad_io::{write_dxf, DxfContents, EntitySpace, ReadEntity};
use neocad_model::{Document, Entity, EntityId, Geometry, Text};
use neocad_transaction::commands::CommandError;
use neocad_transaction::{commands, CommandStack, TransactionError};

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
            Box::new(|editor, ids| {
                // A escala devolve o erro do comando, que pode recusar o fator;
                // aqui o fator é válido, então o único erro possível é o do
                // documento.
                commands::scale(editor, ids, Point2::new(-1.0, 2.0), 1.75).map_err(
                    |erro| match erro {
                        CommandError::Document(erro) => erro,
                        outro => panic!("fator válido recusado: {outro}"),
                    },
                )
            }),
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

#[test]
fn a_paralela_desfeita_devolve_os_mesmos_bytes() {
    let (mut documento, ids) = desenho();
    let antes = bytes(&documento);
    let quantas_antes = documento.entity_count();
    let mut pilha = CommandStack::new();

    // As quatro primeiras formas são curvas; a quinta é texto e não tem
    // paralela.
    for id in ids.iter().take(4) {
        let criadas = pilha
            .edit(&mut documento, "Paralela", |editor| {
                commands::offset(editor, *id, 0.5, Point2::new(100.0, 100.0))
            })
            .expect("paralela de curva não falha");

        assert!(!criadas.is_empty(), "a paralela precisa produzir algo");
    }

    assert!(
        documento.entity_count() > quantas_antes,
        "paralelar acrescenta sem tirar o original"
    );

    for _ in 0..4 {
        assert!(
            pilha.undo(&mut documento).expect("desfazer não falha"),
            "cada paralela é um passo de desfazer"
        );
    }

    assert_eq!(
        bytes(&documento),
        antes,
        "as paralelas desfeitas não devolveram os mesmos bytes"
    );
}

#[test]
fn texto_nao_tem_paralela_e_o_desenho_nao_muda() {
    let (mut documento, ids) = desenho();
    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();

    let resultado = pilha.edit(&mut documento, "Paralela", |editor| {
        commands::offset(editor, ids[4], 1.0, Point2::new(100.0, 100.0))
    });

    assert!(
        matches!(
            resultado,
            Err(TransactionError::Command(CommandError::NotACurve))
        ),
        "obtido {resultado:?}"
    );
    assert_eq!(bytes(&documento), antes, "a recusa não deixa rastro");
    assert!(!pilha.can_undo(), "recusa não vira passo de desfazer");
}

#[test]
fn escalar_recusa_fator_zero_negativo_ou_nao_finito() {
    for fator in [
        0.0,
        -0.0,
        -1.0,
        -2.5,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let (mut documento, ids) = desenho();
        let antes = bytes(&documento);
        let mut pilha = CommandStack::new();
        let resultado = pilha.edit(&mut documento, "Escalar", |editor| {
            commands::scale(editor, &ids, Point2::new(0.0, 0.0), fator)
        });
        assert!(
            resultado.is_err(),
            "escalar com fator {fator} tinha de ser recusado"
        );
        assert_eq!(
            bytes(&documento),
            antes,
            "escalar com fator {fator} não podia alterar o documento"
        );
        assert!(
            matches!(pilha.undo(&mut documento), Ok(false)),
            "escalar com fator {fator} não podia deixar nada para desfazer (undo devia devolver Ok(false))"
        );
    }

    // Controle: um fator positivo válido continua sendo aceito e altera o documento.
    let (mut documento, ids) = desenho();
    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();
    let resultado = pilha.edit(&mut documento, "Escalar", |editor| {
        commands::scale(editor, &ids, Point2::new(0.0, 0.0), 2.0)
    });
    assert!(
        resultado.is_ok(),
        "escalar com fator 2.0 tinha de ser aceito"
    );
    assert_ne!(
        bytes(&documento),
        antes,
        "escalar com fator 2.0 tinha de alterar os bytes do documento"
    );
}

// Aparar e estender (spec 05, E7–E9): testes escritos em contexto limpo a partir do contrato.

#[test]
fn aparar_reta_entre_dois_cortes_e_desfazer() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_reta = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 0.0),
            }),
        ))
        .unwrap();
    let id_corte1 = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(3.0, -5.0),
                end: Point2::new(3.0, 5.0),
            }),
        ))
        .unwrap();
    let id_corte2 = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(7.0, -5.0),
                end: Point2::new(7.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let mut pilha = CommandStack::new();
    let antes = bytes(&documento);
    let result = pilha.edit(&mut documento, "Aparar", |editor| {
        commands::trim(
            editor,
            id_reta,
            &[id_corte1, id_corte2],
            Point2::new(5.0, 0.0),
            0.5,
        )
    });
    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], id_reta);

    let entity1 = documento.entity(ids[0]).unwrap();
    match &entity1.geometry {
        Geometry::Line(line) => {
            assert!((line.start.x - 0.0).abs() < 1e-9);
            assert!((line.start.y - 0.0).abs() < 1e-9);
            assert!((line.end.x - 3.0).abs() < 1e-9);
            assert!((line.end.y - 0.0).abs() < 1e-9);
        }
        _ => panic!("Esperava Line"),
    }

    let entity2 = documento.entity(ids[1]).unwrap();
    match &entity2.geometry {
        Geometry::Line(line) => {
            assert!((line.start.x - 7.0).abs() < 1e-9);
            assert!((line.start.y - 0.0).abs() < 1e-9);
            assert!((line.end.x - 10.0).abs() < 1e-9);
            assert!((line.end.y - 0.0).abs() < 1e-9);
        }
        _ => panic!("Esperava Line"),
    }

    assert_eq!(documento.entity_count(), 4);

    assert!(pilha.undo(&mut documento).unwrap());
    let bytes_apos_undo = bytes(&documento);
    assert_eq!(antes, bytes_apos_undo);
}

#[test]
fn aparar_reta_fora_da_curva() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_reta = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 0.0),
            }),
        ))
        .unwrap();
    let id_corte1 = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(3.0, -5.0),
                end: Point2::new(3.0, 5.0),
            }),
        ))
        .unwrap();
    let id_corte2 = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(7.0, -5.0),
                end: Point2::new(7.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let mut pilha = CommandStack::new();
    let antes = bytes(&documento);
    let result = pilha.edit(&mut documento, "Aparar", |editor| {
        commands::trim(
            editor,
            id_reta,
            &[id_corte1, id_corte2],
            Point2::new(5.0, 3.0),
            0.5,
        )
    });
    assert!(result.is_err());
    match result.unwrap_err() {
        TransactionError::Command(CommandError::Trim(TrimError::PickOffCurve)) => {}
        _ => panic!("Esperava PickOffCurve"),
    }

    assert_eq!(documento.entity_count(), 3);
    assert!(!pilha.can_undo());
    let bytes_apos_undo = bytes(&documento);
    assert_eq!(antes, bytes_apos_undo);
}

#[test]
fn estender_reta_com_fronteira() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_reta = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(5.0, 0.0),
            }),
        ))
        .unwrap();
    let id_fronteira = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(10.0, -5.0),
                end: Point2::new(10.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let mut pilha = CommandStack::new();
    let antes = bytes(&documento);
    let result = pilha.edit(&mut documento, "Estender", |editor| {
        commands::extend(editor, id_reta, &[id_fronteira], Point2::new(4.0, 0.0), 0.5)
    });
    assert!(result.is_ok());

    let entity = documento.entity(id_reta).unwrap();
    match &entity.geometry {
        Geometry::Line(line) => {
            assert!((line.start.x - 0.0).abs() < 1e-9);
            assert!((line.start.y - 0.0).abs() < 1e-9);
            assert!((line.end.x - 10.0).abs() < 1e-9);
            assert!((line.end.y - 0.0).abs() < 1e-9);
        }
        _ => panic!("Esperava Line"),
    }

    assert_eq!(documento.entity_count(), 2);

    assert!(pilha.undo(&mut documento).unwrap());
    let bytes_apos_undo = bytes(&documento);
    assert_eq!(antes, bytes_apos_undo);
}

#[test]
fn estender_reta_sem_fronteira() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_reta = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(5.0, 0.0),
            }),
        ))
        .unwrap();
    let id_fronteira = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(10.0, 1.0),
                end: Point2::new(10.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let mut pilha = CommandStack::new();
    let antes = bytes(&documento);
    let result = pilha.edit(&mut documento, "Estender", |editor| {
        commands::extend(editor, id_reta, &[id_fronteira], Point2::new(4.0, 0.0), 0.5)
    });
    assert!(result.is_err());
    match result.unwrap_err() {
        TransactionError::Command(CommandError::Trim(TrimError::NoBoundary)) => {}
        _ => panic!("Esperava NoBoundary"),
    }

    assert_eq!(documento.entity_count(), 2);
    assert!(!pilha.can_undo());
    let bytes_apos_undo = bytes(&documento);
    assert_eq!(antes, bytes_apos_undo);
}

#[test]
fn aparar_texto_erro_nao_curva() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_texto = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Text(Text {
                position: Point2::new(0.0, 0.0),
                content: "A".to_owned(),
                height: 2.5,
                rotation: 0.0,
            }),
        ))
        .unwrap();
    let id_corte = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(3.0, -5.0),
                end: Point2::new(3.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let mut pilha = CommandStack::new();
    let antes = bytes(&documento);
    let result = pilha.edit(&mut documento, "Aparar", |editor| {
        commands::trim(editor, id_texto, &[id_corte], Point2::new(1.0, 0.0), 0.5)
    });
    assert!(result.is_err());
    match result.unwrap_err() {
        TransactionError::Command(CommandError::NotACurve) => {}
        _ => panic!("Esperava NotACurve"),
    }

    assert_eq!(documento.entity_count(), 2);
    assert!(!pilha.can_undo());
    let bytes_apos_undo = bytes(&documento);
    assert_eq!(antes, bytes_apos_undo);
}

// Concordar e chanfrar (spec 05, E10–E13): testes escritos em contexto limpo a partir do contrato.

fn confere_reta(geometria: &Geometry, inicio: Point2, fim: Point2) {
    let Geometry::Line(line) = geometria else {
        panic!("esperava reta, veio {geometria:?}");
    };
    assert!(
        line.start.distance_to(inicio) < 1e-9 && line.end.distance_to(fim) < 1e-9,
        "reta {line:?}, esperada {inicio:?} → {fim:?}"
    );
}

fn confere_arco(geometria: &Geometry, centro: Point2, raio: f64, inicio: Point2, fim: Point2) {
    let Geometry::Arc(arc) = geometria else {
        panic!("esperava arco, veio {geometria:?}");
    };
    let a = point_on_circle(arc.center, arc.radius, arc.start_angle);
    let b = point_on_circle(arc.center, arc.radius, arc.end_angle);
    assert!(
        arc.center.distance_to(centro) < 1e-9
            && (arc.radius - raio).abs() < 1e-9
            && a.distance_to(inicio) < 1e-9
            && b.distance_to(fim) < 1e-9,
        "arco {arc:?}, esperado centro {centro:?}, raio {raio}, {inicio:?} → {fim:?}"
    );
}

#[test]
fn concordar_com_raio_2() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_a = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 0.0),
            }),
        ))
        .unwrap();
    let id_b = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(10.0, 0.0),
                end: Point2::new(10.0, 10.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();
    let resultado = pilha.edit(&mut documento, "Concordar", |editor| {
        commands::fillet(
            editor,
            id_a,
            Point2::new(2.0, 0.0),
            id_b,
            Point2::new(10.0, 8.0),
            2.0,
        )
    });
    assert!(resultado.is_ok());
    let novo_id = resultado.unwrap().unwrap();

    assert_eq!(documento.entity_count(), 3);
    confere_reta(
        &documento.entity(id_a).unwrap().geometry,
        Point2::new(0.0, 0.0),
        Point2::new(8.0, 0.0),
    );
    confere_reta(
        &documento.entity(id_b).unwrap().geometry,
        Point2::new(10.0, 2.0),
        Point2::new(10.0, 10.0),
    );
    confere_arco(
        &documento.entity(novo_id).unwrap().geometry,
        Point2::new(8.0, 2.0),
        2.0,
        Point2::new(8.0, 0.0),
        Point2::new(10.0, 2.0),
    );

    assert!(pilha.undo(&mut documento).unwrap());
    assert_eq!(bytes(&documento), antes);
}

#[test]
fn chanfrar_com_distancias_2_3() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_a = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 0.0),
            }),
        ))
        .unwrap();
    let id_b = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(10.0, 0.0),
                end: Point2::new(10.0, 10.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();
    let resultado = pilha.edit(&mut documento, "Chanfrar", |editor| {
        commands::chamfer(
            editor,
            id_a,
            Point2::new(2.0, 0.0),
            id_b,
            Point2::new(10.0, 8.0),
            (2.0, 3.0),
        )
    });
    assert!(resultado.is_ok());
    let novo_id = resultado.unwrap().unwrap();

    assert_eq!(documento.entity_count(), 3);
    confere_reta(
        &documento.entity(id_a).unwrap().geometry,
        Point2::new(0.0, 0.0),
        Point2::new(8.0, 0.0),
    );
    confere_reta(
        &documento.entity(id_b).unwrap().geometry,
        Point2::new(10.0, 3.0),
        Point2::new(10.0, 10.0),
    );
    confere_reta(
        &documento.entity(novo_id).unwrap().geometry,
        Point2::new(8.0, 0.0),
        Point2::new(10.0, 3.0),
    );

    assert!(pilha.undo(&mut documento).unwrap());
    assert_eq!(bytes(&documento), antes);
}

#[test]
fn concordar_paralelas_erro_parallel_lines() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_a = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(10.0, 0.0),
            }),
        ))
        .unwrap();
    let id_b = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(0.0, 5.0),
                end: Point2::new(10.0, 5.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();
    let resultado = pilha.edit(&mut documento, "Concordar", |editor| {
        commands::fillet(
            editor,
            id_a,
            Point2::new(2.0, 0.0),
            id_b,
            Point2::new(2.0, 5.0),
            1.0,
        )
    });
    assert!(resultado.is_err());
    assert!(matches!(
        resultado.unwrap_err(),
        TransactionError::Command(CommandError::Corner(CornerError::ParallelLines))
    ));

    assert_eq!(documento.entity_count(), 2);
    assert!(!pilha.can_undo());
    assert_eq!(bytes(&documento), antes);
}

#[test]
fn concordar_texto_erro_not_a_curve() {
    let mut documento = Document::new();
    let camada = documento.layers().default_layer();
    let mut editor = documento.edit();
    let id_texto = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Text(Text {
                position: Point2::new(0.0, 0.0),
                content: "A".to_owned(),
                height: 2.5,
                rotation: 0.0,
            }),
        ))
        .unwrap();
    let id_reta = editor
        .insert_in_model_space(Entity::new(
            camada,
            Geometry::Line(Line {
                start: Point2::new(10.0, 0.0),
                end: Point2::new(10.0, 10.0),
            }),
        ))
        .unwrap();
    let _ = editor.finish();

    let antes = bytes(&documento);
    let mut pilha = CommandStack::new();
    let resultado = pilha.edit(&mut documento, "Concordar", |editor| {
        commands::fillet(
            editor,
            id_texto,
            Point2::new(0.0, 0.0),
            id_reta,
            Point2::new(10.0, 8.0),
            1.0,
        )
    });
    assert!(resultado.is_err());
    assert!(matches!(
        resultado.unwrap_err(),
        TransactionError::Command(CommandError::NotACurve)
    ));

    assert_eq!(documento.entity_count(), 2);
    assert!(!pilha.can_undo());
    assert_eq!(bytes(&documento), antes);
}
