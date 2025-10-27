use anyhow::Result;
use iced::theme::{self, Theme};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings};
use regex::Regex;
use rfd::FileDialog;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Format {
    Markdown,
    PdfDisabled,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Markdown => write!(f, "Markdown (.md)"),
            Format::PdfDisabled => write!(f, "PDF (.pdf) — скоро"),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    UrlChanged(String),
    FormatChanged(Format),
    DownloadClicked,
    Fetched(std::result::Result<String, String>),
}

struct App {
    url: String,
    format: Format,
    status: String,
    preview: String,
    formats: Vec<Format>,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                url: String::new(),
                format: Format::Markdown,
                status: String::new(),
                preview: String::new(),
                formats: vec![Format::Markdown, Format::PdfDisabled],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "ShareSaver (Rust/Iced)".into()
    }

    fn theme(&self) -> Theme {
        theme::Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::UrlChanged(s) => {
                self.url = s;
            }
            Message::FormatChanged(fmt) => {
                self.format = fmt;
            }
            Message::DownloadClicked => {
                self.status.clear();
                self.preview.clear();
                let url = self.url.clone();
                return Command::perform(async move {
                    fetch_and_convert(url).await.map_err(|e| e.to_string())
                }, Message::Fetched);
            }
            Message::Fetched(res) => match res {
                Ok(md) => {
                    self.preview = md.clone();
                    self.status = "Готово. Выберите путь для сохранения.".into();

                    if let Format::Markdown = self.format {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Markdown", &["md"]) 
                            .set_file_name("chatgpt_conversation.md")
                            .save_file()
                        {
                            let _ = std::fs::write(path, md);
                            self.status = "Сохранено".into();
                        } else {
                            self.status = "Сохранение отменено".into();
                        }
                    } else {
                        self.status = "PDF пока недоступен".into();
                    }
                }
                Err(e) => {
                    self.status = format!("Ошибка: {}", e);
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let url_input = text_input("https://chatgpt.com/share/...", &self.url)
            .on_input(Message::UrlChanged)
            .width(Length::Fill);

        let fmt_combo = pick_list(self.formats.clone(), Some(self.format.clone()), Message::FormatChanged);

        let download_btn = button(text("Скачать")).on_press(Message::DownloadClicked);

        let top = row![
            text("Публичная ссылка:").width(Length::Shrink),
            url_input,
        ]
        .spacing(8);

        let second = row![
            text("Формат:").width(Length::Shrink),
            fmt_combo,
            download_btn,
            text(&self.status)
        ]
        .spacing(12)
        .align_items(iced::Alignment::Center);

        let preview = scrollable(container(text(&self.preview)).padding(8)).height(Length::Fill);

        container(column![top, second, preview].spacing(12).padding(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }
}

impl App {}

#[tokio::main]
async fn main() -> iced::Result {
    App::run(Settings::default())
}

async fn fetch_and_convert(share_url: String) -> Result<String> {
    if share_url.trim().is_empty() {
        anyhow::bail!("Укажите ссылку");
    }

    let normalized = share_url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string();

    if let Some(md) = try_fetch_backend_json(&normalized).await? {
        return Ok(md);
    }

    // Fallback: r.jina.ai -> Markdown страницы
    let cache_buster = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let sep = if normalized.contains('?') { '&' } else { '?' };
    let url = format!(
        "https://r.jina.ai/http://{}{}{}_ts={}",
        normalized, sep, if sep == '&' { "" } else { "" }, cache_buster
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()?;
    let text = client
        .get(&url)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let title = Regex::new(r"^Title:\s*(.*)$")
        .unwrap()
        .captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "ChatGPT Conversation".to_string());

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", title));
    out.push_str(&format!("**Источник**: {}\n\n", share_url));

    if let Some(m) = Regex::new(r"(?m)^##### You said:")
        .unwrap()
        .find(&text)
    {
        out.push_str(&text[m.start()..]);
    } else {
        out.push_str(&text);
    }

    Ok(out)
}

async fn try_fetch_backend_json(normalized_share: &str) -> Result<Option<String>> {
    let id = extract_share_id(normalized_share).unwrap_or_else(|| normalized_share.to_string());
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let candidates = vec![
        format!("https://r.jina.ai/http://chatgpt.com/backend-api/share/{}?_ts={}", id, ts),
        format!("https://r.jina.ai/https://chatgpt.com/backend-api/share/{}?_ts={}", id, ts),
    ];

    let client = reqwest::Client::builder().user_agent("Mozilla/5.0").build()?;

    for u in candidates {
        let resp = client
            .get(&u)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .send()
            .await?;
        if !resp.status().is_success() {
            continue;
        }
        let body = resp.text().await?;
        if let Some(md) = parse_backend_to_markdown(&body, normalized_share) {
            return Ok(Some(md));
        }
    }
    Ok(None)
}

fn parse_backend_to_markdown(body: &str, normalized_share: &str) -> Option<String> {
    let title = Regex::new(r#""title"\s*:\s*"(.*?)""#)
        .ok()?
        .captures(body)
        .and_then(|c| c.get(1))
        .map(|m| json_unescape(m.as_str()));

    let pattern = Regex::new(r#""role"\s*:\s*"(user|assistant)"[\s\S]*?"parts"\s*:\s*\[(.*?)\]"#).ok()?;
    let mut msgs: Vec<(String, String)> = Vec::new();
    for cap in pattern.captures_iter(body) {
        let role = cap.get(1)?.as_str().to_string();
        let parts_raw = format!("[{}]", cap.get(2)?.as_str());
        let mut text = String::new();
        if let Ok(vec) = serde_json::from_str::<Vec<String>>(&parts_raw) {
            text = vec.join("\n\n");
        } else {
            text = json_unescape(cap.get(2)?.as_str());
        }
        msgs.push((role, text));
    }

    if msgs.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", title.unwrap_or_else(|| String::from("ChatGPT Conversation"))));
    out.push_str(&format!("**Источник**: https://{}\n\n", normalized_share));

    for (role, text_) in msgs {
        let who = if role == "assistant" { "Ассистент" } else { "Пользователь" };
        out.push_str(&format!("> {}: {}\n\n", who, text_.replace("\r\n", "\n")));
    }
    Some(out)
}

fn extract_share_id(normalized: &str) -> Option<String> {
    let re = Regex::new(r"/share/([a-f0-9\-]+)").ok()?;
    re.captures(normalized)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn json_unescape(s: &str) -> String {
    serde_json::from_str::<String>(&format!("\"{}\"", s)).unwrap_or_else(|_| s.to_string())
}
