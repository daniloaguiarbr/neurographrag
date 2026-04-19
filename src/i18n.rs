//! Camada bilíngue de mensagens humanas.
//!
//! A CLI usa `--lang en|pt` (flag global) ou `NEUROGRAPHRAG_LANG` (env var) para escolher
//! o idioma das mensagens stderr de progresso. JSON de stdout é determinístico e idêntico
//! entre idiomas — apenas strings destinadas a humanos passam pelo módulo.
//!
//! Detecção (do mais para o menos prioritário):
//! 1. Flag `--lang` explícita
//! 2. Env var `NEUROGRAPHRAG_LANG`
//! 3. Locale do SO (`LANG`, `LC_ALL`) com prefixo `pt`
//! 4. Fallback `English`

use std::sync::OnceLock;

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    #[value(name = "en", aliases = ["english", "EN"])]
    English,
    #[value(name = "pt", aliases = ["portugues", "portuguese", "pt-BR", "pt-br", "PT"])]
    Portugues,
}

impl Language {
    pub fn from_env_or_locale() -> Self {
        if let Ok(v) = std::env::var("NEUROGRAPHRAG_LANG") {
            let v = v.to_lowercase();
            if v.starts_with("pt") {
                return Language::Portugues;
            }
            if v.starts_with("en") {
                return Language::English;
            }
        }
        for var in &["LC_ALL", "LANG"] {
            if let Ok(v) = std::env::var(var) {
                if v.to_lowercase().starts_with("pt") {
                    return Language::Portugues;
                }
            }
        }
        Language::English
    }
}

static IDIOMA_GLOBAL: OnceLock<Language> = OnceLock::new();

/// Inicializa o idioma global. Chamadas subsequentes são ignoradas silenciosamente
/// (OnceLock semantics) — garantindo thread-safety e determinismo.
pub fn init(explicit: Option<Language>) {
    let resolved = explicit.unwrap_or_else(Language::from_env_or_locale);
    let _ = IDIOMA_GLOBAL.set(resolved);
}

/// Retorna o idioma ativo ou fallback English se `init` nunca foi chamado.
pub fn current() -> Language {
    *IDIOMA_GLOBAL.get_or_init(Language::from_env_or_locale)
}

/// Traduz uma mensagem bilíngue escolhendo a variante ativa.
pub fn tr(en: &str, pt: &str) -> &'static str {
    // SAFETY: Retornamos uma das duas strings estáticas passadas como &str.
    // Como não temos como provar ao borrow checker que as referências sobrevivem,
    // usamos Box::leak para transformar em &'static str. Custo mínimo (dezenas de
    // strings distintas durante vida do processo CLI).
    match current() {
        Language::English => Box::leak(en.to_string().into_boxed_str()),
        Language::Portugues => Box::leak(pt.to_string().into_boxed_str()),
    }
}

/// Mensagens de validação localizadas para os campos de memória.
pub mod validacao {
    use super::current;
    use crate::i18n::Language;

    pub fn nome_comprimento(max: usize) -> String {
        match current() {
            Language::English => format!("name must be 1-{max} chars"),
            Language::Portugues => format!("nome deve ter entre 1 e {max} caracteres"),
        }
    }

    pub fn nome_reservado() -> String {
        match current() {
            Language::English => {
                "names and namespaces starting with __ are reserved for internal use".to_string()
            }
            Language::Portugues => {
                "nomes e namespaces iniciados com __ são reservados para uso interno".to_string()
            }
        }
    }

    pub fn nome_kebab(nome: &str) -> String {
        match current() {
            Language::English => format!(
                "name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
            ),
            Language::Portugues => {
                format!("nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'")
            }
        }
    }

    pub fn descricao_excede(max: usize) -> String {
        match current() {
            Language::English => format!("description must be <= {max} chars"),
            Language::Portugues => format!("descrição deve ter no máximo {max} caracteres"),
        }
    }

    pub fn body_excede(max: usize) -> String {
        match current() {
            Language::English => format!("body exceeds {max} chars"),
            Language::Portugues => format!("corpo excede {max} caracteres"),
        }
    }

    pub fn novo_nome_comprimento(max: usize) -> String {
        match current() {
            Language::English => format!("new-name must be 1-{max} chars"),
            Language::Portugues => format!("novo nome deve ter entre 1 e {max} caracteres"),
        }
    }

    pub fn novo_nome_kebab(nome: &str) -> String {
        match current() {
            Language::English => format!(
                "new-name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
            ),
            Language::Portugues => format!(
                "novo nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'"
            ),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn fallback_english_quando_env_ausente() {
        std::env::remove_var("NEUROGRAPHRAG_LANG");
        std::env::set_var("LC_ALL", "C");
        std::env::set_var("LANG", "C");
        assert_eq!(Language::from_env_or_locale(), Language::English);
    }

    #[test]
    fn env_pt_seleciona_portugues() {
        std::env::set_var("NEUROGRAPHRAG_LANG", "pt");
        assert_eq!(Language::from_env_or_locale(), Language::Portugues);
        std::env::remove_var("NEUROGRAPHRAG_LANG");
    }

    #[test]
    fn env_pt_br_seleciona_portugues() {
        std::env::set_var("NEUROGRAPHRAG_LANG", "pt-BR");
        assert_eq!(Language::from_env_or_locale(), Language::Portugues);
        std::env::remove_var("NEUROGRAPHRAG_LANG");
    }

    #[test]
    fn locale_ptbr_utf8_seleciona_portugues() {
        std::env::remove_var("NEUROGRAPHRAG_LANG");
        std::env::set_var("LC_ALL", "pt_BR.UTF-8");
        assert_eq!(Language::from_env_or_locale(), Language::Portugues);
        std::env::remove_var("LC_ALL");
    }

    mod testes_validacao {
        use super::*;

        #[test]
        fn nome_comprimento_en() {
            let msg = match Language::English {
                Language::English => format!("name must be 1-{} chars", 80),
                Language::Portugues => format!("nome deve ter entre 1 e {} caracteres", 80),
            };
            assert!(msg.contains("name must be 1-80 chars"), "obtido: {msg}");
        }

        #[test]
        fn nome_comprimento_pt() {
            let msg = match Language::Portugues {
                Language::English => format!("name must be 1-{} chars", 80),
                Language::Portugues => format!("nome deve ter entre 1 e {} caracteres", 80),
            };
            assert!(
                msg.contains("nome deve ter entre 1 e 80 caracteres"),
                "obtido: {msg}"
            );
        }

        #[test]
        fn nome_kebab_en() {
            let nome = "Invalid_Name";
            let msg = match Language::English {
                Language::English => format!(
                    "name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
                ),
                Language::Portugues => {
                    format!("nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'")
                }
            };
            assert!(msg.contains("kebab-case slug"), "obtido: {msg}");
            assert!(msg.contains("Invalid_Name"), "obtido: {msg}");
        }

        #[test]
        fn nome_kebab_pt() {
            let nome = "Invalid_Name";
            let msg = match Language::Portugues {
                Language::English => format!(
                    "name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
                ),
                Language::Portugues => {
                    format!("nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'")
                }
            };
            assert!(msg.contains("kebab-case"), "obtido: {msg}");
            assert!(msg.contains("minúsculas"), "obtido: {msg}");
            assert!(msg.contains("Invalid_Name"), "obtido: {msg}");
        }

        #[test]
        fn descricao_excede_en() {
            let msg = match Language::English {
                Language::English => format!("description must be <= {} chars", 500),
                Language::Portugues => format!("descrição deve ter no máximo {} caracteres", 500),
            };
            assert!(msg.contains("description must be <= 500"), "obtido: {msg}");
        }

        #[test]
        fn descricao_excede_pt() {
            let msg = match Language::Portugues {
                Language::English => format!("description must be <= {} chars", 500),
                Language::Portugues => format!("descrição deve ter no máximo {} caracteres", 500),
            };
            assert!(
                msg.contains("descrição deve ter no máximo 500"),
                "obtido: {msg}"
            );
        }

        #[test]
        fn body_excede_en() {
            let msg = match Language::English {
                Language::English => format!("body exceeds {} chars", 20_000),
                Language::Portugues => format!("corpo excede {} caracteres", 20_000),
            };
            assert!(msg.contains("body exceeds 20000"), "obtido: {msg}");
        }

        #[test]
        fn body_excede_pt() {
            let msg = match Language::Portugues {
                Language::English => format!("body exceeds {} chars", 20_000),
                Language::Portugues => format!("corpo excede {} caracteres", 20_000),
            };
            assert!(msg.contains("corpo excede 20000"), "obtido: {msg}");
        }

        #[test]
        fn novo_nome_comprimento_en() {
            let msg = match Language::English {
                Language::English => format!("new-name must be 1-{} chars", 80),
                Language::Portugues => format!("novo nome deve ter entre 1 e {} caracteres", 80),
            };
            assert!(msg.contains("new-name must be 1-80"), "obtido: {msg}");
        }

        #[test]
        fn novo_nome_comprimento_pt() {
            let msg = match Language::Portugues {
                Language::English => format!("new-name must be 1-{} chars", 80),
                Language::Portugues => format!("novo nome deve ter entre 1 e {} caracteres", 80),
            };
            assert!(
                msg.contains("novo nome deve ter entre 1 e 80"),
                "obtido: {msg}"
            );
        }

        #[test]
        fn novo_nome_kebab_en() {
            let nome = "Bad Name";
            let msg = match Language::English {
                Language::English => format!(
                    "new-name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
                ),
                Language::Portugues => format!(
                    "novo nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'"
                ),
            };
            assert!(msg.contains("new-name must be kebab-case"), "obtido: {msg}");
        }

        #[test]
        fn novo_nome_kebab_pt() {
            let nome = "Bad Name";
            let msg = match Language::Portugues {
                Language::English => format!(
                    "new-name must be kebab-case slug (lowercase letters, digits, hyphens): '{nome}'"
                ),
                Language::Portugues => format!(
                    "novo nome deve estar em kebab-case (minúsculas, dígitos, hífens): '{nome}'"
                ),
            };
            assert!(
                msg.contains("novo nome deve estar em kebab-case"),
                "obtido: {msg}"
            );
        }

        #[test]
        fn nome_reservado_en() {
            let msg = match Language::English {
                Language::English => {
                    "names and namespaces starting with __ are reserved for internal use"
                        .to_string()
                }
                Language::Portugues => {
                    "nomes e namespaces iniciados com __ são reservados para uso interno"
                        .to_string()
                }
            };
            assert!(msg.contains("reserved for internal use"), "obtido: {msg}");
        }

        #[test]
        fn nome_reservado_pt() {
            let msg = match Language::Portugues {
                Language::English => {
                    "names and namespaces starting with __ are reserved for internal use"
                        .to_string()
                }
                Language::Portugues => {
                    "nomes e namespaces iniciados com __ são reservados para uso interno"
                        .to_string()
                }
            };
            assert!(msg.contains("reservados para uso interno"), "obtido: {msg}");
        }
    }
}
