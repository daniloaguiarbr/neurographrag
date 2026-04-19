Leia este documento em [inglês (EN)](CHANGELOG.md).


# Changelog

Todas as mudanças notáveis deste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/spec/v2.0.0.html).

## [2.1.0] - 2026-04-19

### Documentação

- Adicionada seção "Crates Rust Compatíveis" ao README listando 18 crates Rust
- Adicionados 6 agentes de IA ausentes em docs/AGENTS.md (Minimax, Z.ai, Ollama, Hermes, LangChain, LangGraph)
- Adicionada cobertura do shell Nushell em INTEGRATIONS
- Adicionadas 5 receitas novas ao docs/COOKBOOK cobrindo integrações com crates Rust
- Adicionados LICENSE-MIT e LICENSE-APACHE como arquivos separados
- Adicionado llms.pt-BR.txt espelhado para descoberta por LLMs em português
- Adicionado schema canônico evals/queries.json em skill/
- Reescritos README, docs/AGENTS, INTEGRATIONS, llms.txt com framework AIDA
- Removidas 59 violações de em-dash fora de headings H3
- Removidos 4 marcadores de negrito do CHANGELOG
- Divididos 36 bullets longos no CHANGELOG EN e PT em declarações concisas

### Alterado

- Entradas do CHANGELOG abrem com verbos persuasivos e benefícios quantificados
- Toda a documentação agora segue docs_rules/rules_rust_documentacao.md


## [2.0.3] - 2026-04-19

### Adicionado
- `purge --days` aceito como alias de `--retention-days` para compatibilidade com docs (GAP 3)
- `recall --json` e `hybrid-search --json` aceitos como no-op (GAP 6) — saída JSON já é o padrão
- JSON de `health` agora inclui `wal_size_mb` e `journal_mode` (GAP 7)
- JSON de `stats` agora inclui `edges` (alias de `relationships`) e `avg_body_len` (GAP 8)
- Variantes de `AppError` agora localizadas via enum `Idioma` / match exaustivo de `Mensagem` (GAP 13) — `--lang en/pt` aplica-se também às mensagens de erro
- 8 novas seções em HOW_TO_USE.md para subcomandos sem documentação prévia (GAP 12): cleanup-orphans, edit, graph, history, namespace-detect, rename, restore, unlink
- Espelho bilíngue HOW_TO_USE.pt-BR.md
- Aviso de latência no COOKBOOK informando ~1s por invocação CLI vs planos do daemon (GAP P1)

### Alterado
- Toda a documentação: `--type agent` substituído por `--type project` (GAP 1) — PRD define 7 tipos válidos (user/feedback/project/reference/decision/incident/skill); `agent` nunca foi válido
- Toda a documentação: `purge --days` reescrito como `purge --retention-days` (GAP 3)
- Toda a documentação: exemplos de `remember` agora incluem `--description "..."` (GAP 2)
- README, CLAUDE, AGENT_PROTOCOL: contagem de agentes padronizada em 27 (GAP 14)
- Schemas AGENTS.md: raiz JSON de `recall` documentada como `direct_matches[]/graph_matches[]/results[]` (conforme PRD), `hybrid-search` como `results[]` com `vec_rank/fts_rank` (GAPs 4, 5)
- Padrões do COOKBOOK corrigidos: recall --k 10, list --limit 50, pesos hybrid-search 1.0/1.0, purge --retention-days 90 (GAPs 28-31)
- Nota em docs sobre `distance` (cosseno, menor=melhor) vs `score` (1-distance, maior=melhor) em JSON vs text/markdown (GAP 17)
- Nota em docs sobre namespace padrão `global` (não `default`) (GAP 16)

### Corrigido
- Binário não retorna mais exit 2 para `purge --days 30` (GAP 3)
- Binário não retorna mais exit 2 para `recall --json "q"` (GAP 6)
- Documentação de `link` agora explicita pré-requisito de entidade (GAP 9)
- Documentação da flag `--force-merge` (GAP 18)
- Documentação de `graph --format dot|mermaid` (GAP 22)
- Documentação da flag `--db <PATH>` (GAP 25)
- Documentação de `--max-concurrency` limitado a 2×nCPUs (GAP 27)

### Documentação
- `27 agentes de IA` padronizado como contagem oficial em todo o projeto
- Evidência: plano de testes de 2026-04-19 catalogou 31 gaps em `/tmp/neurographrag-testplan-v2.0.2/gaps.md`; v2.0.3 fecha todos os 31
- GAP 11 `elapsed_ms` universal em JSON adiado para v2.1.0 (requer captura de processing_time em todos os comandos)
- GAP P1 latência < 50ms requer modo daemon planejado para v3.0.0


## [2.0.2] - 2026-04-19

### Corrigido

- Flag `--lang` agora aceita os códigos curtos `en`/`pt` conforme documentado.
- Antes exigia identificadores completos `english`/`portugues`; aliases adicionados: `en/english/EN`, `pt/portugues/portuguese/pt-BR/pt-br/PT`.


## [2.0.1] - 2026-04-19

### Adicionado

- Aliases de flags para compatibilidade retroativa com a documentação bilíngue.
- `rename --old/--new` adicionados como aliases de `--name/--new-name`.
- `link/unlink --source/--target` adicionados como aliases de `--from/--to`.
- `related --hops` adicionado como alias de `--max-hops`.
- `sync-safe-copy --output` adicionado como alias de `--dest`.
- `related` agora aceita o nome da memória como argumento posicional.
- `--json` aceito como no-op em `health`, `stats`, `migrate`, `namespace-detect`.
- Flag global `--lang en|pt` com fallback via env var `NEUROGRAPHRAG_LANG`.
- Fallback de locale `LC_ALL`/`LANG` usado para mensagens de progresso no stderr.
- Novo módulo `i18n` com enum `Language` e helpers `init`/`current`/`tr`.
- Helpers bilíngues adicionados em `output::emit_progress_i18n`.
- Timestamps ISO 8601: `created_at_iso` adicionado em `RememberResponse`.
- `updated_at_iso` adicionado em itens de `list`.
- `created_at_iso`/`updated_at_iso` adicionados em `read`, paralelos aos inteiros epoch existentes.
- Resposta `read` agora inclui `memory_id` (alias de `id`).
- Resposta `read` agora inclui `type` (alias de `memory_type`).
- Resposta `read` agora inclui `version` para controle otimista.
- Itens `hybrid-search` agora incluem `score` (alias de `combined_score`).
- Itens `hybrid-search` agora incluem `source: "hybrid"`.
- Itens `list` agora incluem `memory_id` (alias de `id`).
- Resposta `stats` agora inclui `memories_total`, `entities_total`, `relationships_total`.
- Resposta `stats` agora inclui `chunks_total`, `db_bytes` para conformidade com contrato.
- Resposta `health` agora inclui `schema_version` no topo conforme PRD.
- Resposta `health` agora inclui `missing_entities[]` conforme PRD.
- `RememberResponse` inclui `operation` (alias de `action`), `created_at`, `created_at_iso`.
- `RecallResponse` inclui `results[]` com merge de `direct_matches` e `graph_matches`.
- Flag `init --namespace` adicionada, resolvida e ecoada em `InitResponse.namespace`.
- Flag `recall --min-distance <float>` adicionada (default 1.0, desativada por padrão).
- Quando `--min-distance` abaixo de 1.0, retorna exit 4 se todos os hits excederem o threshold.

### Corrigido

- Arquivos DB criados por `open_rw` agora recebem chmod 600 em Unix.
- Arquivos de snapshot criados por `sync-safe-copy` agora recebem chmod 600 em Unix.
- Previne vazamento de credenciais em montagens compartilhadas (Dropbox, NFS, `/tmp` multi-usuário).
- Mensagens de progresso em `remember`, `recall`, `hybrid-search`, `init` usam helper bilíngue.
- Idioma agora respeitado de forma consistente (antes misturava EN/PT na mesma sessão).

### Documentação

- COOKBOOK, AGENT_PROTOCOL, SKILL, CLAUDE.md atualizados para refletir schemas e flags reais.
- README, INTEGRATIONS e llms.txt atualizados para refletir exit codes reais.
- Validados contra o output de `--help` de cada subcomando.
- Subcomandos `graph` e `cleanup-orphans` agora documentados nos guias apropriados.
- Disclaimer honesto de latência adicionado: recall e hybrid-search levam ~1s por invocação.
- Latência de ~8ms requer daemon (planejado para v3.0.0 Tier 4).


## [2.0.0] - 2026-04-18

### Breaking

- EXIT CODE: `DbBusy` movido de 13 para 15 para liberar exit 13 para `BatchPartialFailure`.
- Scripts shell que detectavam `EX_UNAVAILABLE` (13) como DB busy agora devem checar 15.
- HYBRID-SEARCH: formato JSON da resposta remodelado; formato antigo era `{query, combined_rank[], vec_rank[], fts_rank[]}`.
- Novo formato: `{query, k, results: [{memory_id, name, namespace, type, description, body, combined_score, vec_rank?, fts_rank?}], graph_matches: []}`.
- Consumidores que parseavam `combined_rank` devem migrar para `results` conforme PRD linhas 771-787.
- PURGE: `--older-than-seconds` descontinuada em favor de `--retention-days`.
- A flag antiga permanece como alias oculto mas emite warning; será removida em v3.0.0.
- NAME SLUG: `NAME_SLUG_REGEX` mais estrita que `SLUG_REGEX` da v1.x.
- Nomes multichar devem agora começar com letra (requisito do PRD).
- Single-char `[a-z0-9]` ainda permitido; memórias existentes com dígito inicial passam inalteradas.
- `rename` para nomes estilo legado (dígito inicial, multichar) agora falhará.

### Adicionado

- `AppError::BatchPartialFailure { total, failed }` mapeando para exit 13.
- Reservado para `import`, `reindex` e batch stdin (entrando em Tier 3/4).
- Constantes em `src/constants.rs`: `PURGE_RETENTION_DAYS_DEFAULT=90`, `MAX_NAMESPACES_ACTIVE=100`.
- Constantes: `EMBEDDING_MAX_TOKENS=512`, `K_GRAPH_MATCHES_LIMIT=20`, `K_LIST_DEFAULT_LIMIT=100`.
- Constantes: `K_GRAPH_ENTITIES_DEFAULT_LIMIT=50`, `K_RELATED_DEFAULT_LIMIT=10`, `K_HISTORY_DEFAULT_LIMIT=20`.
- Constantes: `WEIGHT_VEC_DEFAULT=1.0`, `WEIGHT_FTS_DEFAULT=1.0`, `TEXT_BODY_PREVIEW_LEN=200`.
- Constantes: `ORT_NUM_THREADS_DEFAULT="1"`, `ORT_INTRA_OP_NUM_THREADS_DEFAULT="1"`, `OMP_NUM_THREADS_DEFAULT="1"`.
- Constantes: `BATCH_PARTIAL_FAILURE_EXIT_CODE=13`, `DB_BUSY_EXIT_CODE=15`.
- Flag `--dry-run` e `--retention-days` em `purge`.
- Campos `namespace` e `merged_into_memory_id: Option<i64>` em `RememberResponse`.
- Campo `k: usize` em `RecallResponse`.
- Campos `bytes_freed: i64`, `oldest_deleted_at: Option<i64>` em `PurgeResponse`.
- Campos `retention_days_used: u32`, `dry_run: bool` em `PurgeResponse`.
- Flag `--format` em `hybrid-search` (apenas JSON; text/markdown reservados para Tier 2).
- Flag `--expected-updated-at` (optimistic locking) em `rename` e `restore`.
- Guard de limite de namespaces ativos (`MAX_NAMESPACES_ACTIVE=100`) em `remember`.
- Retorna exit 5 quando o limite de namespaces ativos é excedido.

### Alterado

- `SLUG_REGEX` renomeada para `NAME_SLUG_REGEX` com valor conforme PRD.
- Novo padrão: `r"^[a-z][a-z0-9-]{0,78}[a-z0-9]$|^[a-z0-9]$"`.
- Nomes multichar devem começar com letra.

### Corrigido

- Prefixo `__` explicitamente rejeitado em `rename` (antes apenas aplicado em `remember`).
- Constantes `WEIGHT_VEC_DEFAULT`, `WEIGHT_FTS_DEFAULT` agora declaradas em `constants.rs`.
- Referências do PRD agora mapeiam símbolos reais.


## [1.2.1] - 2026-04-18

### Corrigido

- Falha de instalação em versões de `rustc` no intervalo `1.88..1.95`.
- Causada pela dependência transitiva `constant_time_eq 0.4.3` (puxada via `blake3`).
- Essa dependência elevou seu MSRV para 1.95.0 em uma patch release.
- `cargo install neurographrag` sem `--locked` agora sucede.
- Pin direto `constant_time_eq = "=0.4.2"` força versão compatível com `rust-version = "1.88"`.

### Alterado

- `Cargo.toml` agora declara pin preventivo explícito `constant_time_eq = "=0.4.2"`.
- Comentário inline documenta a razão do drift de MSRV.
- Pin será revisitado quando `rust-version` for elevado para 1.95.
- Instruções de instalação do `README.md` (EN e PT) atualizadas para `cargo install --locked neurographrag`.
- Bullet adicionado explicando a motivação para `--locked`.

### Adicionado

- Seção `docs_rules/prd.md` "Dependency MSRV Drift Protection" documenta o padrão canônico de mitigação.
- Padrão: pinagem direta de dependências transitivas problemáticas no `Cargo.toml` de nível superior.


## [1.2.0] - 2026-04-18

### Adicionado

- Semáforo de contagem cross-process com até 4 slots simultâneos via `src/lock.rs` (`acquire_cli_slot`).
- Memory guard abortando com exit 77 quando RAM livre está abaixo de 2 GB via `sysinfo` (`src/memory_guard.rs`).
- Signal handler para SIGINT, SIGTERM e SIGHUP via `ctrlc` com feature `termination`.
- Flag `--max-concurrency <N>` para controlar limite de invocações paralelas em runtime.
- Flag oculta `--skip-memory-guard` para testes automatizados onde a alocação real não ocorre.
- Constantes `MAX_CONCURRENT_CLI_INSTANCES`, `MIN_AVAILABLE_MEMORY_MB`, `CLI_LOCK_DEFAULT_WAIT_SECS` em `src/constants.rs`.
- Constantes `EMBEDDING_LOAD_EXPECTED_RSS_MB` e `LOW_MEMORY_EXIT_CODE` em `src/constants.rs`.
- Variantes `AppError::AllSlotsFull` e `AppError::LowMemory` com mensagens em português brasileiro.
- Global `SHUTDOWN: AtomicBool` e função `shutdown_requested()` em `src/lib.rs`.

### Alterado

- Default da flag `--wait-lock` aumentado para 300 segundos (5 minutos) via `CLI_LOCK_DEFAULT_WAIT_SECS`.
- Lock file migrado de `cli.lock` único para `cli-slot-{N}.lock` (semáforo de contagem N=1..4).

### Removido

- BREAKING: flag `--allow-parallel` removida; causou OOM crítico em produção (incidente 2026-04-18).

### Corrigido

- Bug crítico onde invocações CLI paralelas esgotavam a RAM do sistema.
- 58 invocações simultâneas travaram o computador por 38 minutos (incidente 2026-04-18).


## [Unreleased]

### Adicionado

- Flags globais `--allow-parallel` e `--wait-lock SECONDS` para concorrência controlada.
- Módulo `src/lock.rs` implementando lock single-instance baseado em arquivo via `fs4`.
- Nova variante `AppError::LockBusy` mapeando para exit code 75 (`EX_TEMPFAIL`).
- Variáveis de ambiente `ORT_NUM_THREADS`, `OMP_NUM_THREADS` e `ORT_INTRA_OP_NUM_THREADS` pré-definidas para 1.
- Singleton `OnceLock<Mutex<TextEmbedding>>` para reuso do modelo intra-processo.
- Testes de integração em `tests/lock_integration.rs` cobrindo aquisição e liberação de lock.

### Alterado

- Comportamento padrão agora é single-instance.
- Uma segunda invocação concorrente sai com código 75 exceto se `--allow-parallel` for passada.
- Módulo embedder refatorado de struct-com-estado para funções livres operando sobre um singleton.

### Corrigido

- Previne OOM livelock quando a CLI é invocada em paralelismo massivo por orquestradores LLM.


## [0.1.0] - 2026-04-17

### Adicionado

- Fase 1: Fundação: schema SQLite com vec0 (sqlite-vec), FTS5, grafo de entidades.
- Fase 2: Subcomandos essenciais: init, remember, recall, read, list, forget, rename, edit, history.
- Fase 2 continuação: restore, health, stats, optimize, purge, vacuum, migrate, hybrid-search.
- Fase 2 continuação: namespace-detect, sync-safe-copy.

### Corrigido

- Bug de corrupção FTS5 external-content no ciclo forget+purge.
- Removido DELETE manual em forget.rs que causava a corrupção.

### Alterado

- MSRV elevado de 1.80 para 1.88 (exigido por dependências transitivas base64ct 1.8.3, ort-sys, time).

[Unreleased]: https://github.com/daniloaguiarbr/neurographrag/compare/v2.1.0...HEAD
[2.1.0]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v2.1.0
[2.0.2]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v2.0.2
[2.0.1]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v2.0.1
[2.0.0]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v2.0.0
[1.2.1]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v1.2.1
[1.2.0]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v1.2.0
[0.1.0]: https://github.com/daniloaguiarbr/neurographrag/releases/tag/v0.1.0
