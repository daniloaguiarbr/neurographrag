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
    English,
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
}
