Leia este documento em [inglês (EN)](CHANGELOG.md).


# Changelog

Todas as mudanças notáveis deste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/spec/v2.0.0.html).

## [2.0.1] - 2026-04-19

### Adicionado

- Aliases de flags para compatibilidade retroativa com a documentação bilíngue: `rename --old/--new` (aliases de `--name/--new-name`), `link/unlink --source/--target` (aliases de `--from/--to`), `related --hops` (alias de `--max-hops`), `sync-safe-copy --output` (alias de `--dest`).
- `related` agora aceita o nome da memória como argumento posicional.
- `--json` aceito como no-op em `health`, `stats`, `migrate`, `namespace-detect` (o output já é JSON por default).
- Flag global `--lang en|pt` com fallback via env var `NEUROGRAPHRAG_LANG` e locale `LC_ALL`/`LANG` para mensagens de progresso no stderr.
- Novo módulo `i18n` com enum `Language` e helpers `init`/`current`/`tr`. Helpers bilíngues em `output::emit_progress_i18n`.
- Timestamps ISO 8601: `created_at_iso` em `RememberResponse`, `updated_at_iso` em itens de `list`, `created_at_iso`/`updated_at_iso` em `read`. Todos paralelos aos inteiros epoch existentes (sem breaking).
- Resposta `read` agora inclui `memory_id` (alias de `id`), `type` (alias de `memory_type`), `version` (para controle otimista).
- Itens `hybrid-search` agora incluem `score` (alias de `combined_score`) e `source: "hybrid"`.
- Itens `list` agora incluem `memory_id` (alias de `id`).
- Resposta `stats` agora inclui `memories_total`, `entities_total`, `relationships_total`, `chunks_total`, `db_bytes` (aliases de campos existentes) para conformidade com contrato.
- Resposta `health` agora inclui `schema_version` no topo e `missing_entities[]` conforme PRD.
- `RememberResponse` inclui `operation` (alias de `action`), `created_at`, `created_at_iso`.
- `RecallResponse` inclui `results[]` (merge de `direct_matches` + `graph_matches`) conforme contrato SKILL.md.
- Flag `init --namespace` adicionada, resolvida e ecoada em `InitResponse.namespace`.
- Flag `recall --min-distance <float>` (default 1.0, desativada). Quando < 1.0, retorna exit 4 se todos os hits excederem o threshold.

### Corrigido

- Arquivos DB e snapshot criados por `open_rw` e `sync-safe-copy` agora recebem chmod 600 em Unix para prevenir vazamento em montagens compartilhadas (Dropbox, NFS, `/tmp` multi-usuário).
- Mensagens de progresso em `remember`, `recall`, `hybrid-search`, `init` agora usam o helper bilíngue e respeitam o idioma ativo de forma consistente (antes misturavam EN/PT na mesma sessão).

### Documentação

- COOKBOOK, AGENT_PROTOCOL, SKILL, CLAUDE.md, README, INTEGRATIONS e llms.txt atualizados para refletir schemas reais, flags reais e exit codes reais. Validados contra o output de `--help` de cada subcomando.
- Subcomandos `graph` e `cleanup-orphans` agora documentados nos guias apropriados.
- Disclaimer honesto de latência adicionado: recall e hybrid-search levam ~1s por invocação no modo CLI one-shot porque o modelo fastembed recarrega a cada processo; ~8ms requer daemon (planejado para v3.0.0 Tier 4).

## [2.0.0] - 2026-04-18

### Breaking

- Exit code `DbBusy` movido de 13 → 15 para liberar exit 13 para `BatchPartialFailure` conforme PRD. Scripts shell que detectavam `EX_UNAVAILABLE` (13) como DB busy agora devem checar 15.
- Formato JSON da resposta de `hybrid-search` remodelado de `{query, combined_rank[], vec_rank[], fts_rank[]}` para `{query, k, results: [{memory_id, name, namespace, type, description, body, combined_score, vec_rank?, fts_rank?}], graph_matches: []}` conforme PRD linhas 771-787. Consumidores que parseavam `combined_rank` devem migrar para `results`.
- `purge --older-than-seconds` descontinuada em favor de `--retention-days`. A flag antiga permanece como alias oculto mas emite warning. Será removida em v3.0.0.
- `NAME_SLUG_REGEX` mais estrita que `SLUG_REGEX` da v1.x: nomes multichar devem começar com letra (requisito do PRD). Single-char `[a-z0-9]` ainda permitido. Memórias existentes com nomes iniciando em dígito passam inalteradas, mas `rename` para nomes estilo legado agora falhará.

### Adicionado

- `AppError::BatchPartialFailure { total, failed }` mapeando para exit 13 — reservado para `import`, `reindex` e batch stdin (entrando em Tier 3/4).
- Constantes em `src/constants.rs`: `PURGE_RETENTION_DAYS_DEFAULT=90`, `MAX_NAMESPACES_ACTIVE=100`, `EMBEDDING_MAX_TOKENS=512`, `K_GRAPH_MATCHES_LIMIT=20`, `K_LIST_DEFAULT_LIMIT=100`, `K_GRAPH_ENTITIES_DEFAULT_LIMIT=50`, `K_RELATED_DEFAULT_LIMIT=10`, `K_HISTORY_DEFAULT_LIMIT=20`, `WEIGHT_VEC_DEFAULT=1.0`, `WEIGHT_FTS_DEFAULT=1.0`, `TEXT_BODY_PREVIEW_LEN=200`, `ORT_NUM_THREADS_DEFAULT="1"`, `ORT_INTRA_OP_NUM_THREADS_DEFAULT="1"`, `OMP_NUM_THREADS_DEFAULT="1"`, `BATCH_PARTIAL_FAILURE_EXIT_CODE=13`, `DB_BUSY_EXIT_CODE=15`.
- Flag `--dry-run` e `--retention-days` em `purge`.
- Campos `namespace` e `merged_into_memory_id: Option<i64>` em `RememberResponse`.
- Campo `k: usize` em `RecallResponse`.
- Campos `bytes_freed: i64`, `oldest_deleted_at: Option<i64>`, `retention_days_used: u32`, `dry_run: bool` em `PurgeResponse`.
- Flag `--format` em `hybrid-search` (apenas JSON; text/markdown reservados para Tier 2).
- Flag `--expected-updated-at` (optimistic locking) em `rename` e `restore`.
- Guard de limite de namespaces ativos (`MAX_NAMESPACES_ACTIVE=100`) em `remember` — retorna exit 5 quando excedido.

### Alterado

- `SLUG_REGEX` renomeada para `NAME_SLUG_REGEX` com valor conforme PRD `r"^[a-z][a-z0-9-]{0,78}[a-z0-9]$|^[a-z0-9]$"`. Nomes multichar devem começar com letra.

### Corrigido

- Prefixo `__` explicitamente rejeitado em `rename` (antes apenas aplicado em `remember` como efeito colateral da regex).
- Constantes fantasma na fórmula RRF (`WEIGHT_VEC_DEFAULT`, `WEIGHT_FTS_DEFAULT`) agora declaradas em `constants.rs` — referências do PRD agora mapeiam símbolos reais.


## [1.2.1] - 2026-04-18

### Corrigido

- Falha de instalação em versões de `rustc` no intervalo `1.88..1.95` causada pela dependência transitiva `constant_time_eq 0.4.3` (puxada via `blake3`) elevando seu MSRV para 1.95.0 em uma patch release
- `cargo install neurographrag` sem `--locked` agora sucede porque o pin direto `constant_time_eq = "=0.4.2"` força versão resolvida compatível com o `rust-version = "1.88"` declarado

### Alterado

- `Cargo.toml` agora declara pin preventivo explícito `constant_time_eq = "=0.4.2"` com comentário inline documentando a razão do drift de MSRV; o pin será revisitado quando elevarmos `rust-version` para 1.95
- Instruções de instalação do `README.md` (EN e PT) atualizadas de `cargo install neurographrag` para `cargo install --locked neurographrag`, incluindo bullet explicando a motivação

### Adicionado

- Seção `docs_rules/prd.md` "Dependency MSRV Drift Protection" documentando o padrão canônico de mitigação — pinagem direta de dependências transitivas problemáticas no `Cargo.toml` de nível superior


## [1.2.0] - 2026-04-18

### Adicionado

- Semáforo de contagem cross-process com até 4 slots simultâneos via `src/lock.rs` (`acquire_cli_slot`)
- Memory guard abortando com exit 77 quando RAM livre está abaixo de 2 GB via `sysinfo` (`src/memory_guard.rs`)
- Signal handler graceful para SIGINT, SIGTERM e SIGHUP via `ctrlc` com feature `termination`
- Flag `--max-concurrency <N>` para controlar limite de invocações paralelas em runtime
- Flag oculta `--skip-memory-guard` para testes automatizados onde a alocação real não ocorre
- Constantes `MAX_CONCURRENT_CLI_INSTANCES`, `MIN_AVAILABLE_MEMORY_MB`, `CLI_LOCK_DEFAULT_WAIT_SECS`, `EMBEDDING_LOAD_EXPECTED_RSS_MB` e `LOW_MEMORY_EXIT_CODE` em `src/constants.rs`
- Variantes `AppError::AllSlotsFull` e `AppError::LowMemory` com mensagens em português brasileiro
- Global `SHUTDOWN: AtomicBool` e função `shutdown_requested()` em `src/lib.rs`

### Alterado

- Default da flag `--wait-lock` aumentado para 300 segundos (5 minutos) via `CLI_LOCK_DEFAULT_WAIT_SECS`
- Lock file migrado de `cli.lock` único para `cli-slot-{N}.lock` (semáforo de contagem N=1..4)

### Removido

- BREAKING — flag `--allow-parallel` removida — causou OOM crítico em produção (incidente 2026-04-18)

### Corrigido

- Bug crítico onde múltiplas invocações CLI simultâneas esgotavam a RAM do sistema após 58 invocações paralelas travarem o computador por 38 minutos (incidente 2026-04-18)


## [Unreleased]

### Adicionado

- Flags globais `--allow-parallel` e `--wait-lock SECONDS` para concorrência controlada
- Módulo `src/lock.rs` implementando lock single-instance baseado em arquivo via `fs4`
- Nova variante `AppError::LockBusy` mapeando para exit code 75 (`EX_TEMPFAIL`)
- Variáveis de ambiente `ORT_NUM_THREADS`, `OMP_NUM_THREADS` e `ORT_INTRA_OP_NUM_THREADS` pré-definidas para 1 quando não já definidas pelo usuário
- Singleton `OnceLock<Mutex<TextEmbedding>>` para reuso do modelo intra-processo
- Testes de integração em `tests/lock_integration.rs` cobrindo aquisição e liberação de lock

### Alterado

- Comportamento padrão agora é single-instance — uma segunda invocação concorrente sai com código 75 exceto se `--allow-parallel` for passada
- Módulo embedder refatorado de struct-com-estado para funções livres operando sobre um singleton

### Corrigido

- Previne OOM livelock quando a CLI é invocada em paralelismo massivo por orquestradores LLM (incidente 2026-04-18)


## [0.1.0] - 2026-04-17

### Adicionado

- Fase 1 — Fundação: schema SQLite com vec0 (sqlite-vec), FTS5, grafo de entidades
- Fase 2 — Subcomandos essenciais: init, remember, recall, read, list, forget, rename, edit, history, restore, health, stats, optimize, purge, vacuum, migrate, hybrid-search, namespace-detect, sync-safe-copy

### Corrigido

- Bug de corrupção FTS5 external-content no ciclo forget+purge (removido DELETE manual em forget.rs)

### Alterado

- MSRV elevado de 1.80 para 1.88 (exigido por dependências transitivas base64ct 1.8.3, ort-sys, time)

[Unreleased]: https://github.com/daniloaguiarbr/neurographrag/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v0.1.0
