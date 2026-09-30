# Skillset: Web Search & Internet Retrieval Integration

## 1. Tool Overview & Architecture
The Web Search tool is a privacy-first, configurable search and web ingestion micro-daemon running alongside the Styx Agent OS. It bridges standard Model Context Protocol (MCP 2024-11-05) to privacy-respecting search engines (SearXNG instances, DuckDuckGo, or custom aggregators).

### Core Capabilities:
- **`web_search`**: Queries search engines for relevant real-time web results, documentation, and articles.
- **`read_web_page`**: Fetches and extracts clean, readable text/markdown from specific webpage URLs for in-depth inspection.
- **Configurable Backends**: Supports self-hosted SearXNG engines (`SEARXNG_URL`) as well as zero-configuration public fallbacks.

---

## 2. Tool Catalog & Best Practices

### `web_search` (Autonomous, Risk: LOW)
- **Parameters**:
  * `query` (string, required): Clear search keywords or targeted query string.
  * `limit` (number, optional, default: 5, max: 20): Number of top search results to return.
  * `categories` (string, optional): Search category filter (`"general"`, `"news"`, `"science"`, `"it"`).
  * `time_range` (string, optional): Temporal filter (`"day"`, `"week"`, `"month"`, `"year"`).
- **Purpose**: Retrieve up-to-date information, technical documentation, news, or fact-check knowledge.
- **Best Practices**:
  * Formulate concise, keyword-rich search queries rather than conversational sentences.
  * Use specific domain keywords (e.g. library names, version numbers, error codes).
  * When researching an unfamiliar topic or breaking news, check top 3-5 results.

### `read_web_page` (Autonomous, Risk: LOW)
- **Parameters**:
  * `url` (string, required): Full HTTP/HTTPS webpage URL.
  * `max_length` (number, optional, default: 4000): Maximum characters of extracted text to return.
- **Purpose**: Read the full content of an article, documentation page, or repository found via `web_search`.
- **Best Practices**:
  * Call `read_web_page` when a search snippet indicates high relevance but does not contain full instructions or code examples.

---

## 3. Styx Agent Integration Directives

### 1. Verification & Accuracy
When responding to user requests regarding live facts, software versions, current weather, financial data, or external references:
- **Always consult `web_search`** before guessing or hallucinating stale facts.
- **Cite Sources**: Provide clean, clickable HTTP/HTTPS links to the referenced sources in your final response.

### 2. Autonomous Knowledge Retention
- If search results reveal durable facts about tools, APIs, or user preferences, use `write_memory` to store valuable findings in `memory/reference/` or `memory/skills/` for future turns.
