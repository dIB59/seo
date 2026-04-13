/** Preset configurations for common CSS-selector extractors. */
export interface Preset {
  name: string;
  tag: string;
  selector: string;
  attribute: string | null;
  multiple: boolean;
  description: string;
  htmlPreview: string;
  highlightValue: string;
}

export const PRESETS: Preset[] = [
  {
    name: "Hreflang Tags",
    tag: "hreflang",
    selector: "link[rel='alternate'][hreflang]",
    attribute: "hreflang",
    multiple: true,
    description: "Collects all language/region codes declared on the page (e.g. en-US, fr-FR).",
    htmlPreview: `<link rel="alternate" hreflang="en-US" href="..." />`,
    highlightValue: "en-US",
  },
  {
    name: "OG Image",
    tag: "og_image",
    selector: "meta[property='og:image']",
    attribute: "content",
    multiple: false,
    description: "The Open Graph image URL used when sharing on social media.",
    htmlPreview: `<meta property="og:image" content="https://example.com/img.jpg" />`,
    highlightValue: "https://example.com/img.jpg",
  },
  {
    name: "OG Title",
    tag: "og_title",
    selector: "meta[property='og:title']",
    attribute: "content",
    multiple: false,
    description: "The title shown when the page is shared on social media.",
    htmlPreview: `<meta property="og:title" content="My Page Title" />`,
    highlightValue: "My Page Title",
  },
  {
    name: "Canonical URL",
    tag: "canonical",
    selector: "link[rel='canonical']",
    attribute: "href",
    multiple: false,
    description: "The preferred URL for this page, used to avoid duplicate content issues.",
    htmlPreview: `<link rel="canonical" href="https://example.com/page" />`,
    highlightValue: "https://example.com/page",
  },
  {
    name: "JSON-LD Schema",
    tag: "schema_types",
    selector: "script[type='application/ld+json']",
    attribute: null,
    multiple: true,
    description: "Extracts structured data blocks for schema.org markup.",
    htmlPreview: `<script type="application/ld+json">{"@type": "Article"}</script>`,
    highlightValue: `{"@type": "Article"}`,
  },
  {
    name: "Robots Meta",
    tag: "robots_meta",
    selector: "meta[name='robots']",
    attribute: "content",
    multiple: false,
    description: "The robots directive (e.g. noindex, nofollow) on this page.",
    htmlPreview: `<meta name="robots" content="noindex, follow" />`,
    highlightValue: "noindex, follow",
  },
  {
    name: "Author",
    tag: "author",
    selector: "meta[name='author']",
    attribute: "content",
    multiple: false,
    description: "The author name as declared in the page head.",
    htmlPreview: `<meta name="author" content="Jane Smith" />`,
    highlightValue: "Jane Smith",
  },
  {
    name: "H1 Heading",
    tag: "h1_text",
    selector: "h1",
    attribute: null,
    multiple: false,
    description: "The text content of the main H1 heading.",
    htmlPreview: `<h1>Welcome to Our Store</h1>`,
    highlightValue: "Welcome to Our Store",
  },
];
