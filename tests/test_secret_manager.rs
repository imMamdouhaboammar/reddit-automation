use reddit_agent::auth::secret_manager::GcpSecretManagerStore;
use reddit_agent::auth::CredentialStore;

#[test]
fn test_headless_credential_store_fallback_and_memory() {
    let store = GcpSecretManagerStore::new_with_memory_fallback();

    // Set a secret in headless mode
    store
        .set_secret(
            "reddit-agent",
            "browser_cookies:memory_isolated",
            "cookie_data_test",
        )
        .expect("set secret");

    let retrieved = store
        .get_secret("reddit-agent", "browser_cookies:memory_isolated")
        .expect("get secret");

    assert_eq!(retrieved.expose_secret(), "cookie_data_test");
}

#[test]
fn test_headless_env_secret_provider() {
    let env_key = "REDDIT_SECRET_BROWSER_COOKIES_ENV_ISOLATED";
    std::env::set_var(env_key, "env_injected_cookie_json");
    let store = GcpSecretManagerStore::new_with_memory_fallback();

    let retrieved = store
        .get_secret("reddit-agent", "browser_cookies:env_isolated")
        .expect("get secret from env");

    assert_eq!(retrieved.expose_secret(), "env_injected_cookie_json");
    std::env::remove_var(env_key);
}
