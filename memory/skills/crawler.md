# Skillset: Web Scraper & Crawler Integration

## 1. Tool Overview & Architecture
The Web Crawler & Scraper tool is a high-performance content extraction micro-daemon running alongside the Styx Agent OS. It bridges the Model Context Protocol (MCP 2024-11-05) to modern web scraping engines (such as self-hosted Firecrawl, Crawl4AI, or generic crawler endpoints) with zero-configuration native fallback.

### Core Capabilities:
- **`scrape_page`**: Scrapes a single webpage URL and converts the content into clean, structured Markdown, extracting page title, description, and metadata.
- **`crawl_site`**: Initiates a multi-page crawl starting from a seed URL, traversing internal links within the domain up to specified depth and page limits.
- **Configurable Crawler Endpoints**: Uses the environment variable `CRAWLER_URL` (e.g., `http://192.168.1.2:3000` or `http://192.168.1.2:3000/v2/scrape`), with per-call overrides via the `crawler_url` argument.
- **Seamless Native Fallback**: If an external crawler instance is not reachable, the tool automatically executes an in-process headless scraper/crawler without breaking.

---

## 2. Tool Catalog & Best Practices

### `scrape_page` (Autonomous, Risk: LOW)
- **Parameters**:
  * `url` (string, required): Full HTTP/HTTPS webpage URL to scrape.
  * `crawler_url` (string, optional): Custom crawler endpoint override (e.g., `"http://192.168.1.2:3000/v2/scrape"`).
  * `only_main_content` (boolean, optional, default: true): Extract only the primary article/body content.
  * `wait_for` (number, optional): Milliseconds to wait before scraping (for dynamic client-side rendered pages).
  * `max_length` (number, optional, default: 12000): Maximum characters of markdown content.
- **When to Use**: Deep extraction of a single article, documentation page, or repository file where standard text extraction is insufficient.

### `crawl_site` (Autonomous, Risk: LOW)
- **Parameters**:
  * `url` (string, required): Seed HTTP/HTTPS URL to start crawling.
  * `crawler_url` (string, optional): Custom crawler endpoint override.
  * `limit` (number, optional, default: 5, max: 20): Maximum number of pages to crawl.
  * `max_depth` (number, optional, default: 2): Maximum depth level from seed URL.
  * `same_domain_only` (boolean, optional, default: true): Confines crawling strictly to the seed domain.
- **When to Use**: Crawling documentation sites, blogs, product sites, or exploring an entire domain's linked resources in a single call.

---

## 3. Best Practices for the Agent
1. **Target Specific Sub-Paths**: When crawling documentation, start at the root docs URL (e.g., `https://docs.example.com/guide/`) with `limit: 5` or `10` to avoid pulling unnecessary landing page noise.
2. **Combine with Search**: Use `web_search` to discover the most relevant domain or entry point, then use `crawl_site` or `scrape_page` to ingest the full contents.
3. **Respect Content Limits**: Keep `limit` reasonable (e.g. 5–10 pages) for focused synthesis.
