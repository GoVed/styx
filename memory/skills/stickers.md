# Skillset: Web & GIPHY Sticker Search Integration

## 1. Tool Overview & Architecture
The Sticker Search tool empowers the Styx agent to dynamically discover, download, and format stickers from GIPHY and online transparent image repositories. It automatically formats any discovered sticker into a standard **512x512 pixel WebP with WhatsApp EXIF pack metadata**, making it ready for instant delivery across WhatsApp, web chat, or other messaging channels.

### Core Capabilities:
- **`search_stickers`**: Queries GIPHY and online sticker engines for transparent stickers matching emotions, gestures, memes, or topics (e.g. "shrug", "confetti party", "thumbs up", "laughing").
- **`download_sticker_as_webp`**: Ingests an image URL or sticker ID, resizes and crops it to 512x512 pixels with alpha transparency, embeds official WhatsApp EXIF pack metadata, and caches it locally as a valid WebP sticker.

---

## 2. Tool Catalog & Best Practices

### `search_stickers` (Autonomous, Risk: LOW)
- **Parameters**:
  * `query` (string, required): Emotional cue, action, or character (e.g. `"shrug cat"`, `"celebration"`, `"thumbs up"`, `"mind blown"`).
  * `limit` (number, optional, default: 5, max: 20): Number of sticker choices to return.
- **Purpose**: Discover expressive transparent stickers matching conversational context.
- **Best Practices**:
  * Include keywords like `"sticker"` or emotional intent if searching for specific aesthetics.
  * Check the returned `title` and `preview_url` to select the most fitting sticker for the conversation.

### `download_sticker_as_webp` (Autonomous, Risk: LOW)
- **Parameters**:
  * `url` (string, required): Online image or sticker URL (WebP, GIF, or PNG).
  * `pack_name` (string, optional, default: `"Styx Assistant"`): Name of the sticker pack.
  * `author` (string, optional, default: `"Styx"`): Publisher attribution for the sticker.
- **Purpose**: Prepare an online sticker for messaging delivery.
- **Returns**:
  * `local_path`: Cached file path on disk.
  * `base64`: Base64-encoded WebP string.
  * `byte_length`: Size in bytes (compliant with WhatsApp size limits).

---

## 3. Styx Agent Integration Directives

### 1. Dynamic Sticker Selection Flow
When a user asks to send a sticker or when reacting to a message with a sticker:
1. First, search for relevant stickers using `search_stickers(query: "...")`.
2. Select the most appropriate result.
3. Call `download_sticker_as_webp(url: "...")` to obtain the formatted WebP buffer.
4. When sending via WhatsApp or messaging tools, call `send_sticker(to, sticker_id: url)` passing the `url` returned by `download_sticker_as_webp` (or image URL directly). Never invent custom sticker IDs.
