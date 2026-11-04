#!/usr/bin/env python3
# Caminho relativo: skills/limites-de-uso/scripts/usage-limits.py
"""Lê os limites de uso da conta do agente (janelas de cota) e os expõe ao agente.

Motivação: o percentual de cota consumida costuma existir apenas na barra de
status da CLI e NÃO chega ao contexto do modelo — por isso o agente responde
"não tenho essa visibilidade". Este script recupera o dado de um provedor local
e o devolve num formato que o agente consegue ler.

Agnóstico ao agente por construção: a leitura fica atrás de PROVIDERS, uma lista
de funções que devolvem um dicionário normalizado (ou None, se aquele agente não
estiver instalado). Para suportar outra ferramenta, acrescente um provedor — o
resto do script, a skill e o hook não mudam.

Contrato de runtime (ADR 0004): a única entrada externa é a env var opcional
AGENT_USAGE_DB (caminho alternativo do banco). É validada antes do uso e nunca
há I/O de rede — este script só lê arquivo local, em modo somente-leitura.

Modos:
  usage-limits.py                 texto legível
  usage-limits.py --oneline       uma linha compacta
  usage-limits.py --json          JSON para consumo programático
  usage-limits.py --gate          silencioso; só fala se passar do limiar
                                  (--warn-at N, padrão 80) — feito para hooks
"""

import os
import sys
import json
import time
import sqlite3
from datetime import datetime

# Idade acima da qual o dado não orienta mais decisão: nenhuma sessão renderizou
# a barra de status desde então, logo o percentual pode ter andado sem registro.
STALE_WARN_SECONDS = 15 * 60

# Projeção de ritmo: exige dois pontos na MESMA janela, afastados o bastante
# para a inclinação não ser ruído de arredondamento (o dado vem em passos de 1%).
# 30min é o piso; abaixo disso extrapolar para horas produz número confiante e
# errado — um pico de 10min viraria "estoura em 1h".
BURN_MIN_SPAN_SECONDS = 30 * 60
# Abaixo destes valores a projeção sai marcada como baixa confiança.
BURN_GOOD_SPAN_SECONDS = 90 * 60
BURN_GOOD_SAMPLES = 3

DEFAULT_WARN_AT = 80.0


# --------------------------------------------------------------------------
# Provedores — cada um devolve o dicionário normalizado abaixo, ou None.
#   {"provider": str, "measured_at": int, "windows": {nome: (pct, resets_at)},
#    "db": str | None, "detail": dict}
# --------------------------------------------------------------------------

def _claude_code_db_path():
    override = os.environ.get("AGENT_USAGE_DB", "").strip()
    if override:
        # Validação defensiva do contrato de runtime: recusa placeholder não
        # substituído em vez de tentar abrir um caminho absurdo.
        if "{{" in override:
            return None
        return os.path.expanduser(override)
    return os.path.join(os.path.expanduser("~"), ".claude", "usage", "usage.db")


def provider_claude_code(now):
    """Claude Code: o payload da statusline é persistido em SQLite por um logger
    do usuário (~/.claude/usage/usage-logger.py). Somos apenas leitores."""
    path = _claude_code_db_path()
    if not path or not os.path.isfile(path):
        return None
    try:
        con = sqlite3.connect(f"file:{path}?mode=ro", uri=True, timeout=5)
    except sqlite3.Error:
        return None
    try:
        con.execute("PRAGMA busy_timeout=3000")
        row = con.execute(
            "SELECT last_seen_ts, five_hour_pct, five_hour_resets_at, "
            "       seven_day_pct, seven_day_resets_at, session_id, model_display "
            "FROM sessions "
            "WHERE five_hour_pct IS NOT NULL OR seven_day_pct IS NOT NULL "
            "ORDER BY last_seen_ts DESC LIMIT 1"
        ).fetchone()
        if not row:
            return None
        seen, fh_pct, fh_reset, sd_pct, sd_reset, sid, model = row
        return {
            "provider": "claude-code",
            "measured_at": seen,
            "db": path,
            "detail": {"session_id": sid, "model": model},
            "windows": {"five_hour": (fh_pct, fh_reset),
                        "seven_day": (sd_pct, sd_reset)},
            "burn": {name: _burn_rate(con, pcol, rcol, reset, now)
                     for name, pcol, rcol, reset in (
                         ("five_hour", "five_hour_pct", "five_hour_resets_at", fh_reset),
                         ("seven_day", "seven_day_pct", "seven_day_resets_at", sd_reset))},
        }
    except sqlite3.Error:
        return None
    finally:
        con.close()


def _burn_rate(con, pct_col, reset_col, resets_at, now):
    """Ritmo de consumo (%/hora) e projeção de esgotamento.

    O logger mantém UMA linha por sessão, então uma sessão longa deixa só o seu
    último ponto. Mas sessões distintas dentro da MESMA janela (mesmo resets_at)
    formam uma série temporal utilizável. Devolve None quando os pontos não
    sustentam a conta — melhor não projetar do que projetar errado.
    """
    if not resets_at:
        return None
    try:
        pts = con.execute(
            f"SELECT last_seen_ts, {pct_col} FROM sessions "
            f"WHERE {reset_col}=? AND {pct_col} IS NOT NULL AND last_seen_ts IS NOT NULL "
            f"ORDER BY last_seen_ts", (resets_at,)).fetchall()
    except sqlite3.Error:
        return None
    if len(pts) < 2:
        return None
    (t0, p0), (t1, p1) = pts[0], pts[-1]
    span = t1 - t0
    if span < BURN_MIN_SPAN_SECONDS or p1 <= p0:
        return None
    per_hour = (p1 - p0) / (span / 3600.0)
    if per_hour <= 0:
        return None
    remaining = max(0.0, 100.0 - p1)
    good = span >= BURN_GOOD_SPAN_SECONDS and len(pts) >= BURN_GOOD_SAMPLES
    return {
        "percent_per_hour": round(per_hour, 1),
        "exhausted_in_seconds": int(remaining / per_hour * 3600),
        "samples": len(pts),
        "span_seconds": span,
        "confidence": "media" if good else "baixa",
    }


PROVIDERS = (provider_claude_code,)


# --------------------------------------------------------------------------
# Normalização e formatação
# --------------------------------------------------------------------------

def fmt_dur(seconds):
    s = max(0, int(seconds))
    if s < 60:
        return f"{s}s"
    m, h = (s // 60) % 60, s // 3600
    return f"{h}h{m:02d}m" if h else f"{m}m"


def pill(pct, cells=10):
    if pct is None:
        return "▱" * cells
    filled = max(0, min(cells, int(pct) // (100 // cells)))
    return "▰" * filled + "▱" * (cells - filled)


def build_window(pct, resets_at, burn, now):
    if pct is None:
        return {"status": "unknown"}
    # resets_at no passado: a janela virou e o percentual guardado é de outra.
    expired = bool(resets_at) and resets_at <= now
    resets_in = max(0, resets_at - now) if resets_at else None
    w = {
        "status": "expired" if expired else "ok",
        "used_percentage": round(float(pct), 1),
        "remaining_percentage": round(max(0.0, 100.0 - float(pct)), 1),
        "resets_at_iso": datetime.fromtimestamp(resets_at).isoformat(timespec="minutes")
        if resets_at else None,
        "resets_in_seconds": resets_in,
    }
    if burn and not expired:
        w["burn"] = burn
        # O que decide: o teto chega antes ou depois do reset?
        w["exhausts_before_reset"] = (
            resets_in is not None and burn["exhausted_in_seconds"] < resets_in)
    return w


def collect():
    now = int(time.time())
    raw = None
    for provider in PROVIDERS:
        try:
            raw = provider(now)
        except Exception:
            raw = None
        if raw:
            break

    if not raw:
        return {
            "status": "no_data",
            "generated_at": datetime.fromtimestamp(now).isoformat(timespec="seconds"),
            "note": "Nenhum provedor de limites disponível nesta máquina. No Claude Code, "
                    "exige o logger de statusline gravando ~/.claude/usage/usage.db.",
        }

    age = now - (raw["measured_at"] or now)
    burn = raw.get("burn") or {}
    out = {
        "status": "stale" if age > STALE_WARN_SECONDS else "ok",
        "generated_at": datetime.fromtimestamp(now).isoformat(timespec="seconds"),
        "measured_at_iso": datetime.fromtimestamp(raw["measured_at"]).isoformat(timespec="seconds")
        if raw["measured_at"] else None,
        "age_seconds": age,
        "source": {"provider": raw["provider"], "db": raw.get("db"), **raw.get("detail", {})},
    }
    for name, (pct, reset) in raw["windows"].items():
        out[name] = build_window(pct, reset, burn.get(name), now)
    return out


LABELS = {"five_hour": "Janela 5h", "seven_day": "Janela 7d"}


def render_window(name, w):
    label = LABELS.get(name, name)
    if w.get("status") == "unknown":
        return f"  {label}: sem dado"
    pct = w["used_percentage"]
    over = " (ESTOURADO)" if pct >= 100 else ""
    if w["status"] == "expired":
        return f"  {label}: {pill(pct)} {pct}%{over} — janela já reiniciou; valor obsoleto"
    line = f"  {label}: {pill(pct)} {pct}%{over}"
    if w.get("resets_in_seconds") is not None:
        line += f" · reinicia em {fmt_dur(w['resets_in_seconds'])} ({w['resets_at_iso']})"
    b = w.get("burn")
    if b:
        line += (f"\n           ritmo ~{b['percent_per_hour']}%/h → teto em "
                 f"{fmt_dur(b['exhausted_in_seconds'])}")
        line += " — ANTES do reset" if w.get("exhausts_before_reset") else " — depois do reset"
        line += (f" (confianca {b['confidence']}: {b['samples']} amostras em "
                 f"{fmt_dur(b['span_seconds'])})")
    return line


def worst(data):
    """Maior percentual entre as janelas válidas, e qual é ela."""
    top, name = None, None
    for key in ("five_hour", "seven_day"):
        w = data.get(key) or {}
        if w.get("status") != "ok":
            continue
        if top is None or w["used_percentage"] > top:
            top, name = w["used_percentage"], key
    return top, name


def main():
    args = sys.argv[1:]
    flags = set(a for a in args if a.startswith("--"))

    warn_at = DEFAULT_WARN_AT
    if "--warn-at" in args:
        try:
            warn_at = float(args[args.index("--warn-at") + 1])
        except (IndexError, ValueError):
            pass
    else:
        env_warn = os.environ.get("AGENT_USAGE_WARN_AT", "").strip()
        if env_warn:
            try:
                warn_at = float(env_warn)
            except ValueError:
                pass

    data = collect()

    # --- modo hook: silêncio é a saída esperada na esmagadora maioria das vezes
    if "--gate" in flags:
        if data["status"] == "no_data":
            return
        top, name = worst(data)
        if top is None or top < warn_at:
            return
        w = data[name]
        msg = (f"[limites] {LABELS[name]} em {top}% da cota da conta"
               f" (reinicia em {fmt_dur(w['resets_in_seconds'])}).")
        if w.get("exhausts_before_reset") and w["burn"]["confidence"] != "baixa":
            msg += (f" No ritmo atual (~{w['burn']['percent_per_hour']}%/h) a cota acaba em"
                    f" ~{fmt_dur(w['burn']['exhausted_in_seconds'])}, ANTES do reset.")
        if data["status"] == "stale":
            msg += f" (dado de {fmt_dur(data['age_seconds'])} atrás)"
        msg += " Antes de iniciar trabalho longo, consulte a skill limites-de-uso."
        print(msg)
        return

    if "--json" in flags:
        print(json.dumps(data, ensure_ascii=False, indent=2))
        return

    if data["status"] == "no_data":
        print("Limites de uso: " + data["note"])
        return

    if "--oneline" in flags:
        parts = []
        for key in ("five_hour", "seven_day"):
            w = data.get(key) or {}
            if w.get("status") == "unknown":
                continue
            r = (f" (reset {fmt_dur(w['resets_in_seconds'])})"
                 if w.get("resets_in_seconds") else "")
            parts.append(f"{'5h' if key == 'five_hour' else '7d'} {w['used_percentage']}%{r}")
        tail = f"  [dado de {fmt_dur(data['age_seconds'])} atrás]" if data["status"] == "stale" else ""
        print("Limites da conta: " + " · ".join(parts) + tail)
        return

    print(f"Limites de uso da conta ({data['source']['provider']})")
    for key in ("five_hour", "seven_day"):
        if key in data:
            print(render_window(key, data[key]))
    print(f"  Medido em {data['measured_at_iso']} (há {fmt_dur(data['age_seconds'])})")
    if data["status"] == "stale":
        print("  AVISO: nenhuma sessão atualizou a barra de status recentemente; "
              "o percentual pode estar defasado.")


if __name__ == "__main__":
    try:
        main()
    except Exception:
        # Nunca quebrar quem chama — em modo --gate isso rodaria a cada prompt.
        if "--gate" not in sys.argv:
            print("Limites de uso: indisponível.")
