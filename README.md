# kraken

Kraken is a tiny cross‑platform GUI utility to save public ChatGPT share threads as Markdown.

- Input: a public share URL (`https://chatgpt.com/share/<id>`)
- Output: a `.md` file with the conversation (message order preserved)
- Formats: Markdown supported; PDF is disabled for now
- Platforms: macOS, Windows, Linux (x64/arm64)

## Download

1) Open the repository’s Releases page (GitHub → “Releases” tab).
2) Download the asset for your platform:
   - macOS (Apple Silicon): `kraken-Darwin-arm64`
   - macOS (Intel): `kraken-Darwin-x86_64`
   - Windows: `kraken-Windows-x86_64.exe` or `kraken-Windows-arm64.exe`
   - Linux: `kraken-Linux-x86_64` or `kraken-Linux-arm64`

Notes:
- macOS: you may need to allow the app in System Settings → Privacy & Security.
- Windows: SmartScreen may ask for confirmation (“Run anyway”).
- Linux: you may need to make the file executable: `chmod +x ./kraken-Linux-*`.

## Use

1) Run Kraken.
2) Paste a public ChatGPT share link (example: `https://chatgpt.com/share/<id>`).
3) Pick the file format (Markdown). PDF is “coming soon”.
4) Click “Download” and choose where to save.

The saved Markdown contains the dialog (user/assistant turns) and a source link at the top.

## Troubleshooting

- Ensure the link is publicly accessible (shared view).
- If a newly extended conversation doesn’t appear immediately, try again in a moment; the app requests a fresh copy, but some relays can be briefly cached.
