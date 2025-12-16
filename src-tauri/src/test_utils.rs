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
		.nav-bar-hack {
			bottom: 0px !important;
			height: auto !important;
		}

		body a.w-webflow-badge {
			display: none !important;
		}

		body {
			text-rendering: optimizeLegibility;
		}

		@media screen and (max-width: 476px) {
			.hide-on-mobile {
				display: none;
			}
		}

		.hr-style {
			border: 0;
			height: 8px;
			background-color: #F0F0F0;
			margin: 20px 0px;
		}

		.BlogBodyQuote {
			grid-column: 2/-1;
			font-size: 24px;
			line-height: 36px;
			padding: 0px 0px 15px 40px;
			font-style: italic;
			margin: 40px 0px 50px 0px;
		}

		.quote-text {
			width: 100%;
			margin-top: 10px;
		}

		.quote-icon {
			position: relative;
			top: 16px;
		}

		footer.quote-footer {
			font-size: 16px;
			margin-top: 20px;
			font-style: normal;
		}
	</style>

	<style>
		.language {
			display: flex;
			user-select: none;
		}

		.language .lang-container {
			position: relative;
		}

		.language .lang-selector-container {
			display: flex;
			align-items: center;
			cursor: pointer;
		}

		.language .locale-container {
			display: flex;
			align-items: center;
		}

		.language .flag {
			width: 24px;
			height: 16px;
			margin-right: 8px;
		}

		.language .selector-language-name {
			color: #fff;
			font-size: 14px;
			line-height: 18px;
		}

		.language .arrow-icon {
			padding-left: 8px;
		}

		.language .lang-dropdown-container {
			z-index: 10;
			bottom: 100%;
			margin-bottom: 8px;
			position: absolute;
			background-color: #fff;
			border-radius: 8px;
			box-shadow: 0 1px 1px rgb(0 0 0 / 10%);
			overflow: hidden;
			display: none;
		}

		.language .lang-dropdown {
			max-height: 320px;
			min-width: 150px;
			overflow-x: hidden;
			overflow-y: auto;
			overscroll-behavior: contain;
		}

		.language .dropdown-item {
			padding: 8px;
		}

		.language .dropdown-clickable {
			cursor: pointer;
		}

		.language .dropdown-language-name {
			color: #23272a;
			font-size: 14px;
			line-height: 18px;
		}

		.flag-ru {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1059cf43b4b819eb95b_62cb46f39e6ac4c46ce39566_ru.png);
		}

		.flag-bg {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e102e41f7fa23a5227a4_6257c2a1e7544e303083b2b1_bolg.png);
		}

		.flag-cs {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e103c57874ff191897b4_62cb46f1254305732a01676d_cs.png);
		}

		.flag-da {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104bec923ede26512ee_62cb46f16128094022db6768_da.png);
		}

		.flag-de {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104ffc29d5941c1a397_62cb46f1c50496ce73c40d99_de.png);
		}

		.flag-en-GB {
			content: url("https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e0f7f9ddbfd4727be59e_62d01c2078d11b68a1633276_Rectangle%201%20(3).svg");
		}

		.flag-el {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1034d3ae3f65cbdb9b8_62cb46f17c26b5fe5a53876f_el.png);
		}

		.flag-en-US {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e102e1d2a86dd1303277_6257bf8b5ba300233705a542_en.png);
		}

		.flag-en {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e102e1d2a86dd1303277_6257bf8b5ba300233705a542_en.png);
		}

		.flag-es {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104230aaa2d5e30970f_62cb46f14edab1b0029593fc_es-ES.png);
		}

		.flag-fi {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e103fe46ca167ef01313_62cb46f1921c0cf82fc59da7_fi.png);
		}

		.flag-fr {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1033eac32725d7835f2_62cb46f1544a7ab7c66e9ccb_fr.png);
		}

		.flag-hi {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e103d0e078a0cd2993f2_62cb46f13fcb6e76c05b504e_hi.png);
		}

		.flag-hr {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e10217e49a173027280f_62cb46f1aeebe9064763c90c_hr.png);
		}

		.flag-hu {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1036a45eb296afddaae_62cb46f19e6ac41dcce39561_hu.png);
		}

		.flag-it {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e103ea937c7af18587dd_62cb46f1bd099a25f8f77ea4_it.png);
		}

		.flag-ja {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1030a1aad0842635e83_62cb46f1e819841940bec47d_ja.png);
		}

		.flag-ko {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104dfaf1a667f1baa62_62cb46f125430509b9016776_ko.png);
		}

		.flag-lt {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e10405dce1080cd14041_62cb46f14edab152b8959405_lt.png);
		}

		.flag-nl {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e105e41c3b84b4b29bb0_62cb46f3e00ff80959abff2a_nl.png);
		}

		.flag-no {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104e1d2a838f5303279_62cb46f37c26b5e22453877d_no.png);
		}

		.flag-pl {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1058c4fb3bbc8647a0b_62cb46f3c504963019c40db7_pl.png);
		}

		.flag-pt-BR {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e10450d59c5856791795_62cb46f3d809bc2503e62bec_pt-BR.png);
		}

		.flag-ro {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e104706ab95ed5303690_62cb46f36e94d725ce411ab6_ro.png);
		}

		.flag-sv {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e10514a4260485fb1f13_62cb46f49e6ac47674e39567_sv-SE.png);
		}

		.flag-th {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e105baaf6ca7efe18f60_62cb46f465c529bf26e211a1_th.png);
		}

		.flag-tr {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e105d0e0787391299405_62cb46f4e819848178bec4d1_tr.png);
		}

		.flag-uk {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e10544802c73845b4d93_62cb46f37c26b54f6a53877f_uk.png);
		}

		.flag-vi {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e1349fd1f95536dd2ff2_62cb46f4e819840d89bec4d2_vi.png);
		}

		.flag-zh-Hans {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e105a3d7e17d4c78a5e2_62cb46f49e6ac45f35e39568_zh-CN.png);
		}

		.flag-zh-TW {
			content: url(https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/6332e105b41b547cdb6f990f_62cb46f33fcb6ea5c95b5069_zh-TW.png);
		}
	</style>


	<style>
		.nav-bar-hack {
			bottom: 0px !important;
			height: auto !important;
		}

		body a.w-webflow-badge {
			display: none !important;
		}

		body {
			text-rendering: optimizeLegibility;
		}

		@media screen and (max-width: 476px) {
			.hide-on-mobile {
				display: none;
			}
		}

		.hr-style {
			border: 0;
			height: 8px;
			background-color: #F0F0F0;
			margin: 20px 0px;
		}

		.BlogBodyQuote {
			grid-column: 2/-1;
			font-size: 24px;
			line-height: 36px;
			padding: 0px 0px 15px 40px;
			font-style: italic;
			margin: 40px 0px 50px 0px;
		}

		.quote-text {
			width: 100%;
			margin-top: 10px;
		}

		.quote-icon {
			position: relative;
			top: 16px;
		}

		footer.quote-footer {
			font-size: 16px;
			margin-top: 20px;
			font-style: normal;
		}
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
					@media (max-width: 1279px) and (min-width: 992px) {
						.discord-2022--nav_brand_main-logo-2 {
							opacity: 1;
						}

						.discord-2022--nav_brand_main-black-icon {
							opacity: 0;
						}

						.discord-2022--nav_brand {
							width: 9rem;
						}

						.discord-2022--nav_brand_main-black-logo {
							background-color: black;
						}

					}
				</style>
			</div>
		</div>
		<div>
			<div>
				<div class="navigation-styles w-embed">
					<style>
						/*nav styles*/
						ul {
							margin-bottom: 0;
						}

						.discord-2022--nav:has(.discord-2022--nav_dd_trigger.w--open) .discord-2022--nav_bg {
							height: 100%;
							opacity: 1;
						}

						.discord-2022--nav_burger_content .discord-2022--nav_link {
							padding-top: 1.5rem;
							padding-left: 0;
							padding-bottom: 1.5rem;
							border-bottom: 1px solid #ffffff10;
							border-radius: 0px;
							justify-content: left;
						}

						.discord-2022--nav_blur {
							display: block;
							/* Makes the block visible when displayed */
							opacity: 0;
							/* Initial transparency 0 */
							visibility: hidden;
							transition: opacity 0.4s, visibility 0.4s;
						}

						.discord-2022--nav:has(.discord-2022--nav_dd_trigger.w--open) .discord-2022--nav_blur {
							opacity: 1;
							visibility: visible;
						}


						.discord-2022--nav_burger_content .discord-2022--nav_dd_trigger {
							padding-top: 1.5rem;
							padding-left: 0;
							padding-bottom: 1.5rem;
							border-bottom: none;
							justify-content: space-between;
							flex-grow: 1;
						}

						.discord-2022--nav_burger_content .discord-2022--nav_dd {
							border-bottom: 1px solid #ffffff10;
							border-radius: 0px;
						}

						.discord-2022--nav_menu.discord-2022--is-burger::-webkit-scrollbar {
							width: 0px;
							height: 10px;
						}

						.discord-2022--nav_dd_link-group:last-child {
							border: 0px;
						}

						.discord-2022--dd_nav-link:hover .discord-2022--nav_dd_link_arrow {
							transform: translate(2px, -2px);
						}


						@-moz-document url-prefix() {
							.discord-2022--dropdown-language-list-wr .discord-2022--dropdown-list-container-wr {
								scrollbar-width: thin;
								scrollbar-color: white transparent;
							}
						}

						.discord-2022--dropdown-language-list-wr .discord-2022--dropdown-list-container-wr {
							max-height: 240px !important;
							overflow: auto !important;

						}

						.discord-2022--dropdown-language-list-wr .discord-2022--dropdown-list-container-wr::-webkit-scrollbar {
							width: 6px;
						}

						.discord-2022--dropdown-language-list-wr .discord-2022--dropdown-list-container-wr::-webkit-scrollbar-thumb {
							background-color: white;
							border-radius: 3px;
							margin-right: 30px;
						}

						.discord-2022--dropdown-language-list-wr .discord-2022--dropdown-list-container-wr::-webkit-scrollbar-button {
							display: none;
						}

						.discord-2022--nav:has(.discord-2022--nav_dd_trigger.w--open) .discord-2022--nav_dd_bg {
							transform: scaleY(1);
						}

						.discord-2022--footer_new {

							background-color: #00000000;
						}

						@media screen and (min-width: 1280px) {

							html[lang="ru"] .discord-2022--nav_link {
								font-size: 0.75rem !important;
							}

							html[lang="ru"] .discord-2022--nav_dd_trigger {
								font-size: 0.75rem !important;
							}
						}
					</style>
				</div>
				<div class="styles">
					<div class="global-styles w-embed">
						<style>
							/* Make text look crisper and more legible in all browsers */
							body {
								-webkit-font-smoothing: antialiased;
								-moz-osx-font-smoothing: grayscale;
								font-smoothing: antialiased;
								text-rendering: optimizeLegibility;
							}

							/* Focus state style for keyboard navigation for the focusable elements */
							*[tabindex]:focus-visible,
							input[type="file"]:focus-visible {
								outline: 0.125rem solid #4d65ff;
								outline-offset: 0.125rem;
							}

							/* Get rid of top margin on first element in any rich text element */
							.w-richtext> :not(div):first-child,
							.w-richtext>div:first-child> :first-child {
								margin-top: 0 !important;
							}

							/* Get rid of bottom margin on last element in any rich text element */
							.w-richtext>:last-child,
							.w-richtext ol li:last-child,
							.w-richtext ul li:last-child {
								margin-bottom: 0 !important;
							}

							/* Prevent all click and hover interaction with an element */
							.pointer-events-off {
								pointer-events: none;
							}

							/* Enables all click and hover interaction with an element */
							.pointer-events-on {
								pointer-events: auto;
							}

							/* Create a class of .div-square which maintains a 1:1 dimension of a div */
							.div-square::after {
								content: "";
								display: block;
								padding-bottom: 100%;
							}

							/* Make sure containers never lose their center alignment */
							.container-medium,
							.container-small,
							.container-large {
								margin-right: auto !important;
								margin-left: auto !important;
							}

							/* 
Make the following elements inherit typography styles from the parent and not have hardcoded values. 
Important: You will not be able to style for example "All Links" in Designer with this CSS applied.
Uncomment this CSS to use it in the project. Leave this message for future hand-off.
*/
							/*
a,
.w-input,
.w-select,
.w-tab-link,
.w-nav-link,
.w-dropdown-btn,
.w-dropdown-toggle,
.w-dropdown-link {
  color: inherit;
  text-decoration: inherit;
  font-size: inherit;
}
*/

							/* Apply "..." after 3 lines of text */
							.text-style-3lines {
								display: -webkit-box;
								overflow: hidden;
								-webkit-line-clamp: 3;
								-webkit-box-orient: vertical;
							}

							/* Apply "..." after 2 lines of text */
							.text-style-2lines {
								display: -webkit-box;
								overflow: hidden;
								-webkit-line-clamp: 2;
								-webkit-box-orient: vertical;
							}

							/* Apply "..." after 1 lines of text */
							.text-style-1line {
								display: -webkit-box;
								overflow: hidden;
								-webkit-line-clamp: 1;
								-webkit-box-orient: vertical;
							}

							/* Adds inline flex display */
							.display-inlineflex {
								display: inline-flex;
							}

							/* These classes are never overwritten */
							.hide {
								display: none !important;
							}

							@media screen and (max-width: 991px),
							@media screen and (max-width: 767px),
							@media screen and (max-width: 479px) {

								.hide,
								.hide-tablet {
									display: none !important;
								}
							}

							@media screen and (max-width: 767px) .hide-mobile-landscape {
								display: none !important;
							}
							}

							@media screen and (max-width: 479px) .hide-mobile {
								display: none !important;
							}
							}

							.margin-0 {
								margin: 0rem !important;
							}

							.padding-0 {
								padding: 0rem !important;
							}

							.spacing-clean {
								padding: 0rem !important;
								margin: 0rem !important;
							}

							.margin-top {
								margin-right: 0rem !important;
								margin-bottom: 0rem !important;
								margin-left: 0rem !important;
							}

							.padding-top {
								padding-right: 0rem !important;
								padding-bottom: 0rem !important;
								padding-left: 0rem !important;
							}

							.margin-right {
								margin-top: 0rem !important;
								margin-bottom: 0rem !important;
								margin-left: 0rem !important;
							}

							.padding-right {
								padding-top: 0rem !important;
								padding-bottom: 0rem !important;
								padding-left: 0rem !important;
							}

							.margin-bottom {
								margin-top: 0rem !important;
								margin-right: 0rem !important;
								margin-left: 0rem !important;
							}

							.padding-bottom {
								padding-top: 0rem !important;
								padding-right: 0rem !important;
								padding-left: 0rem !important;
							}

							.margin-left {
								margin-top: 0rem !important;
								margin-right: 0rem !important;
								margin-bottom: 0rem !important;
							}

							.padding-left {
								padding-top: 0rem !important;
								padding-right: 0rem !important;
								padding-bottom: 0rem !important;
							}

							.margin-horizontal {
								margin-top: 0rem !important;
								margin-bottom: 0rem !important;
							}

							.padding-horizontal {
								padding-top: 0rem !important;
								padding-bottom: 0rem !important;
							}

							.margin-vertical {
								margin-right: 0rem !important;
								margin-left: 0rem !important;
							}

							.padding-vertical {
								padding-right: 0rem !important;
								padding-left: 0rem !important;
							}

							a {
								text-decoration: none;
								font-size: inherit;
								color: inherit;
								font-weight: inherit;
							}

							.discord-2022--nav:has(.discord-2022--nav_dd_trigger.w--open) .discord-2022--nav_dd_bg {
								transform: scaleY(1);
							}
						</style>
					</div>
					<div class="project-styles w-embed">
						<style>
							.btn:hover .button-arrow {
								transform: translateX(2px);
							}

							.btn.is-back:hover .button-arrow-back {
								transform: translateX(-2px);
							}

							.btn_hover {
								opacity: 0;
								transition: opacity 0.5s ease;
							}

							.btn:hover .btn_hover {
								opacity: 1;
							}

							.btn:active .btn_hover {
								opacity: 1;
							}

							.btn:focuse .btn_hover {
								opacity: 1;
							}

							.featured_image {
								transition: transform 0.3s ease;
							}

							.featured_main-card:hover .featured_image {
								transform: scale(1.15);
							}


							@media (max-width: 767px) {
								.featured_main-card:hover .featured_image {
									transform: scale(1.0);
								}
							}
						</style>
					</div>
				</div>
				<header class="discord-2022--nav">
					<div class="discord-2022--nav_styles w-embed">
						<style>
							/*nav styles*/
							.nav:has(.nav_dd_trigger.w--open) .nav_blur {
								display: block;
								height: 100%;
								transition: opacity 0.4s;
								opacity: 1;

							}

							/* Focus state style for keyboard navigation for the focusable elements */
							*[tabindex]:focus-visible,
							input[type="file"]:focus-visible {
								outline: 0.125rem solid #fff;
								outline-offset: 0.125rem;
							}


							.nav_blur {
								display: block;
								opacity: 0;
								visibility: hidden;
								transition: opacity 0.4s, visibility 0.4s;
							}

							.nav:has(.nav_dd_trigger.w--open) .nav_blur {
								opacity: 1;
								visibility: visible;
							}

							.nav:has(.nav_dd_trigger.w--open) .nav_dd_bg {
								transform: scaleY(1);
							}

							.nav_burger_content .nav_link {
								padding-top: 1.5rem;
								padding-left: 0;
								padding-bottom: 1.5rem;
								border-bottom: 1px solid #ffffff10;
								border-radius: 0px;
								justify-content: left;
							}

							.nav_burger_content .nav_dd_trigger {
								padding-top: 1.5rem;
								padding-left: 0;
								padding-bottom: 1.5rem;
								border-bottom: none;
								justify-content: space-between;
								flex-grow: 1;
							}

							.nav_burger_content .nav_dd {
								border-bottom: 1px solid #ffffff10;
								border-radius: 0px;
							}

							.nav_menu.is-burger::-webkit-scrollbar {
								width: 0px;
								height: 10px;
							}

							.nav_dd_link-group:last-child {
								border: 0px;
							}

							.dd_nav-link:hover .nav_dd_link_arrow {
								transform: translate(2px, -2px);
							}

							@media screen and (max-width: 340px) and (min-width: 240px) {
								.nav_brand {
									width: 7.45rem;
								}
							}
						</style>
					</div>
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
									<li>
										<div data-delay="0" data-hover="true"
											class="discord-2022--nav_dd discord-2022--is-relative w-dropdown">
											<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
												<div>Quests</div>
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
												</div><a tabindex="0" href="/ads/quests"
													class="discord-2022--nav_link_dropdown w-inline-block"></a>
											</div>
											<nav
												class="discord-2022--nav_dd_list discord-2022--is-safety w-dropdown-list">
												<div class="discord-2022--nav_dd_content-wr discord-2022--isnew">
													<div
														class="discord-2022--nav_dd_content_layout discord-2022--top-none discord-2022--1-col">
														<div class="discord-2022--nav_dd_link-group">
															<div class="discord-2022--nav_dd_link_list">
																<div class="discord-2022--nav_dd_link_title">Resources
																</div><a href="/ads/quests"
																	class="discord-2022--dd_nav-link">Advertising</a><a
																	href="/ads/quests-success-stories"
																	class="discord-2022--dd_nav-link">Success
																	Stories</a><a href="/ads/quests-faq"
																	class="discord-2022--dd_nav-link">Quests FAQ</a>
															</div>
														</div>
													</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e5a_62f7ba36cdbc603f1ee0e841efd35ca6_Set%201%2015.webp" loading="eager" alt="" class="discord-2022--nav-dd-decor"/></div>
											</nav>
										</div>
									</li>
									<li>
										<div data-delay="0" data-hover="true"
											class="discord-2022--nav_dd discord-2022--is-relative w-dropdown">
											<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
												<div>Support</div>
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
												</div><a tabindex="0" href="https://support.discord.com/hc/"
													class="discord-2022--nav_link_dropdown w-inline-block"></a>
											</div>
											<nav
												class="discord-2022--nav_dd_list discord-2022--is-safety w-dropdown-list">
												<div class="discord-2022--nav_dd_content-wr discord-2022--isnew">
													<div
														class="discord-2022--nav_dd_content_layout discord-2022--top-none discord-2022--1-col">
														<div class="discord-2022--nav_dd_link-group">
															<div class="discord-2022--nav_dd_link_list">
																<div class="discord-2022--nav_dd_link_title">Resources
																</div><a href="https://support.discord.com/hc"
																	class="discord-2022--dd_nav-link">Help Center</a><a
																	href="https://support.discord.com/hc/en-us/community/topics"
																	class="discord-2022--dd_nav-link">Feedback</a><a
																	href="https://support.discord.com/hc/en-us/requests/new"
																	class="discord-2022--dd_nav-link">Submit a
																	Request</a>
															</div>
														</div>
													</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e4e_ceb9e6714b94e9c3c2a28f342a55be19_Discord_Nelly_Pose2_Flying%201.webp" loading="eager" alt="" class="discord-2022--nav-dd-decor discord-2022--is-support"/></div>
											</nav>
										</div>
									</li>
									<li>
										<div data-delay="0" data-hover="true"
											class="discord-2022--nav_dd discord-2022--is-relative w-dropdown">
											<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
												<div>Blog</div>
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
												</div><a tabindex="0" href="/blog"
													class="discord-2022--nav_link_dropdown w-inline-block"></a>
											</div>
											<nav
												class="discord-2022--nav_dd_list discord-2022--is-safety w-dropdown-list">
												<div class="discord-2022--nav_dd_content-wr discord-2022--isnew">
													<div
														class="discord-2022--nav_dd_content_layout discord-2022--top-none discord-2022--1-col">
														<div class="discord-2022--nav_dd_link-group">
															<div class="discord-2022--nav_dd_link_list">
																<div class="discord-2022--nav_dd_link_title">Collections
																</div><a href="/blog"
																	class="discord-2022--dd_nav-link">Featured</a><a
																	href="/category/community"
																	class="discord-2022--dd_nav-link">Community</a><a
																	href="/category/company"
																	class="discord-2022--dd_nav-link">Discord HQ</a><a
																	href="/category/engineering"
																	class="discord-2022--dd_nav-link">Engineering &amp;
																	Developers</a><a href="/category/how-to-discord"
																	class="discord-2022--dd_nav-link">How to
																	Discord</a><a href="/category/safety"
																	class="discord-2022--dd_nav-link">Policy &amp;
																	Safety</a><a href="/category/product"
																	class="discord-2022--dd_nav-link">Product &amp;
																	Features</a>
															</div>
														</div>
													</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e53_4c74f967510e9480b8118b05964a3039_Clyde%20Cube.webp" loading="eager" alt="" class="discord-2022--nav-dd-decor discord-2022--is-blog"/></div>
											</nav>
										</div>
									</li>
									<li>
										<div data-delay="0" data-hover="true"
											class="discord-2022--nav_dd discord-2022--is-relative w-dropdown">
											<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
												<div>Developers</div>
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
												</div><a tabindex="0" href="/developers"
													class="discord-2022--nav_link_dropdown w-inline-block"></a>
											</div>
											<nav
												class="discord-2022--nav_dd_list discord-2022--is-safety w-dropdown-list">
												<div class="discord-2022--nav_dd_content-wr discord-2022--isnew">
													<div
														class="discord-2022--nav_dd_content_layout discord-2022--top-none discord-2022--2-col">
														<div class="discord-2022--nav_dd_link-group">
															<div class="discord-2022--nav_dd_link_list">
																<div class="discord-2022--nav_dd_link_title">Featured
																</div><a href="/developers/social-sdk"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Discord Social SDK</div>
																</a><a href="/developers/build"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Apps and Activities</div>
																</a>
															</div>
															<div
																class="discord-2022--nav_dd_link_line discord-2022--is-n">
															</div>
															<div class="discord-2022--nav_dd_link_list">
																<div class="discord-2022--nav_dd_link_title">
																	Documentation</div><a href="/developers"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Developer Home</div>
																</a><a href="/developers/docs/intro"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Developer Documentation</div>
																	<div
																		class="discord-2022--nav_dd_link_arrow w-embed">
																		<svg width="16" height="16" viewBox="0 0 16 16"
																			fill="none"
																			xmlns="http://www.w3.org/2000/svg">
																			<path d="M5 4H12M12 4V11M12 4L4 12"
																				stroke="white" stroke-opacity="0.5"
																				stroke-width="2" stroke-linecap="round"
																				stroke-linejoin="round" />
																		</svg></div>
																</a><a href="/developers/applications"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Developer Applications</div>
																	<div
																		class="discord-2022--nav_dd_link_arrow w-embed">
																		<svg width="16" height="16" viewBox="0 0 16 16"
																			fill="none"
																			xmlns="http://www.w3.org/2000/svg">
																			<path d="M5 4H12M12 4V11M12 4L4 12"
																				stroke="white" stroke-opacity="0.5"
																				stroke-width="2" stroke-linecap="round"
																				stroke-linejoin="round" />
																		</svg></div>
																</a><a href="https://support-dev.discord.com/hc/en-us"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Developer Help Center</div>
																	<div
																		class="discord-2022--nav_dd_link_arrow w-embed">
																		<svg width="16" height="16" viewBox="0 0 16 16"
																			fill="none"
																			xmlns="http://www.w3.org/2000/svg">
																			<path d="M5 4H12M12 4V11M12 4L4 12"
																				stroke="white" stroke-opacity="0.5"
																				stroke-width="2" stroke-linecap="round"
																				stroke-linejoin="round" />
																		</svg></div>
																</a><a href="/developers/developer-newsletter"
																	class="discord-2022--dd_nav-link w-inline-block">
																	<div>Developer Newsletter</div>
																</a>
															</div>
														</div>
													</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e56_0e84da058e5f250e47f9418ec9d928ef_Clyde%20(1).webp" loading="eager" alt="" class="discord-2022--nav-dd-decor discord-2022--is-build"/></div>
											</nav>
										</div>
									</li>
									<li><a href="/careers"
											class="discord-2022--nav_link discord-2022--is_careers">Careers</a></li>
								</ul>
							</div>
							<div class="discord-2022--nav_buttons-wr_new"><a id="login" data-track="login"
									data-track-nav="login" href="https://discord.com/app"
									class="discord-2022--button-nav discord-2022--login-button-js discord-2022--new w-button">Log
									In</a><a data-track-nav="login" data-track="login" href="https://discord.com/app"
									class="discord-2022--menu-button-login discord-2022--login-button-js discord-2022--abc">Log
									in</a></div>
							<div id="w-node-_61d7cab4-1188-1fe8-8375-476738b4ecb9-38b4ec56"
								class="discord-2022--nav_burger_button">
								<div data-w-id="61d7cab4-1188-1fe8-8375-476738b4ecba"
									class="discord-2022--nav_burger_trigger">
									<div class="discord-2022--close_icon w-embed"><svg width="24" height="24"
											viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
											<path fill-rule="evenodd" clip-rule="evenodd"
												d="M4 6C4 5.44772 4.44772 5 5 5H19C19.5523 5 20 5.44772 20 6C20 6.55228 19.5523 7 19 7H5C4.44772 7 4 6.55228 4 6ZM4 12C4 11.4477 4.44772 11 5 11H19C19.5523 11 20 11.4477 20 12C20 12.5523 19.5523 13 19 13H5C4.44772 13 4 12.5523 4 12ZM5 17C4.44772 17 4 17.4477 4 18C4 18.5523 4.44772 19 5 19H12C12.5523 19 13 18.5523 13 18C13 17.4477 12.5523 17 12 17H5Z"
												fill="white" />
										</svg></div>
								</div>
								<div data-w-id="61d7cab4-1188-1fe8-8375-476738b4ecbc"
									class="discord-2022--nav_burger_trigger-black">
									<div class="discord-2022--close_icon w-embed"><svg width="24" height="24"
											viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
											<path fill-rule="evenodd" clip-rule="evenodd"
												d="M4 6C4 5.44772 4.44772 5 5 5H19C19.5523 5 20 5.44772 20 6C20 6.55228 19.5523 7 19 7H5C4.44772 7 4 6.55228 4 6ZM4 12C4 11.4477 4.44772 11 5 11H19C19.5523 11 20 11.4477 20 12C20 12.5523 19.5523 13 19 13H5C4.44772 13 4 12.5523 4 12ZM5 17C4.44772 17 4 17.4477 4 18C4 18.5523 4.44772 19 5 19H12C12.5523 19 13 18.5523 13 18C13 17.4477 12.5523 17 12 17H5Z"
												fill="currentColor" />
										</svg></div>
								</div>
								<div class="discord-2022--nav_burger_list">
									<div class="discord-2022--nav_burger_content">
										<div class="discord-2022--nav_burger_top"><a href="https://discord.com/"
												class="discord-2022--nav_brand discord-2022--is-burger w-nav-brand"><img loading="lazy" src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e58_50421399b7d807a39b976375b8b2f21e_Symbol.svg" alt="" class="discord-2022--nav-brand_logo-burger"/></a><a
													href="#" title="Close menu"
													data-w-id="61d7cab4-1188-1fe8-8375-476738b4ecc3"
													class="discord-2022--nav_burger_close w-nav-brand">
													<div class="discord-2022--close_icon w-embed"><svg width="24"
															height="24" viewBox="0 0 24 24" fill="none"
															xmlns="http://www.w3.org/2000/svg">
															<path d="M18 6L6 18M6 6L18 18" stroke="white"
																stroke-width="2" stroke-linecap="round" />
														</svg></div>
												</a></div>
										<ul role="list" class="discord-2022--nav_menu discord-2022--is-burger">
											<li><a href="https://discord.com/download"
													class="discord-2022--nav_link">Download</a></li>
											<li><a href="https://discord.com/nitro"
													class="discord-2022--nav_link">Nitro</a></li>
											<li><a href="https://discord.com/servers"
													class="discord-2022--nav_link">Discover</a></li>
											<li>
												<div data-delay="0" data-hover="false"
													class="discord-2022--nav_dd w-dropdown">
													<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
														<div>Safety</div>
														<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
																height="16" viewBox="0 0 16 16" fill="none"
																xmlns="http://www.w3.org/2000/svg">
																<path d="M12 6L8 10L4 6" stroke="white"
																	stroke-opacity="0.5" stroke-width="2"
																	stroke-linecap="round" stroke-linejoin="round" />
															</svg></div>
													</div>
													<nav
														class="discord-2022--nav_dd_list discord-2022--is-burger w-dropdown-list">
														<div class="discord-2022--nav_dd_content-wr">
															<div class="discord-2022--nav_dd_separator"></div>
															<div class="discord-2022--nav_dd_content_layout">
																<div class="discord-2022--nav_dd_link-group"><a
																		href="/safety"
																		class="discord-2022--nav_dd_link-group_title discord-2022--is-new">Safety</a>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Resources</div><a
																			href="/safety-family-center"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Family Center</div>
																		</a><a href="/safety-library"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Safety
																			Library</a><a href="/safety-news"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Safety
																			News</a><a href="/safety-teen-charter"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Teen Charter</div>
																		</a>
																	</div>
																</div>
																<div class="discord-2022--nav_dd_link-group">
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Hubs</div><a href="/safety-parents"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Parent Hub</div>
																		</a><a href="/safety-policies"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Policy Hub</div>
																		</a><a href="/safety-privacy"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Privacy Hub</div>
																		</a><a href="/safety-transparency"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Transparency Hub</div>
																		</a><a href="/safety-wellbeing"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Wellbeing Hub</div>
																		</a>
																	</div>
																</div>
															</div>
															<div class="discord-2022--nav_dd_list_bg"></div>
														</div>
													</nav>
												</div>
											</li>
											<li>
												<div data-delay="0" data-hover="false"
													class="discord-2022--nav_dd w-dropdown">
													<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
														<div>Quests</div>
														<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
																height="16" viewBox="0 0 16 16" fill="none"
																xmlns="http://www.w3.org/2000/svg">
																<path d="M12 6L8 10L4 6" stroke="white"
																	stroke-opacity="0.5" stroke-width="2"
																	stroke-linecap="round" stroke-linejoin="round" />
															</svg></div>
													</div>
													<nav
														class="discord-2022--nav_dd_list discord-2022--is-burger w-dropdown-list">
														<div class="discord-2022--nav_dd_content-wr">
															<div class="discord-2022--nav_dd_separator"></div>
															<div class="discord-2022--nav_dd_content_layout">
																<div class="discord-2022--nav_dd_link-group"><a
																		href="https://discord.com/ads/quests"
																		class="discord-2022--nav_dd_link-group_title discord-2022--is-new">Quests</a>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Resources</div><a
																			href="https://discord.com/ads/quests"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Advertising</a><a
																			href="https://discord.com/ads/quests-success-stories"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Success
																			Stories</a><a
																			href="https://discord.com/ads/quests-faq"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Quests
																			FAQ</a>
																	</div>
																</div>
															</div>
															<div class="discord-2022--nav_dd_list_bg"></div>
														</div>
													</nav>
												</div>
											</li>
											<li>
												<div data-delay="0" data-hover="false"
													class="discord-2022--nav_dd w-dropdown">
													<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
														<div>Support</div>
														<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
																height="16" viewBox="0 0 16 16" fill="none"
																xmlns="http://www.w3.org/2000/svg">
																<path d="M12 6L8 10L4 6" stroke="white"
																	stroke-opacity="0.5" stroke-width="2"
																	stroke-linecap="round" stroke-linejoin="round" />
															</svg></div>
													</div>
													<nav
														class="discord-2022--nav_dd_list discord-2022--is-burger w-dropdown-list">
														<div class="discord-2022--nav_dd_content-wr">
															<div class="discord-2022--nav_dd_separator"></div>
															<div class="discord-2022--nav_dd_content_layout">
																<div class="discord-2022--nav_dd_link-group"><a
																		href="https://support.discord.com/hc"
																		class="discord-2022--nav_dd_link-group_title discord-2022--is-new">Support</a>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Resources</div><a
																			href="https://support.discord.com/hc"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Help
																			Center</a><a
																			href="https://support.discord.com/hc/en-us/community/topics"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Feedback</a><a
																			href="https://support.discord.com/hc/en-us/requests/new"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Submit
																			a Request</a>
																	</div>
																</div>
															</div>
														</div>
													</nav>
												</div>
											</li>
											<li>
												<div data-delay="0" data-hover="false"
													class="discord-2022--nav_dd w-dropdown">
													<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
														<div>Blog</div>
														<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
																height="16" viewBox="0 0 16 16" fill="none"
																xmlns="http://www.w3.org/2000/svg">
																<path d="M12 6L8 10L4 6" stroke="white"
																	stroke-opacity="0.5" stroke-width="2"
																	stroke-linecap="round" stroke-linejoin="round" />
															</svg></div>
													</div>
													<nav
														class="discord-2022--nav_dd_list discord-2022--is-burger w-dropdown-list">
														<div class="discord-2022--nav_dd_content-wr">
															<div class="discord-2022--nav_dd_separator"></div>
															<div class="discord-2022--nav_dd_content_layout">
																<div class="discord-2022--nav_dd_link-group"><a
																		href="https://discord.com/blog"
																		class="discord-2022--nav_dd_link-group_title discord-2022--is-new">Blog</a>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Collections</div><a
																			href="https://discord.com/blog"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Featured</a><a
																			href="https://discord.com/category/community"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Community</a><a
																			href="https://discord.com/category/company"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Discord
																			HQ</a><a
																			href="https://discord.com/category/engineering"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Engineering
																			&amp; Developers</a><a
																			href="https://discord.com/category/how-to-discord"
																			class="discord-2022--dd_nav-link discord-2022--is-new">How
																			to Discord</a><a
																			href="https://discord.com/category/safety"
																			class="discord-2022--dd_nav-link discord-2022--is-new">Policy
																			&amp; Safety</a><a
																			href="https://discord.com/category/product"
																			class="discord-2022--dd_nav-link">Product
																			&amp; Features</a>
																	</div>
																</div>
															</div>
															<div class="discord-2022--nav_dd_list_bg"></div>
														</div>
													</nav>
												</div>
											</li>
											<li>
												<div data-delay="0" data-hover="false"
													class="discord-2022--nav_dd w-dropdown">
													<div class="discord-2022--nav_dd_trigger w-dropdown-toggle">
														<div>Developers</div>
														<div class="discord-2022--nav_dd_arrow w-embed"><svg width="16"
																height="16" viewBox="0 0 16 16" fill="none"
																xmlns="http://www.w3.org/2000/svg">
																<path d="M12 6L8 10L4 6" stroke="white"
																	stroke-opacity="0.5" stroke-width="2"
																	stroke-linecap="round" stroke-linejoin="round" />
															</svg></div>
													</div>
													<nav
														class="discord-2022--nav_dd_list discord-2022--is-burger w-dropdown-list">
														<div class="discord-2022--nav_dd_content-wr">
															<div class="discord-2022--nav_dd_separator"></div>
															<div class="discord-2022--nav_dd_content_layout">
																<div class="discord-2022--nav_dd_link-group"><a
																		href="https://discord.com/developers"
																		class="discord-2022--nav_dd_link-group_title discord-2022--is-new">Developers</a>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Featured</div><a
																			href="https://discord.com/developers/social-sdk"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Discord Social SDK</div>
																		</a><a
																			href="https://discord.com/developers/build"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Apps and Activities</div>
																		</a>
																	</div>
																	<div class="discord-2022--nav_dd_link_line"></div>
																	<div class="discord-2022--nav_dd_link_list">
																		<div
																			class="discord-2022--nav_dd_link_title discord-2022--is-new">
																			Documentation</div><a
																			href="https://discord.com/developers"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Developer Home</div>
																		</a><a
																			href="https://discord.com/developers/docs/intro"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Developer Documentation</div>
																			<div
																				class="discord-2022--nav_dd_link_arrow w-embed">
																				<svg width="16" height="16"
																					viewBox="0 0 16 16" fill="none"
																					xmlns="http://www.w3.org/2000/svg">
																					<path d="M5 4H12M12 4V11M12 4L4 12"
																						stroke="white"
																						stroke-opacity="0.5"
																						stroke-width="2"
																						stroke-linecap="round"
																						stroke-linejoin="round" />
																				</svg></div>
																		</a><a
																			href="https://discord.com/developers/applications"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Developer Applications</div>
																			<div
																				class="discord-2022--nav_dd_link_arrow w-embed">
																				<svg width="16" height="16"
																					viewBox="0 0 16 16" fill="none"
																					xmlns="http://www.w3.org/2000/svg">
																					<path d="M5 4H12M12 4V11M12 4L4 12"
																						stroke="white"
																						stroke-opacity="0.5"
																						stroke-width="2"
																						stroke-linecap="round"
																						stroke-linejoin="round" />
																				</svg></div>
																		</a><a
																			href="https://support-dev.discord.com/hc/en-us"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Developer Help Center</div>
																			<div
																				class="discord-2022--nav_dd_link_arrow w-embed">
																				<svg width="16" height="16"
																					viewBox="0 0 16 16" fill="none"
																					xmlns="http://www.w3.org/2000/svg">
																					<path d="M5 4H12M12 4V11M12 4L4 12"
																						stroke="white"
																						stroke-opacity="0.5"
																						stroke-width="2"
																						stroke-linecap="round"
																						stroke-linejoin="round" />
																				</svg></div>
																		</a><a href="/developers/developer-newsletter"
																			class="discord-2022--dd_nav-link discord-2022--is-new w-inline-block">
																			<div>Developer Newsletter</div>
																		</a>
																	</div>
																</div>
															</div>
														</div>
													</nav>
												</div>
											</li>
											<li><a href="https://discord.com/careers"
													class="discord-2022--nav_link">Careers</a></li>
										</ul>
										<div class="discord-2022--nav_burger_bottom"><a href="https://discord.com/app"
												data-track-nav="login" data-track="login"
												class="discord-2022--button-nav discord-2022--is-burger discord-2022--is-ghost discord-2022--login-button-js w-nav-brand">
												<div>Log In</div>
											</a><a href="#" data-track-download="Download Page"
												class="discord-2022--button-nav discord-2022--is-burger discord-2022--download-button w-nav-brand">
												<div class="discord-2022--embed-center w-embed"><svg width="16"
														height="16" viewBox="0 0 16 16" fill="none"
														xmlns="http://www.w3.org/2000/svg">
														<path fill-rule="evenodd" clip-rule="evenodd"
															d="M9 1C9 0.447715 8.55228 0 8 0C7.44772 0 7 0.447715 7 1V7.58579L4.70711 5.29289C4.31658 4.90237 3.68342 4.90237 3.29289 5.29289C2.90237 5.68342 2.90237 6.31658 3.29289 6.70711L7.29289 10.7071C7.68342 11.0976 8.31658 11.0976 8.70711 10.7071L12.7071 6.70711C13.0976 6.31658 13.0976 5.68342 12.7071 5.29289C12.3166 4.90237 11.6834 4.90237 11.2929 5.29289L9 7.58579V1ZM2 12C2 11.4477 1.55228 11 1 11C0.447715 11 0 11.4477 0 12V13C0 14.6569 1.34315 16 3 16H13C14.6569 16 16 14.6569 16 13V12C16 11.4477 15.5523 11 15 11C14.4477 11 14 11.4477 14 12V13C14 13.5523 13.5523 14 13 14H3C2.44772 14 2 13.5523 2 13V12Z"
															fill="black" />
													</svg></div>
												<div>App Store</div>
											</a>
											<div class="discord-2022--nav_burger_bottom_grad"></div>
										</div>
									</div>
								</div>
								<div class="discord-2022--nav_burger_bottom_grad"></div>
							</div>
						</div>
					</div>
					<div class="discord-2022--nav_blur"></div>
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
				<div class="container-1260px _980px">
					<div style="background-image:url(&quot;https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639752c1cb940a795f15f472_COMM_PortalHeaders_Manage_v2.png&quot;)"
						class="hero-image-safeties"></div>
					<div class="w-layout-grid grid-branding">
						<div id="w-node-_9427f693-c30b-3d3b-ab1e-76ec923ea3b0-f36a026f" class="title-wrapper">
							<div>
								<div class="link-blue-16px safetie">Manage</div>
								<h1 class="new-h1 left">Establishing Trust with Social Media Connections and Roles</h1>
								<div class="padding-32px"></div>
							</div>
							<p class="paragraph-small font-grey build align-left w-dyn-bind-empty"></p>
						</div>
					</div>
					<div class="rich-content-wr">
						<div class="rich-content-left">
							<div id="title-1" class="rich-safeties w-richtext">
								<p>Would your community members like to know more about who they’re talking to? Are
									there details they’d find important, interesting, or reassuring when interacting
									with others on the server?</p>
								<p>Then your community might benefit from new features that have been added to Discord
									Connections - a service that allows members to share info about their third-party
									accounts and profiles.</p>
								<h2><strong>Connections: Basics and New Features</strong></h2>
								<p>Connections allow you to link certain external accounts to your Discord account.
									Adding account authenticity that it belongs to you, so anyone checking your profile
									will know that’s really your Steam account, X account, Instagram profile, and so on.
								</p>
								<p>So what’s new? Firstly, you can now add certain details that might help prove your
									bonafides to others. For example, if you’ve had your Steam account for longer than a
									month, you might display that detail to help other people decide whether your
									accomplishments in games are legitimate. Or it could just be a way to share some
									fast facts—your year and major if you’re on a school server, your stats on a
									Fortnite server, and so on.</p>
								<figure class="w-richtext-align-center w-richtext-figure-type-image">
									<div>
										<img src="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/63975448e7938b245160df67_Screen%20Shot%202022-12-12%20at%208.18.02%20AM.png" loading="lazy" alt=""/></div>
										<figcaption>Connect your accounts</figcaption>
								</figure>
							</div>
							<div id="title-2" class="rich-safeties w-richtext">
								<h3><strong>Linked Roles</strong></h3>
								<p>Additionally, Connections now allow admins to create roles based on these details:
									e.g., a role only for people who linked their Steam account. Not only will these
									Linked Roles immediately show who meets certain criteria, they make it easy to craft
									exclusive channels and spaces that only authenticated members can access. Hopefully,
									restricting these spaces to persons with Linked Roles makes your community members
									feel more confident connecting with one another.</p>
							</div>
							<div id="title-3" class="rich-safeties w-richtext">
								<h3><strong>Linked Roles with Apps</strong></h3>
								<p>We support several third-party services natively within Discord (Facebook, X, Steam,
									etc). We’re also opening up Connections to third-party app developers, which will
									allow anyone to create a Connection for members to add (and admins to check) when
									granting Linked Roles. If you’re interested in adding connection capabilities to an
									existing app or a new app, learn more in the Role Connections API doc <a
										href="https://discord.com/developers/docs/resources/application-role-connection-metadata">here</a>
									and our How-To-Guide <a
										href="https://discord.com/developers/docs/tutorials/configuring-app-metadata-for-linked-roles">here</a>.
								</p>
								<p>We plan to extend this for more developers in the future so that they can add
									connection capabilities for their own apps. To learn more about the App Directory
									and how to access apps check out the App Directory guide <a
										href="https://support.discord.com/hc/en-us/articles/9360431966359-Welcome-to-the-App-Directory-#h_01GFK7PTNE49GJ3GZEX5265EXQ">here</a>.
								</p>
								<figure class="w-richtext-align-center w-richtext-figure-type-image">
									<div>
										<img src="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639762cc5b560f714f07ffb7_Add_Connection_Modal_NEW.png" loading="lazy" alt=""/></div>
								</figure>
								<p>For more information on Connections, adding details, and using Linked Roles on your
									server, you can <a
										href="https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Community-Members">visit
										our Help center article here</a>!</p>
							</div>
							<div id="title-4" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div id="title-5" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div id="title-6" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div id="title-7" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div id="title-8" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div id="title-9" class="rich-safeties w-condition-invisible w-dyn-bind-empty w-richtext">
							</div>
							<div class="author-wrapper w-condition-invisible">
								<div class="padding-40px"></div>
								<h3 class="new-h3 author w-dyn-bind-empty"></h3>
								<div class="position-author w-dyn-bind-empty"></div>
								<div class="date-publiscin">December 12, 2022</div>
							</div>
						</div>
						<div class="rich-content-right safety">
							<div class="paragraph-medium bot-marg left">Contents</div><a href="#title-1"
								class="title-menu-anchor">Connections: Basics and New Features</a><a href="#title-2"
								class="title-menu-anchor">Linked Roles</a><a href="#title-3"
								class="title-menu-anchor">Linked Roles with Apps</a><a href="#title-4"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a><a href="#title-5"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a><a href="#title-6"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a><a href="#title-7"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a><a href="#title-8"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a><a href="#title-9"
								class="title-menu-anchor w-condition-invisible w-dyn-bind-empty"></a>
						</div>
					</div>
				</div>
			</div>
			<div class="section-list w-condition-invisible">
				<div class="container-1260px _980px">
					<div class="how-wrapper">
						<div class="how-wr-content">
							<div id="w-node-b3c6518f-bd8e-ede9-71ac-6a79ae79202b-f36a026f" class="how-content-left">
								<h1 class="new-h2 heading2-48pxwhite w-dyn-bind-empty"></h1>
								<div class="p-400-16-white w-dyn-bind-empty"></div>
							</div>
							<div id="w-node-a70e607f-2287-f774-4b08-f603e46889ee-f36a026f" class="how-content-right">
								<div class="how-rich w-dyn-bind-empty w-richtext"></div>
							</div>
						</div>
					</div>
				</div>
			</div>
			<div class="section-white grey w-condition-invisible">
				<div class="container-1260px is-center">
					<div class="flex-horizontal-r-l">
						<h3 class="new-h2 bottom-none _48px font-ginto is-text-center heading-32px-black">Related
							articles</h3>
					</div>
					<div class="padding-40px"></div>
					<div id="cms-collection" class="safety-wrapper">
						<div class="w-dyn-list">
							<div class="w-dyn-empty">
								<div>No items found.</div>
							</div>
						</div>
						<div class="padding-40px hidden"></div><a href="#"
							class="new-dark-button gdr max-440 hidden w-inline-block">
							<div class="button-text-big">Show more articles</div>
						</a>
					</div>
					<div id="search-result-content" class="cms-search-results"></div>
					<div id="search-result-empty" class="no-match-results">
						<div class="no-match">No matching results.</div>
					</div>
				</div>
			</div>
		</div>
		<header class="navbarsection hidden">
			<div class="headercontainer"><a href="/wip-private/community-old"
					data-track-nav="article-header-discord-logo"
					class="brandlogo w-nav-brand"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/623b7212c8af466ce928d285_DiscordLogoDark.svg" loading="lazy" alt="Discord Creator Portal" class="logo"/></a>
					<div class="nav-menu"><a data-track-download="header-download" href="https://discord.com/download"
							class="navlink download-button">Download</a><a data-track-nav="header-why-discord"
							href="https://discord.com/why-discord-is-different" class="navlink">Why Discord</a><a
							data-track-nav="header-nitro" href="https://discord.com/nitro" class="navlink">Nitro</a><a
							data-track-nav="header-safety" href="https://discord.com/safetycenter"
							class="navlink">Safety</a><a data-track-nav="header-support"
							href="https://support.discord.com/hc/en-us" class="navlink">Support</a></div>
					<div class="headermenus"><a id="login-or-open-button" data-track-nav="navbar-login-button"
							href="//discord.com/login"
							class="button login-button-blurple login-button-js w-button">Login</a><a
							data-track-nav="header-mobile-menu" href="#"
							class="mobilemenuopenbutton w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/623bb864e2419bfebacacc59_menu-black.svg" loading="lazy" width="32" alt="Menu" class="mobilemenuopenicon"/></a>
					</div>
			</div>
		</header>
		<div class="bodyseparator hidde"></div>
		<div class="bodysectioncontainer articlepage hidden">
			<div class="toccontainer">
				<div class="tocheader">
					<h1 class="maincontentheader">COMMUNITY RESOURCES</h1><a data-track-nav="header-mobile-menu-close"
						href="#"
						class="mobilemenuclosebutton w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/623e5258ecca6c9581c3904b_Close.svg" loading="lazy" alt="Icon for closing the menu" class="nav-close"/></a>
				</div>
				<div class="tocpillarcontent">
					<h2 class="tocleveloneitem">Build</h2>
					<div class="tocleveltwo">
						<div class="w-dyn-list">
							<div role="list" class="w-dyn-items">
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/getting-started-as-a-server-admin"
											class="tocleveltwoitem">Getting Started as a Server Admin &amp; Running Your
											Server</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/getting-set-up"
											class="tocleveltwoitem">Setting Up Your Discord Server - How to Get
											Started</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/securing-your-server"
											class="tocleveltwoitem">Securing Your Discord Server &amp; Creating a Safe
											Community</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/building-a-home-for-your-community"
											class="tocleveltwoitem">Making a Great Server &amp; Building a Home for Your
											Community</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/community-onboarding"
											class="tocleveltwoitem">Community Onboarding: Welcoming New Members</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/channels-every-community-server-should-have"
											class="tocleveltwoitem">Essential Channels Every Community Server Should
											Have</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/customizing-branding-your-server"
											class="tocleveltwoitem">Customizing &amp; Branding Your Discord Server</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/the-app-directory"
											class="tocleveltwoitem">The Discord App Directory</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/server-information-and-announcement-channels"
											class="tocleveltwoitem">Server Information and Announcement Channels</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/basic-channel-setup"
											class="tocleveltwoitem">Basic Channel Setup</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/utilizing-role-colors"
											class="tocleveltwoitem">Utilizing Role Colors</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/channel-categories-and-names"
											class="tocleveltwoitem">Channel Categories and Names</a></div>
								</div>
							</div>
						</div>
					</div>
				</div>
				<div class="tocpillarcontent">
					<h2 class="tocleveloneitem">Engage</h2>
					<div class="tocleveltwo">
						<div class="w-dyn-list">
							<div role="list" class="w-dyn-items">
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/onboarding-new-members"
											class="tocleveltwoitem">Onboarding New Members &amp; Helping Your Community
											Find Its Way</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/regularly-engaging-your-community"
											class="tocleveltwoitem">Regularly Engaging Your Community</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/creating-value-with-conversation"
											class="tocleveltwoitem">Creating Value with Conversation</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/co-creating-with-your-community"
											class="tocleveltwoitem">Co-Creating with Your Community - Events, Art &amp;
											Feedback</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/turning-your-social-posts-into-community-activations"
											class="tocleveltwoitem">Turning Your Social Posts into Community
											Activations</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/using-roles-to-increase-engagement"
											class="tocleveltwoitem">Using Roles to Increase Server Engagement</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/hosting-voice-events"
											class="tocleveltwoitem">Hosting Voice Events with Stage &amp; Voice
											Channels</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/getting-the-most-out-of-stages"
											class="tocleveltwoitem">Getting the Most Out of Stage Channels</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/understanding-server-insights"
											class="tocleveltwoitem">Understanding Server Insights</a></div>
								</div>
							</div>
						</div>
					</div>
				</div>
				<div class="tocpillarcontent">
					<h2 class="tocleveloneitem">Grow</h2>
					<div class="tocleveltwo">
						<div class="w-dyn-list">
							<div role="list" class="w-dyn-items">
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/welcoming-newcomers"
											class="tocleveltwoitem">Welcoming Server Newcomers</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/growing-and-promoting-your-discord-server"
											class="tocleveltwoitem">Growing and Promoting Your Discord Server</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/growing-your-server-with-invites"
											class="tocleveltwoitem">Brand, Optimize and Grow Your Server With
											Invites</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/growing-your-community-through-member-referrals"
											class="tocleveltwoitem">Growing Your Server Community Through Member
											Referrals</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/growing-your-server-with-social-media"
											class="tocleveltwoitem">Growing Your Server With Social Media</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/growing-your-community-through-server-insights"
											class="tocleveltwoitem">Growing Your Community Through Server Insights &amp;
											Analytics</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/designing-effective-events"
											class="tocleveltwoitem">Designing Effective Events to Bring Your Community
											Together</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/growing-your-server-through-community-events"
											class="tocleveltwoitem">Growing Your Server Through Community Events</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/understanding-event-metrics"
											class="tocleveltwoitem">Understanding Event Metrics</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/understanding-your-community-through-insights"
											class="tocleveltwoitem">Understanding Your Community Through Insights</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/bringing-other-communities-to-discord"
											class="tocleveltwoitem">Bringing Other Communities to Discord</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/using-insights-to-improve-community-growth-engagement"
											class="tocleveltwoitem">Using Insights to Improve Community Growth and
											Engagement</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/ethical-community-growth"
											class="tocleveltwoitem">Ethical Community Growth</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/understanding-community-partnerships"
											class="tocleveltwoitem">Community Partnerships</a></div>
								</div>
							</div>
						</div>
					</div>
				</div>
				<div class="tocpillarcontent">
					<h2 class="tocleveloneitem">Manage</h2>
					<div class="tocleveltwo">
						<div class="w-dyn-list">
							<div role="list" class="w-dyn-items">
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/community-management-the-basics"
											class="tocleveltwoitem">The Basics of a Safe, Active Server</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/keeping-your-community-safe"
											class="tocleveltwoitem">Keeping Your Server &amp; Community Safe</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/establishing-trust-with-connections-connection-details-and-linked-roles"
											aria-current="page" class="tocleveltwoitem w--current">Establishing Trust
											with Social Media Connections and Roles</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/identifying-assigning-moderators"
											class="tocleveltwoitem">Identifying and Assigning Server Moderators</a>
									</div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a
											href="/community/automating-moderation-community-support"
											class="tocleveltwoitem">Moderation &amp; Community Support to Manage Your
											Server</a></div>
								</div>
								<div role="listitem" class="w-dyn-item">
									<div class="tocleveltwoitem"><a href="/community/learning-more-about-your-community"
											class="tocleveltwoitem">Learning More About Your Community</a></div>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
			<div class="mainsection article">
				<div class="breadcrumbs">
					<h3 class="breadcrumb">Creator Portal</h3>
					<h4 class="breadcrumb lastbreadcrumb">Manage</h4>
				</div><img src="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639752c1cb940a795f15f472_COMM_PortalHeaders_Manage_v2.png" loading="lazy" alt="" class="articlehero"/>
				<div id="main-content" class="articlemain centered">
					<h1 class="articletitle">Establishing Trust with Social Media Connections and Roles</h1>
					<h3 class="articledescription w-dyn-bind-empty"></h3>
					<div class="articlebody w-richtext">
						<p>Would your community members like to know more about who they’re talking to? Are there
							details they’d find important, interesting, or reassuring when interacting with others on
							the server?</p>
						<p>Then your community might benefit from new features that have been added to Discord
							Connections - a service that allows members to share info about their third-party accounts
							and profiles.</p>
						<h3><strong>Connections: Basics and New Features</strong></h3>
						<p>Connections allow you to link certain external accounts to your Discord account. Adding
							account authenticity that it belongs to you, so anyone checking your profile will know
							that’s really your Steam account, X account, Instagram profile, and so on.</p>
						<p>So what’s new? Firstly, you can now add certain details that might help prove your bonafides
							to others. For example, if you’ve had your Steam account for longer than a month, you might
							display that detail to help other people decide whether your accomplishments in games are
							legitimate. Or it could just be a way to share some fast facts—your year and major if you’re
							on a school server, your stats on a Fortnite server, and so on.</p>
						<figure class="w-richtext-align-center w-richtext-figure-type-image">
							<div>
								<img src="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/63975448e7938b245160df67_Screen%20Shot%202022-12-12%20at%208.18.02%20AM.png" loading="lazy" alt=""/></div>
								<figcaption>Connect your accounts</figcaption>
						</figure>
						<h4><strong>Linked Roles</strong></h4>
						<p>Additionally, Connections now allow admins to create roles based on these details: e.g., a
							role only for people who linked their Steam account. Not only will these Linked Roles
							immediately show who meets certain criteria, they make it easy to craft exclusive channels
							and spaces that only authenticated members can access. Hopefully, restricting these spaces
							to persons with Linked Roles makes your community members feel more confident connecting
							with one another.</p>
						<h4><strong>Linked Roles with Apps</strong></h4>
						<p>We support several third-party services natively within Discord (Facebook, X, Steam, etc).
							We’re also opening up Connections to third-party app developers, which will allow anyone to
							create a Connection for members to add (and admins to check) when granting Linked Roles. If
							you’re interested in adding connection capabilities to an existing app or a new app, learn
							more in the Role Connections API doc <a
								href="https://discord.com/developers/docs/resources/application-role-connection-metadata">here</a>
							and our How-To-Guide <a
								href="https://discord.com/developers/docs/tutorials/configuring-app-metadata-for-linked-roles">here</a>.
						</p>
						<p>We plan to extend this for more developers in the future so that they can add connection
							capabilities for their own apps. To learn more about the App Directory and how to access
							apps check out the App Directory guide <a
								href="https://support.discord.com/hc/en-us/articles/9360431966359-Welcome-to-the-App-Directory-#h_01GFK7PTNE49GJ3GZEX5265EXQ">here</a>.
						</p>
						<figure class="w-richtext-align-center w-richtext-figure-type-image">
							<div>
								<img src="https://cdn.prod.website-files.com/623b578041aa1f5fc6e3adc2/639762cc5b560f714f07ffb7_Add_Connection_Modal_NEW.png" loading="lazy" alt=""/></div>
						</figure>
						<p>For more information on Connections, adding details, and using Linked Roles on your server,
							you can <a
								href="https://support.discord.com/hc/en-us/articles/8063233404823-Connections-Community-Members">visit
								our Help center article here</a>!</p>
					</div>
					<div class="articlefooter">
						<div class="articlemeta">
							<h4 class="authortitle w-condition-invisible w-dyn-bind-empty"></h4>
							<div class="aboutauthor w-condition-invisible">
								<h5 class="authorname w-dyn-bind-empty"></h5>
								<img height="16" loading="lazy" alt="Author&#x27;s avatar" src="" class="w-dyn-bind-empty"/></div>
								<div class="articleupdatetime">
									<div class="lastupdatelabel">Last updated </div>
									<div class="lastupdatedatetime">December 12, 2022</div>
								</div>
							</div>
							<div class="maincontentseparator articlemeta"></div>
							<div class="articlelinks"><a data-track="article-page-previous-article"
									href="/community/keeping-your-community-safe"
									class="articlelinkprevious w-inline-block">
									<h5 class="articlelinkpreviouslabel">Previous</h5>
									<div class="articlelinkname articlelinkprevious">Keeping Your Server &amp; Community
										Safe</div>
								</a><a data-track="article-page-next-article"
									href="/community/identifying-assigning-moderators"
									class="articlelinknext w-inline-block">
									<h5 class="articlelinknextlabel">Next</h5>
									<div class="articlelinkname articlelinknext">Identifying and Assigning Server
										Moderators</div>
								</a></div>
						</div>
					</div>
				</div>
			</div>
			<div class="section-white-moderation bg safety">
				<div data-w-id="b7b074bc-fd7a-f1ad-7417-f6e9caa61c1b" data-is-ix2-target="1" class="nitro-lottie"
					data-animation-type="lottie"
					data-src="https://cdn.prod.website-files.com/6257adef93867e50d84d30e2/642af22f1bfde053ca776d60_sparkles%20(1).json"
					data-loop="0" data-direction="1" data-autoplay="0" data-renderer="svg" data-duration="3"></div>
				<div class="container-1180px flex-center">
					<h2 class="heading2-48pxwhite max-weight-700px">Stay updated on everything Discord Admin!</h2>
					<div class="container-small">
						<p class="paragraph-small font-white safety text-align-center">We know you wouldn&#x27;t dream
							of missing out on secret mail from us.<br/>Right?</p>
					</div><a href="/admin-newsletter" class="new-dark-button gdr max-440 w-inline-block">
						<div class="button-text-big">Sign up</div>
					</a>
				</div>
			</div>
			<div class="footer_n">
				<div class="discord-2022--footer_new">
					<div class="discord-2022--footer-styles w-embed">
						<!--style>

/* For Chrome and Safari */
.dropdown-language-list-wr::-webkit-scrollbar {
    width: 5px;
}
.dropdown-language-list-wr::-webkit-scrollbar-thumb {
  background-color: white; 
  border-radius: 3px;
}
.dropdown-language-list-wr::-webkit-scrollbar-track {
  margin-top:1.75rem;
  margin-bottom:1.75rem;
}
.dropdown-language-list-wr::-webkit-scrollbar-button { 
     display:none;
}

</style-->
					</div>
					<div class="discord-2022--container-1762">
						<div class="w-layout-grid discord-2022--grid-footer discord-2022--is-new-com">
							<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b922-99d9b91e"
								class="discord-2022--vertical-flex discord-2022--mobile-left discord-2022--is_new"><a
									href="/"
									class="discord-2022--footer-logo-link w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7b9ddb8f70479500e58_50421399b7d807a39b976375b8b2f21e_Symbol.svg" loading="lazy" alt="Home page"/></a>
									<div class="discord-2022--p-footer">Language</div>
									<div class="discord-2022--padding-16px"></div>
									<div data-hover="false" data-delay="0"
										data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b928"
										class="discord-2022--dropdown-language-wr w-dropdown">
										<div class="discord-2022--dropdown-language-btn w-dropdown-toggle">
											<div class="discord-2022--dropdown-language-name">English (US)</div>
											<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b243_13b796631a0178df3105d55d1d629706_Chevron%20Down.svg" loading="lazy" alt="" class="discord-2022--dropdown-language-arrow"/></div>
											<nav class="discord-2022--dropdown-language-list-wr w-dropdown-list">
												<ul role="list" class="discord-2022--dropdown-list-container-wr">
													<li tabindex="0" data-locale="cs"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Čeština</div>
													</li>
													<li tabindex="0" data-locale="da"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Dansk</div>
													</li>
													<li tabindex="0" data-locale="de"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Deutsch</div>
													</li>
													<li tabindex="0" data-locale="en"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">English</div>
													</li>
													<li tabindex="0" data-locale="en-GB"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">English (UK)
														</div>
													</li>
													<li tabindex="0" data-locale="es"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Español</div>
													</li>
													<li tabindex="0" data-locale="es-LA"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Español
															(América Latina)</div>
													</li>
													<li tabindex="0" data-locale="fr"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Français</div>
													</li>
													<li tabindex="0" data-locale="hr"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Hrvatski</div>
													</li>
													<li tabindex="0" data-locale="it"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Italiano</div>
													</li>
													<li tabindex="0" data-locale="lt"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">lietuvių kalba
														</div>
													</li>
													<li tabindex="0" data-locale="hu"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Magyar</div>
													</li>
													<li tabindex="0" data-locale="nl"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Nederlands
														</div>
													</li>
													<li tabindex="0" data-locale="no"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Norsk</div>
													</li>
													<li tabindex="0" data-locale="pl"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Polski</div>
													</li>
													<li tabindex="0" data-locale="pt-BR"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Português
															(Brasil)</div>
													</li>
													<li tabindex="0" data-locale="ro"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Română</div>
													</li>
													<li tabindex="tabindex" data-locale="fi"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Suomi</div>
													</li>
													<li tabindex="0" data-locale="sv"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Svenska</div>
													</li>
													<li tabindex="0" data-locale="vi"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Tiếng Việt
														</div>
													</li>
													<li tabindex="0" data-locale="tr"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Türkçe</div>
													</li>
													<li tabindex="0" data-locale="el"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Ελληνικά</div>
													</li>
													<li tabindex="0" data-locale="bg"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">български
														</div>
													</li>
													<li tabindex="0" data-locale="ru"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Русский</div>
													</li>
													<li tabindex="0" data-locale="uk"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">Українська
														</div>
													</li>
													<li tabindex="0" data-locale="hi"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">हिंदी</div>
													</li>
													<li tabindex="0" data-locale="th"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">ไทย</div>
													</li>
													<li tabindex="0" data-locale="ko"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">한국어</div>
													</li>
													<li tabindex="0" data-locale="zh-Hans"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">中文</div>
													</li>
													<li tabindex="0" data-locale="zh-TW"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">中文(繁體)</div>
													</li>
													<li tabindex="0" data-locale="ja"
														class="discord-2022--dropdown-list-container">
														<div class="discord-2022--dropdown-language-item">日本語</div>
													</li>
												</ul>
											</nav>
										</div>
										<div class="discord-2022--desctop-soc">
											<div class="discord-2022--p-footer discord-2022--hide-landscape">Social
											</div>
											<div class="discord-2022--flex-horizontal discord-2022--top-soc-new"><a
													data-track="twitter" href="https://twitter.com/discord"
													target="_blank"
													class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b23e_91ce5945e0716b8f27aba591ed3ce824_x.svg" loading="lazy" alt="Twitter" class="discord-2022--image"/></a><a
														data-track="instagram" href="https://www.instagram.com/discord/"
														target="_blank"
														class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b240_456a1737b263ca0ec63b760ac332ed2a_instagram.svg" loading="lazy" alt="Instagram" class="discord-2022--image"/></a><a
															data-track="facebook"
															href="https://www.facebook.com/discord/" target="_blank"
															class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b23f_a77cbd313cca7494393b7a8ccc20fa16_facebook.svg" loading="lazy" alt="Facebook" class="discord-2022--image"/></a><a
																data-track="youtube"
																href="https://www.youtube.com/discord" target="_blank"
																class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b242_341e1594d600b55ff2302e0169b321ce_youtube.svg" loading="lazy" alt="Youtube" class="discord-2022--image"/></a><a
																	data-track="Tiktok"
																	href="https://www.tiktok.com/@discord"
																	target="_blank"
																	class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b241_1b8105cd54e8b765ed7c14cd03b13ffc_tiktok.svg" loading="lazy" alt="Tiktok" class="discord-2022--image"/></a>
											</div>
										</div>
									</div>
									<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b99a-99d9b91e">
										<div class="discord-2022--footer-h-link discord-2022--show-landscape">Menu</div>
										<div data-hover="false" data-delay="0"
											data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b99d"
											class="discord-2022--dropdown-footer w-dropdown">
											<div class="discord-2022--dropdown-toggle-footer w-dropdown-toggle">
												<div>Product</div>
												<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b244_0de9af0fe90fba53b80f020909344da6_Chevron%20Down.svg" loading="lazy" alt="" class="discord-2022--arrow-drop discord-2022--show-landscape"/></div>
												<nav class="discord-2022--dropdown-list-footer w-dropdown-list">
													<div
														class="discord-2022--padding-16px discord-2022--show-landscape">
													</div><a data-track="download" href="/download"
														class="discord-2022--link-footer discord-2022--top-new-link">Download</a><a
														data-track="nitro" href="/nitro"
														class="discord-2022--link-footer discord-2022--top-new-link">Nitro</a><a
														data-track="status" href="https://discordstatus.com/"
														class="discord-2022--link-footer discord-2022--top-new-link">Status</a><a
														data-track="app directory" href="/application-directory"
														class="discord-2022--link-footer discord-2022--top-new-link">App
														Directory</a>
												</nav>
											</div>
										</div>
										<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b9ac-99d9b91e">
											<div data-hover="false" data-delay="0"
												data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b9ad"
												class="discord-2022--dropdown-footer w-dropdown">
												<div class="discord-2022--dropdown-toggle-footer w-dropdown-toggle">
													<div>Company</div>
													<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b244_0de9af0fe90fba53b80f020909344da6_Chevron%20Down.svg" loading="lazy" alt="" class="discord-2022--arrow-drop discord-2022--show-landscape"/></div>
													<nav class="discord-2022--dropdown-list-footer w-dropdown-list">
														<div
															class="discord-2022--padding-16px discord-2022--show-landscape">
														</div><a data-track="about" href="/company"
															class="discord-2022--link-footer discord-2022--top-new-link">About</a><a
															data-track="jobs" href="/careers"
															class="discord-2022--link-footer discord-2022--top-new-link">Jobs</a><a
															data-track="branding" href="/branding"
															class="discord-2022--link-footer discord-2022--top-new-link">Brand</a><a
															data-track="newsroom" href="/newsroom"
															class="discord-2022--link-footer discord-2022--top-new-link">Newsroom</a>
													</nav>
												</div>
											</div>
											<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b9bc-99d9b91e">
												<div data-hover="false" data-delay="0"
													data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b9bd"
													class="discord-2022--dropdown-footer w-dropdown">
													<div class="discord-2022--dropdown-toggle-footer w-dropdown-toggle">
														<div>Resources</div>
														<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b244_0de9af0fe90fba53b80f020909344da6_Chevron%20Down.svg" loading="lazy" alt="" class="discord-2022--arrow-drop discord-2022--show-landscape"/></div>
														<nav class="discord-2022--dropdown-list-footer w-dropdown-list">
															<div
																class="discord-2022--padding-16px discord-2022--show-landscape">
															</div><a data-track="support"
																href="https://support.discord.com/hc"
																class="discord-2022--link-footer discord-2022--top-new-link">Support</a><a
																data-track="safety" href="/safety"
																class="discord-2022--link-footer discord-2022--top-new-link">Safety</a><a
																data-track="blog" href="/blog"
																class="discord-2022--link-footer discord-2022--top-new-link">Blog</a><a
																data-track="creators" href="/creators"
																class="discord-2022--link-footer discord-2022--top-new-link">Creators</a><a
																data-track="community" href="/community"
																class="discord-2022--link-footer discord-2022--top-new-link">Community</a><a
																data-track="Build" href="/developers"
																class="discord-2022--link-footer discord-2022--top-new-link">Developers</a><a
																data-track="Build" href="/ads/quests"
																class="discord-2022--link-footer discord-2022--top-new-link">Quests</a><a
																data-track="store"
																href="https://discordmerch.com/evergreenfooter"
																target="_blank"
																class="discord-2022--link-footer discord-2022--top-new-link">Official
																3rd Party Merch</a><a data-track="feedback"
																href="https://support.discord.com/hc/en-us/community/topics"
																class="discord-2022--link-footer discord-2022--top-new-link">Feedback</a>
														</nav>
													</div>
												</div>
												<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b9d6-99d9b91e">
													<div data-hover="false" data-delay="0"
														data-w-id="827f118f-7fce-63dd-0b4b-c14499d9b9d7"
														class="discord-2022--dropdown-footer discord-2022--line-none w-dropdown">
														<div
															class="discord-2022--dropdown-toggle-footer w-dropdown-toggle">
															<div>Policies</div>
															<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b244_0de9af0fe90fba53b80f020909344da6_Chevron%20Down.svg" loading="lazy" alt="" class="discord-2022--arrow-drop discord-2022--show-landscape"/></div>
															<nav
																class="discord-2022--dropdown-list-footer w-dropdown-list">
																<div
																	class="discord-2022--padding-16px discord-2022--show-landscape">
																</div><a data-track="terms" href="/terms"
																	class="discord-2022--link-footer discord-2022--top-new-link">Terms</a><a
																	data-track="privacy" href="/privacy"
																	class="discord-2022--link-footer discord-2022--top-new-link">Privacy</a><a
																	data-open-cookie-settings="true" href="#"
																	class="discord-2022--link-footer discord-2022--top-new-link">Cookie
																	Settings</a><a data-track="guidelines"
																	href="/guidelines"
																	class="discord-2022--link-footer discord-2022--top-new-link">Guidelines</a><a
																	data-track="acknowledgement"
																	href="/acknowledgements"
																	class="discord-2022--link-footer discord-2022--top-new-link">Acknowledgements</a><a
																	data-track="licenses" href="/licenses"
																	class="discord-2022--link-footer discord-2022--top-new-link">Licenses</a><a
																	data-track="moderation" href="/company-information"
																	class="discord-2022--link-footer discord-2022--top-new-link">Company
																	Information</a>
															</nav>
														</div>
													</div>
													<div id="w-node-_827f118f-7fce-63dd-0b4b-c14499d9b9ec-99d9b91e"
														class="discord-2022--show-soc">
														<div class="discord-2022--p-footer">Social</div>
														<div
															class="discord-2022--flex-horizontal discord-2022--top-soc-new">
															<a data-track="twitter" href="https://x.com/discord"
																target="_blank"
																class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b23e_91ce5945e0716b8f27aba591ed3ce824_x.svg" loading="lazy" alt="Twitter" class="discord-2022--image"/></a><a
																	data-track="instagram"
																	href="https://www.instagram.com/discord/"
																	target="_blank"
																	class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b240_456a1737b263ca0ec63b760ac332ed2a_instagram.svg" loading="lazy" alt="Instagram" class="discord-2022--image"/></a><a
																		data-track="facebook"
																		href="https://www.facebook.com/discord/"
																		target="_blank"
																		class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b23f_a77cbd313cca7494393b7a8ccc20fa16_facebook.svg" loading="lazy" alt="Facebook" class="discord-2022--image"/></a><a
																			data-track="youtube"
																			href="https://www.youtube.com/discord"
																			target="_blank"
																			class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e2112f22b8f351b242_341e1594d600b55ff2302e0169b321ce_youtube.svg" loading="lazy" alt="Youtube" class="discord-2022--image"/></a><a
																				data-track="Tiktok"
																				href="https://www.tiktok.com/@discord"
																				target="_blank"
																				class="discord-2022--link-s w-inline-block"><img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67a4d7e1112f22b8f351b241_1b8105cd54e8b765ed7c14cd03b13ffc_tiktok.svg" loading="lazy" alt="Tiktok" class="discord-2022--image"/></a>
														</div>
													</div>
												</div>
											</div>
											<div class="discord-2022--container_word">
												<img src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/67ac9dcae06c45cff3fac606_63bbb8d20f0336ebb2218972a83c5eec_Wordmark.svg" loading="lazy" aria-label="Discord" alt="Discord" class="discord-2022--word"/></div>
											</div>
										</div>
										<div data-w-id="f8c2604c-4846-bf09-53b2-b377380c1f91" class="menu-animation">
										</div>
										<script
											src="https://d3e54v103j8qbb.cloudfront.net/js/jquery-3.5.1.min.dc5e7f18c8.js?site=6238e97f6441e30a13a52345"
											type="text/javascript"
											integrity="sha256-9/aliU8dGd2tb6OSsuzixeV4y/faTqgFtohetphbbj0="
											crossorigin="anonymous"></script>
										<script
											src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/js/webflow.schunk.36b8fb49256177c8.js"
											type="text/javascript"></script>
										<script
											src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/js/webflow.schunk.8208d3e53b97e3c7.js"
											type="text/javascript"></script>
										<script
											src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/js/webflow.schunk.190b44dbfd51d2eb.js"
											type="text/javascript"></script>
										<script
											src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/js/webflow.schunk.a5328fc6e4f79712.js"
											type="text/javascript"></script>
										<script
											src="https://cdn.prod.website-files.com/6238e97f6441e30a13a52345/js/webflow.e1a0e204.2d30f71b25c27f08.js"
											type="text/javascript"></script>
										<script src="https://discord.com/webflow-scripts/bodyEnd.js" defer></script>
										<script type="text/javascript"
											src="https://discord.com/w/assets/latest/loginOrDownload.js" defer></script>
										<script src="https://discord.com/webflow-scripts/newBlogLocalization.js" defer>
										</script>


										<!-- section:dataLayer -->
										<script src="/assets/oneTrust/v7/scripttemplates/otSDKStub.js"
											type="text/javascript" charset="UTF-8"
											data-domain-script="04da1d72-0626-4fff-b3c6-150c719cc115"></script>
										<script
											src="/assets/oneTrust/v7/consent/04da1d72-0626-4fff-b3c6-150c719cc115/OtAutoBlock.js"
											type="text/javascript" charset="UTF-8"
											data-domain-script="04da1d72-0626-4fff-b3c6-150c719cc115"></script>
										<!-- build:inlineScriptNonceTag -->
										<script>
											<!-- endbuild 
											-->
											window.dataLayer
											=
											window.dataLayer
											||
											[];
											window.dataLayer.push({
											'allCookiesOK':
											false
											});
										</script>
										<!-- endsection -->
										<!-- section:gtm -->
										<!-- build:inlineScriptNonceTag -->
										<script>
											(function (w, d, s, l, i) {
      w[l] = w[l] || []; w[l].push({
        'gtm.start':
          new Date().getTime(), event: 'gtm.js'
      }); var f = d.getElementsByTagName(s)[0],
        j = d.createElement(s), dl = l != 'dataLayer' ? '&l=' + l : ''; j.async = true; j.src =
          'https://www.googletagmanager.com/gtm.js?id=' + i + dl + '&gtm_auth=GI0g9O-54_SitcgmxQKxlA&gtm_preview=env-2&gtm_cookies_win=x'; f.parentNode.insertBefore(j, f);
    })(window, document, 'script', 'dataLayer', 'GTM-N7BVC2W');
										</script>
										<!-- endsection -->
										<script
											src="/w/assets/4efad7abbb181e82f5667100cf0adb93a40de8f6/index-consolidated.js"
											defer></script>
										<script nonce="MjAxLDMsODYsMTE2LDM1LDIwMywzOSwxMjg=">
											(function(){function c(){var b=a.contentDocument||a.contentWindow.document;if(b){var d=b.createElement('script');d.nonce='MjAxLDMsODYsMTE2LDM1LDIwMywzOSwxMjg=';d.innerHTML="window.__CF$cv$params={r:'9aeacd0c3cb3c0fd',t:'MTc2NTg1MjY2OC4wMDAwMDA='};var a=document.createElement('script');a.nonce='MjAxLDMsODYsMTE2LDM1LDIwMywzOSwxMjg=';a.src='/cdn-cgi/challenge-platform/scripts/jsd/main.js';document.getElementsByTagName('head')[0].appendChild(a);";b.getElementsByTagName('head')[0].appendChild(d)}}if(document.body){var a=document.createElement('iframe');a.height=1;a.width=1;a.style.position='absolute';a.style.top=0;a.style.left=0;a.style.border='none';a.style.visibility='hidden';document.body.appendChild(a);if('loading'!==document.readyState)c();else if(window.addEventListener)document.addEventListener('DOMContentLoaded',c);else{var e=document.onreadystatechange||function(){};document.onreadystatechange=function(b){e(b);'loading'!==document.readyState&&(document.onreadystatechange=e,c())}}}})();
										</script>
</body>

</html>
"##.to_string()
    }
}
