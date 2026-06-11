use gingr::config::{ApiKey, BaseUrl, ClientConfig, Subdomain};

const SENTINEL_KEY: &str = "gingr_test_api_key_do_not_send";

#[test]
fn subdomain_accepts_gingr_app_slug_and_builds_https_base_url() {
    let subdomain = Subdomain::parse("example-pet-resort").expect("valid app slug");
    let base_url = BaseUrl::for_subdomain(&subdomain);

    assert_eq!(base_url.as_str(), "https://example-pet-resort.gingrapp.com");
}

#[test]
fn subdomain_rejects_values_that_could_escape_the_gingr_host_boundary() {
    for invalid in [
        "",
        "Example",
        "example.pet",
        "example_pet",
        "-example",
        "example-",
        "https://example.gingrapp.com",
        "example.com",
    ] {
        assert!(
            Subdomain::parse(invalid).is_err(),
            "{invalid:?} should be rejected"
        );
    }
}

#[test]
fn base_url_rejects_non_https_and_non_gingr_hosts() {
    for invalid in [
        "http://example-pet-resort.gingrapp.com",
        "https://example.com",
        "https://example.gingrapp.com.evil.test",
    ] {
        assert!(
            BaseUrl::parse(invalid).is_err(),
            "{invalid:?} should be rejected"
        );
    }
}

#[test]
fn client_config_debug_and_display_never_expose_api_key() {
    let config = ClientConfig::new(
        BaseUrl::parse("https://example-pet-resort.gingrapp.com").unwrap(),
        ApiKey::from_secret(SENTINEL_KEY),
    );

    let debug = format!("{config:?}");
    let display = config.to_string();

    assert!(!debug.contains(SENTINEL_KEY));
    assert!(!display.contains(SENTINEL_KEY));
    assert!(debug.contains("<redacted>"));
    assert!(display.contains("<redacted>"));
}
