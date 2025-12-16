//! Shared test utilities and fixtures
//!
//! This module provides common test helpers to reduce duplication
//! and make tests more robust against implementation changes.

#[cfg(test)]
pub mod fixtures {
    use crate::commands::analysis::AnalysisSettingsRequest;
    use crate::domain::models::*;
    use crate::service::gemini::GeminiRequest;
    use sqlx::SqlitePool;

    /// Creates an in-memory SQLite database with migrations applied
    pub async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");
        sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");
        pool
    }

    /// Creates a minimal GeminiRequest for testing
    pub fn minimal_gemini_request() -> GeminiRequest {
        GeminiRequest {
            analysis_id: "test".into(),
            url: "https://example.com".into(),
            seo_score: 50,
            pages_count: 1,
            total_issues: 0,
            critical_issues: 0,
            warning_issues: 0,
            suggestion_issues: 0,
            top_issues: vec![],
            avg_load_time: 1.0,
            total_words: 100,
            ssl_certificate: true,
            sitemap_found: true,
            robots_txt_found: true,
        }
    }

    /// Creates default analysis settings for testing
    pub fn default_settings() -> AnalysisSettingsRequest {
        AnalysisSettingsRequest::default()
    }

    /// Creates settings with a specific max_pages value
    pub fn settings_with_max_pages(max_pages: i64) -> AnalysisSettingsRequest {
        AnalysisSettingsRequest {
            max_pages,
            ..Default::default()
        }
    }
}

/// Helper assertions for tests
#[cfg(test)]
pub mod assertions {
    use crate::domain::models::SeoIssue;

    /// Checks if issues contain a specific issue title
    pub fn has_issue(issues: &[SeoIssue], title: &str) -> bool {
        issues.iter().any(|i| i.title == title)
    }

    /// Counts issues of a specific type
    pub fn count_issues(issues: &[SeoIssue], title: &str) -> usize {
        issues.iter().filter(|i| i.title == title).count()
    }

    /// Asserts that a result contains the expected issue
    #[macro_export]
    macro_rules! assert_has_issue {
        ($issues:expr, $title:expr) => {
            assert!(
                $crate::test_utils::assertions::has_issue($issues, $title),
                "Expected to find issue '{}' but it was not present",
                $title
            );
        };
    }

    /// Asserts that a result does NOT contain the specified issue
    #[macro_export]
    macro_rules! assert_no_issue {
        ($issues:expr, $title:expr) => {
            assert!(
                !$crate::test_utils::assertions::has_issue($issues, $title),
                "Expected NOT to find issue '{}' but it was present",
                $title
            );
        };
    }
}

/// Mock server helpers for integration tests
#[cfg(test)]
pub mod mocks {
    use serde_json::json;

    /// Creates a standard HTML page for testing
    pub fn basic_html_page(title: &str, h1: &str) -> String {
        format!(
            r#"
            <html>
                <head><title>{}</title></head>
                <body>
                    <h1>{}</h1>
                    <p>Some content here.</p>
                </body>
            </html>
            "#,
            title, h1
        )
    }

    /// Creates HTML with an image missing alt text
    pub fn html_with_missing_alt() -> String {
        r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Welcome</h1>
                <img src="logo.png">
            </body>
        </html>
        "#
        .to_string()
    }

    /// Creates a mock Gemini API response body
    pub fn gemini_response(text: &str) -> String {
        json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": text }]
                }
            }]
        })
        .to_string()
    }

    pub fn discord_html() -> String {
        r##"
<!DOCTYPE html><!-- This site was created in Webflow. https://webflow.com -->
<!-- Last Published: Thu Dec 11 2025 10:48:56 GMT+0000 (Coordinated Universal Time) -->
<html data-wf-domain="prod-wf3.discord.com" data-wf-page="62842999620aab8ff36a026f"
	data-wf-site="6238e97f6441e30a13a52345" data-wf-collection="62842999620aab480e6a0290"
	data-wf-item-slug="establishing-trust-with-connections-connection-details-and-linked-roles">

<head>
	<meta charset="utf-8" />
	<title>Establishing Trust with Social Media Connections and Roles</title>
	<meta
		content="Discord Connections - a service that allows members to share info about their third-party accounts and profiles."
		name="description" />
	<meta content="Establishing Trust with Social Media Connections and Roles" property="og:title" />
	<meta
		content="Discord Connections - a service that allows members to share info about their third-party accounts and profiles."
		property="og:description" />
	<meta
		content="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639752c1cb940a795f15f472_COMM_PortalHeaders_Manage_v2.png"
		property="og:image" />
	<meta content="Establishing Trust with Social Media Connections and Roles" property="twitter:title" />
	<meta
		content="Discord Connections - a service that allows members to share info about their third-party accounts and profiles."
		property="twitter:description" />
	<meta
		content="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639752c1cb940a795f15f472_COMM_PortalHeaders_Manage_v2.png"
		property="twitter:image" />
	<meta property="og:type" content="website" />
	<meta content="summary_large_image" name="twitter:card" />
	<meta content="width=device-width, initial-scale=1" name="viewport" />
	<meta content="Webflow" name="generator" />
	<link
		href="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/css/creator-portal.webflow.shared.e2e5c8260.css"
		rel="stylesheet" type="text/css" />
	<link href="https://fonts.googleapis.com" rel="preconnect" />
	<link href="https://fonts.gstatic.com" rel="preconnect" crossorigin="anonymous" />
	<script src="https://ajax.googleapis.com/ajax/libs/webfont/1.6.26/webfont.js" type="text/javascript"></script>
	<script type="text/javascript">
		WebFont.load({  google: {    families: ["Press Start 2P:regular:cyrillic,latin","Poppins:200,300,regular,500,600,700,800,900"]  }});
	</script>
	<script type="text/javascript">
		!function(o,c){var n=c.documentElement,t=" w-mod-";n.className+=t+"js",("ontouchstart"in o||o.DocumentTouch&&c instanceof DocumentTouch)&&(n.className+=t+"touch")}(window,document);
	</script>
	<link
		href="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/623e0a78145b815f5f0ea8d0_60ae916347747e71167e21cc_favicon.png"
		rel="shortcut icon" type="image/x-icon" />
	<link
		href="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/623e0a7c7556c107fd964565_5f91fae62cc821206588b837_Frame%20246.png"
		rel="apple-touch-icon" />
	<link href="discord.com/community/community/establishing-trust-with-connections-connection-details-and-linked-roles"
		rel="canonical" /><!-- Localize integration code -->
	<script src="https://global.localizecdn.com/localize.js"></script>
	<script src="https://discord.com/webflow-scripts/head.js"></script>
	<script>
		!function(a){if(!a.Localize){a.Localize={};for(var e=["translate","untranslate","phrase","initialize","translatePage","setLanguage","getLanguage","getSourceLanguage","detectLanguage","getAvailableLanguages","untranslatePage","bootstrap","prefetch","on","off","hideWidget","showWidget"],t=0;t<e.length;t++)a.Localize[e[t]]=function(){}}}(window);
	</script>

	<script>
		Localize.initialize({
    key: 'XTwS61yOs521g',
    rememberLanguage: true,
    disableWidget: true,
    autodetectLanguage: true,
    blockedClasses: ['dont-translate'],
    blockedIds: ['onetrust-banner-sdk', 'onetrust-consent-sdk']
  });
	</script>
	<style>
		/* Truncated for brevity in test */
	</style>
	<script src="/w/assets/4efad7abbb181e82f5667100cf0adb93a40de8f6/styles.js" defer></script>
	<!-- webflow can't style ol/ul differently. So we need to reset ul style for ol list items -->
	<style>
		.articlebody ol li {
			background-image: none;
			padding-left: 12px;
			list-style-type: auto;
		}
	</style>
</head>

<body class="bodystyle2">
	<div class="hide">
		<div class="discord-2022--nav">
			<img src="https://cdn.prod.website-files.com/plugins/Basic/assets/placeholder.60f9b1840c.svg" loading="lazy" alt="" class="discord-2022--nav_brand_main-logo"/><img src="https://cdn.prod.website-files.com/plugins/Basic/assets/placeholder.60f9b1840c.svg" loading="lazy" alt="" class="discord-2022--nav_brand_main-black-logo"/>
			<div class="discord-2022--menu-button-login discord-2022--login-button-js">This is some text inside of a div
				block.</div>
			<div class="discord-2022--button-nav">This is some text inside of a div block.</div>
			<img src="https://cdn.prod.website-files.com/plugins/Basic/assets/placeholder.60f9b1840c.svg" loading="lazy" alt="" class="discord-2022--nav_brand_main-logo-2"/></div>
			<div class="w-embed">
				<style>
					/* Truncated */
				</style>
			</div>
		</div>
		<div>
			<div>
				<div class="navigation-styles w-embed">
					<style>
						/* Truncated */
					</style>
				</div>
                <div class="styles">
					<div class="global-styles w-embed">
						<style>
                            /* Truncated */
                        </style>
                    </div>
                </div>
				<header class="discord-2022--nav">
					<div class="discord-2022--nav_padding">
						<div class="discord-2022--nav_wrapper"><a href="/"
								class="discord-2022--nav_brand w-nav-brand"><img width="146" loading="lazy" alt="Home" src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e50_4f638e89da5d36ef6d5044397b029042_Discrod-Main-Logo.svg" class="discord-2022--nav_brand_main-logo"/>
								<div class="discord-2022--nav_brand_main-black-logo">
									<img width="146" loading="lazy" alt="" src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/69039aa74c5826ff311d7469_57cabb9d300748204f0f1fc4433f5649_Logo.svg" class="discord-2022--nav_brand_main-black-icon"/><img width="Auto" loading="lazy" alt="" src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/69039aa74c5826ff311d746a_65bf17408a3c0a90c528ffdb3b1305ed_Logo-black-bg.svg" class="discord-2022--nav_brand_main-logo-2"/><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/69039aa74c5826ff311d7468_7dd30979524089ebc62203a1c5ea5141_Logo-3-bg.svg" loading="lazy" alt="" class="discord-2022--nav_brand_main-logo-bg"/></div>
							</a>
							<div class="discord-2022--nav_menu_wr">
								<ul role="list" class="discord-2022--nav_menu">
									<li><a href="/download" class="discord-2022--nav_link">Download</a></li>
									<li><a href="/nitro" class="discord-2022--nav_link">Nitro</a></li>
									<li><a href="/servers" class="discord-2022--nav_link">Discover</a></li>
									<li>
										<div data-delay="0" data-hover="true"
											class="discord-2022--nav_dd discord-2022--is-relative w-dropdown">
											<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
												<div>Safety</div>
												<div class="discord-2022--nav_dd_arrow-wr-white">
													<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
															height="16" viewBox="0 0 16 16" fill="none"
															xmlns="http://www.w3.org/2000/svg">
															<path d="M12 6L8 10L4 6" stroke="white" stroke-opacity="0.5"
																stroke-width="2" stroke-linecap="round"
																stroke-linejoin="round" />
														</svg></div>
												</div>
												<div class="discord-2022--nav_dd_arrow-wr-black">
													<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
															height="16" viewBox="0 0 16 16" fill="none"
															xmlns="http://www.w3.org/2000/svg">
															<path d="M12 6L8 10L4 6" stroke="black" stroke-opacity="0.5"
																stroke-width="2" stroke-linecap="round"
																stroke-linejoin="round" />
														</svg></div>
												</div><a tabindex="0" href="/safety"
													class="discord-2022--nav_link_dropdown w-inline-block"></a>
											</div>
											<nav
												class="discord-2022--nav_dd_list discord-2022--is-safety w-dropdown-list">
												<div class="discord-2022--nav_dd_content-wr discord-2022--isnew">
													<div
														class="discord-2022--nav_dd_content_layout discord-2022--top-none discord-2022--2-col discord-2022--is_n">
														<div
															class="discord-2022--nav_dd_link-group discord-2022--is_2025 discord-2022--is_n">
															<div
																class="discord-2022--nav_dd_link_list discord-2022--is-new">
																<div class="discord-2022--nav_dd_link_title">Resources
																</div><a href="/safety-family-center"
																	class="discord-2022--dd_nav-link">Family
																	Center</a><a href="/safety-library"
																	class="discord-2022--dd_nav-link">Safety
																	Library</a><a href="/safety-news"
																	class="discord-2022--dd_nav-link">Safety News</a><a
																	href="/safety-teen-charter"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Teen Charter</div>
																</a>
															</div>
														</div>
														<div
															class="discord-2022--nav_dd_link-group discord-2022--is_hub">
															<div
																class="discord-2022--nav_dd_link_list discord-2022--is-new">
																<div class="discord-2022--nav_dd_link_title">Hubs</div>
																<a href="/safety-parents"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Parent Hub</div>
																</a><a href="/safety-policies"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Policy Hub</div>
																</a><a href="/safety-privacy"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Privacy Hub</div>
																</a><a href="/safety-transparency"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Transparency Hub</div>
																</a><a href="/safety-wellbeing"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Wellbeing Hub</div>
																</a>
															</div>
														</div>
													</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e4b_9ecc52c46529a25a6f5795dc86a9707e_Egg.webp" loading="eager" alt="" class="discord-2022--nav-dd-decor discord-2022--is-safety"/></div>
											</nav>
										</div>
									</li>
                                    <!-- ... (Truncated rest of nav for brevity in regex replacement, assume full content was passed if I had infinite context) ... -->
                                    <!-- Included key parts for test -->
                                    <li><a href="https://discord.com/careers" class="discord-2022--nav_link">Careers</a></li>
								</ul>
                                <div class="discord-2022--nav_buttons-wr_new"><a id="login" data-track="login"
									data-track-nav="login" href="https://discord.com/app"
									class="discord-2022--button-nav discord-2022--login-button-js discord-2022--new w-button">Log
									In</a></div>
							</div>
                        </div>
                    </div>
                </header>
            </div>
            <div class="s-hero-safeties">
				<div class="container-1260px _980px">
					<div class="breadcrumbs-wrapper-2"><a href="/community" class="breadcrumbs-link-2">Community
							Portal</a><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6436a90247198009dc18c84b_chevron%20right%202.svg" loading="lazy" alt=""/><a
							href="/community/establishing-trust-with-connections-connection-details-and-linked-roles"
							aria-current="page" class="breadcrumbs-link-2 _2 w--current">Establishing Trust with Social
							Media Connections and Roles</a></div>
				</div>
			</div>
            <div class="section-blue white safeties">
                <!-- Content ... -->
                <a href="https://discord.com/developers/docs/resources/application-role-connection-metadata">here</a>
            </div>
            <!-- Footer -->
             <div class="discord-2022--p-footer">Language</div>
                                    <div class="discord-2022--padding-16px"></div>
                                    <div data-hover="false" data-delay="0"
                                        data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b928"
                                        class="discord-2022--dropdown-language-wr w-dropdown">
                                        <!-- Language dropdown with many items -->
                                        <nav class="discord-2022--dropdown-language-list-wr w-dropdown-list">
												<ul role="list" class="discord-2022--dropdown-list-container-wr">
													<li tabindex="0" data-locale="cs"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Čeština</div>
													</li>
                                                </ul>
                                        </nav>
                                    </div>
    </body>
</html>
"##.to_string()
    }
}
