use std::{
    env, fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

use chrono::Local;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily, RichText, TextEdit};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CONFIG_FILE: &str = "blog_writer_config.json";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Blog Writer",
        options,
        Box::new(|cc| {
            setup_korean_fonts(&cc.egui_ctx);
            Ok(Box::new(BlogWriterApp::new()))
        }),
    )
}

fn setup_korean_fonts(ctx: &egui::Context) {
    let font_paths = [
        "/System/Library/Fonts/AppleSDGothicNeo.ttc",
        "/System/Library/Fonts/Supplemental/AppleGothic.ttf",
        "/Library/Fonts/AppleGothic.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "C:\\Windows\\Fonts\\malgun.ttf",
    ];

    let Some(font_bytes) = font_paths.iter().find_map(|path| fs::read(path).ok()) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    let font_name = "korean_system".to_owned();
    fonts
        .font_data
        .insert(font_name.clone(), FontData::from_owned(font_bytes).into());

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, font_name.clone());
    }

    ctx.set_fonts(fonts);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Write,
    Accounts,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Visibility {
    Private,
    Public,
}

impl Visibility {
    fn as_naver_open_bool(self) -> &'static str {
        match self {
            Self::Private => "false",
            Self::Public => "true",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Private => "비공개",
            Self::Public => "발행",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Draft {
    title: String,
    body: String,
    tags: String,
    visibility: Visibility,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            title: "새 블로그 포스트 제목".to_owned(),
            body: "<h2>소제목</h2>\n<p>본문을 HTML로 작성하세요.</p>".to_owned(),
            tags: "자동화,블로그".to_owned(),
            visibility: Visibility::Private,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct NaverConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    access_token: String,
    refresh_token: String,
    category_no: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TistoryConfig {
    editor_url: String,
}

impl Default for TistoryConfig {
    fn default() -> Self {
        Self {
            editor_url: "https://www.tistory.com/manage/newpost/".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiConfig {
    api_key: String,
    model: String,
    topic: String,
    audience: String,
    keywords: String,
    tone: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gpt-5.2".to_owned(),
            topic: "블로그 자동화 도구 소개".to_owned(),
            audience: "개인 블로그 운영자".to_owned(),
            keywords: "네이버 블로그,티스토리,자동화,AI 글쓰기".to_owned(),
            tone: "실용적이고 자연스러운 한국어".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SavedState {
    draft: Draft,
    naver: NaverConfig,
    tistory: TistoryConfig,
    ai: AiConfig,
}

#[derive(Debug)]
enum WorkerResult {
    NaverPublished(Result<String, String>),
    NaverCategories(Result<String, String>),
    NaverToken(Result<TokenResponse, String>),
    NaverRefresh(Result<TokenResponse, String>),
    AiGenerated(Result<Draft, String>),
}

#[derive(Debug, Clone, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

struct BlogWriterApp {
    active_tab: Tab,
    draft: Draft,
    naver: NaverConfig,
    tistory: TistoryConfig,
    ai: AiConfig,
    auth_code: String,
    auth_state: String,
    status: String,
    log: Vec<String>,
    worker: Option<Receiver<WorkerResult>>,
}

impl BlogWriterApp {
    fn new() -> Self {
        let saved = load_state().unwrap_or_default();
        Self {
            active_tab: Tab::Write,
            draft: saved.draft,
            naver: saved.naver,
            tistory: saved.tistory,
            ai: saved.ai,
            auth_code: String::new(),
            auth_state: String::new(),
            status: "준비됨".to_owned(),
            log: vec![
                "티스토리는 2024년 2월 Open API 종료로 직접 발행 대신 보조 발행만 지원합니다."
                    .to_owned(),
            ],
            worker: None,
        }
    }

    fn poll_worker(&mut self) {
        let Some(rx) = self.worker.take() else {
            return;
        };

        let mut keep_worker = true;
        loop {
            match rx.try_recv() {
                Ok(result) => {
                    keep_worker = false;
                    match result {
                        WorkerResult::NaverPublished(result) => match result {
                            Ok(message) => self.set_status("네이버 발행 완료", message),
                            Err(err) => self.set_status("네이버 발행 실패", err),
                        },
                        WorkerResult::NaverCategories(result) => match result {
                            Ok(message) => self.set_status("네이버 카테고리 조회 완료", message),
                            Err(err) => self.set_status("네이버 카테고리 조회 실패", err),
                        },
                        WorkerResult::NaverToken(result) => match result {
                            Ok(token) => {
                                self.apply_token(token);
                                self.set_status(
                                    "네이버 토큰 발급 완료",
                                    "토큰을 설정에 반영했습니다.",
                                );
                            }
                            Err(err) => self.set_status("네이버 토큰 발급 실패", err),
                        },
                        WorkerResult::NaverRefresh(result) => match result {
                            Ok(token) => {
                                self.apply_token(token);
                                self.set_status(
                                    "네이버 토큰 갱신 완료",
                                    "새 토큰을 설정에 반영했습니다.",
                                );
                            }
                            Err(err) => self.set_status("네이버 토큰 갱신 실패", err),
                        },
                        WorkerResult::AiGenerated(result) => match result {
                            Ok(draft) => {
                                self.draft = draft;
                                self.set_status(
                                    "AI 초안 생성 완료",
                                    "제목, 본문, 태그를 반영했습니다.",
                                );
                            }
                            Err(err) => self.set_status("AI 초안 생성 실패", err),
                        },
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_worker = false;
                    break;
                }
            }
        }

        if keep_worker {
            self.worker = Some(rx);
        }
    }

    fn apply_token(&mut self, token: TokenResponse) {
        if let Some(access_token) = token.access_token {
            self.naver.access_token = access_token;
        }
        if let Some(refresh_token) = token.refresh_token {
            self.naver.refresh_token = refresh_token;
        }
        if let Some(token_type) = token.token_type {
            self.log.push(format!("토큰 타입: {token_type}"));
        }
        if let Some(expires_in) = token.expires_in {
            self.log.push(format!("만료까지: {expires_in}초"));
        }
        if let Some(error) = token.error {
            self.log.push(format!("토큰 응답 오류: {error}"));
        }
        if let Some(error_description) = token.error_description {
            self.log
                .push(format!("토큰 응답 설명: {error_description}"));
        }
    }

    fn set_status(&mut self, status: impl Into<String>, detail: impl Into<String>) {
        let status = status.into();
        let detail = detail.into();
        self.status = status.clone();
        self.log.push(format!(
            "[{}] {status}: {detail}",
            Local::now().format("%H:%M:%S")
        ));
    }

    fn start_worker(&mut self, result_kind: impl FnOnce() -> WorkerResult + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(result_kind());
        });
        self.worker = Some(rx);
        self.status = "요청 처리 중".to_owned();
    }

    fn publish_to_naver(&mut self) {
        let draft = self.draft.clone();
        let naver = self.naver.clone();
        if draft.title.trim().is_empty() {
            self.set_status("검증 실패", "제목을 입력하세요.");
            return;
        }
        if draft.body.trim().is_empty() {
            self.set_status("검증 실패", "본문을 입력하세요.");
            return;
        }
        if naver.access_token.trim().is_empty() {
            self.set_status("검증 실패", "네이버 Access Token을 입력하거나 발급하세요.");
            return;
        }

        self.start_worker(move || WorkerResult::NaverPublished(publish_naver_post(&naver, &draft)));
    }

    fn fetch_naver_categories(&mut self) {
        let naver = self.naver.clone();
        if naver.access_token.trim().is_empty() {
            self.set_status("검증 실패", "네이버 Access Token이 필요합니다.");
            return;
        }
        self.start_worker(move || WorkerResult::NaverCategories(fetch_naver_categories(&naver)));
    }

    fn request_naver_token(&mut self) {
        let naver = self.naver.clone();
        let code = self.auth_code.trim().to_owned();
        let state = self.auth_state.trim().to_owned();
        if code.is_empty() {
            self.set_status("검증 실패", "인증 code를 입력하세요.");
            return;
        }
        self.start_worker(move || {
            WorkerResult::NaverToken(exchange_naver_token(&naver, &code, &state))
        });
    }

    fn refresh_naver_token(&mut self) {
        let naver = self.naver.clone();
        if naver.refresh_token.trim().is_empty() {
            self.set_status("검증 실패", "Refresh Token이 필요합니다.");
            return;
        }
        self.start_worker(move || WorkerResult::NaverRefresh(refresh_naver_token(&naver)));
    }

    fn generate_ai_draft(&mut self) {
        let ai = self.ai.clone();
        if ai.api_key.trim().is_empty() && env::var("OPENAI_API_KEY").is_err() {
            self.set_status(
                "검증 실패",
                "OpenAI API Key를 입력하거나 OPENAI_API_KEY 환경 변수를 설정하세요.",
            );
            return;
        }
        if ai.topic.trim().is_empty() {
            self.set_status("검증 실패", "AI가 쓸 주제를 입력하세요.");
            return;
        }

        self.start_worker(move || WorkerResult::AiGenerated(generate_ai_draft(&ai)));
    }

    fn copy_to_clipboard(&mut self, ctx: &egui::Context, label: &str, value: String) {
        ctx.copy_text(value);
        self.set_status("클립보드 복사", label);
    }

    fn save(&mut self) {
        let state = SavedState {
            draft: self.draft.clone(),
            naver: self.naver.clone(),
            tistory: self.tistory.clone(),
            ai: self.ai.clone(),
        };
        match save_state(&state) {
            Ok(path) => self.set_status("저장 완료", path.display().to_string()),
            Err(err) => self.set_status("저장 실패", err),
        }
    }

    fn export_html(&mut self) {
        match export_html(&self.draft) {
            Ok(path) => self.set_status("HTML 내보내기 완료", path.display().to_string()),
            Err(err) => self.set_status("HTML 내보내기 실패", err),
        }
    }

    fn open_tistory_editor(&mut self) {
        match webbrowser::open(self.tistory.editor_url.trim()) {
            Ok(_) => self.set_status("티스토리 편집기 열기", self.tistory.editor_url.clone()),
            Err(err) => self.set_status("티스토리 편집기 열기 실패", err.to_string()),
        }
    }

    fn open_naver_auth(&mut self, ctx: &egui::Context) {
        if self.naver.client_id.trim().is_empty() || self.naver.redirect_uri.trim().is_empty() {
            self.set_status("검증 실패", "Client ID와 Redirect URI가 필요합니다.");
            return;
        }
        if self.auth_state.trim().is_empty() {
            self.auth_state = format!("bw{}", Local::now().timestamp());
        }
        let url = naver_authorize_url(&self.naver, &self.auth_state);
        ctx.copy_text(url.clone());
        match webbrowser::open(&url) {
            Ok(_) => self.set_status("네이버 로그인 URL 열기", "URL도 클립보드에 복사했습니다."),
            Err(err) => self.set_status(
                "네이버 로그인 URL 복사",
                format!("브라우저 열기 실패: {err}. URL은 클립보드에 복사했습니다."),
            ),
        }
    }
}

impl eframe::App for BlogWriterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_worker();

        ui.horizontal(|ui| {
            tab_button(ui, &mut self.active_tab, Tab::Write, "작성");
            tab_button(ui, &mut self.active_tab, Tab::Accounts, "계정");
            tab_button(ui, &mut self.active_tab, Tab::Log, "로그");
            ui.separator();
            ui.label(RichText::new(&self.status).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("설정 저장").clicked() {
                    self.save();
                }
            });
        });
        ui.separator();

        match self.active_tab {
            Tab::Write => self.write_tab(ui, &ctx),
            Tab::Accounts => self.accounts_tab(ui, &ctx),
            Tab::Log => self.log_tab(ui),
        }
    }
}

impl BlogWriterApp {
    fn write_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.columns(2, |columns| {
            let left = &mut columns[0];
            left.heading("포스트 작성");
            left.add_space(8.0);
            left.label("제목");
            left.text_edit_singleline(&mut self.draft.title);
            left.add_space(8.0);
            left.label("본문 HTML");
            left.add(
                TextEdit::multiline(&mut self.draft.body)
                    .desired_rows(24)
                    .lock_focus(true)
                    .code_editor(),
            );
            left.add_space(8.0);
            left.label("태그");
            left.text_edit_singleline(&mut self.draft.tags);
            left.add_space(8.0);
            left.horizontal(|ui| {
                ui.label("상태");
                ui.selectable_value(&mut self.draft.visibility, Visibility::Private, "비공개");
                ui.selectable_value(&mut self.draft.visibility, Visibility::Public, "발행");
            });

            let right = &mut columns[1];
            right.heading("AI 초안");
            right.add_space(8.0);
            right.group(|ui| {
                ui.label("주제");
                ui.text_edit_singleline(&mut self.ai.topic);
                ui.label("대상 독자");
                ui.text_edit_singleline(&mut self.ai.audience);
                ui.label("키워드");
                ui.text_edit_singleline(&mut self.ai.keywords);
                ui.label("톤");
                ui.text_edit_singleline(&mut self.ai.tone);
                if ui
                    .add_enabled(self.worker.is_none(), egui::Button::new("AI 글 생성"))
                    .clicked()
                {
                    self.generate_ai_draft();
                }
            });

            right.add_space(12.0);
            right.heading("발행");
            right.add_space(8.0);
            right.group(|ui| {
                ui.heading("네이버 블로그");
                ui.label("공식 블로그 글쓰기 API로 발행합니다.");
                ui.horizontal(|ui| {
                    ui.label("카테고리 번호");
                    ui.text_edit_singleline(&mut self.naver.category_no);
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.worker.is_none(), egui::Button::new("네이버 발행"))
                        .clicked()
                    {
                        self.publish_to_naver();
                    }
                    if ui
                        .add_enabled(self.worker.is_none(), egui::Button::new("카테고리 조회"))
                        .clicked()
                    {
                        self.fetch_naver_categories();
                    }
                });
            });

            right.add_space(12.0);
            right.group(|ui| {
                ui.heading("티스토리");
                ui.label("Open API 종료 때문에 직접 발행 대신 편집기 보조 기능을 제공합니다.");
                ui.horizontal(|ui| {
                    if ui.button("편집기 열기").clicked() {
                        self.open_tistory_editor();
                    }
                    if ui.button("제목 복사").clicked() {
                        self.copy_to_clipboard(ctx, "제목", self.draft.title.clone());
                    }
                    if ui.button("본문 복사").clicked() {
                        self.copy_to_clipboard(ctx, "본문 HTML", self.draft.body.clone());
                    }
                    if ui.button("태그 복사").clicked() {
                        self.copy_to_clipboard(ctx, "태그", self.draft.tags.clone());
                    }
                });
                if ui.button("HTML 파일 내보내기").clicked() {
                    self.export_html();
                }
            });

            right.add_space(12.0);
            right.group(|ui| {
                ui.heading("미리보기용 정보");
                ui.label(format!("제목 길이: {}자", self.draft.title.chars().count()));
                ui.label(format!("본문 길이: {}자", self.draft.body.chars().count()));
                ui.label(format!("태그: {}", normalize_tags(&self.draft.tags)));
                ui.label(format!("공개 상태: {}", self.draft.visibility.label()));
            });
        });
    }

    fn accounts_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.columns(2, |columns| {
            let left = &mut columns[0];
            left.heading("네이버 OAuth");
            left.add_space(8.0);
            left.label("Client ID");
            left.text_edit_singleline(&mut self.naver.client_id);
            left.label("Client Secret");
            left.add(TextEdit::singleline(&mut self.naver.client_secret).password(true));
            left.label("Redirect URI");
            left.text_edit_singleline(&mut self.naver.redirect_uri);
            left.label("State");
            left.text_edit_singleline(&mut self.auth_state);
            left.horizontal(|ui| {
                if ui.button("로그인 URL 열기").clicked() {
                    self.open_naver_auth(ctx);
                }
                if ui.button("로그인 URL 복사").clicked() {
                    if self.auth_state.trim().is_empty() {
                        self.auth_state = format!("bw{}", Local::now().timestamp());
                    }
                    let url = naver_authorize_url(&self.naver, &self.auth_state);
                    self.copy_to_clipboard(ctx, "네이버 로그인 URL", url);
                }
            });
            left.add_space(8.0);
            left.label("Redirect 후 URL에서 받은 code");
            left.text_edit_singleline(&mut self.auth_code);
            left.horizontal(|ui| {
                if ui
                    .add_enabled(self.worker.is_none(), egui::Button::new("토큰 발급"))
                    .clicked()
                {
                    self.request_naver_token();
                }
                if ui
                    .add_enabled(self.worker.is_none(), egui::Button::new("토큰 갱신"))
                    .clicked()
                {
                    self.refresh_naver_token();
                }
            });
            left.add_space(8.0);
            left.label("Access Token");
            left.add(TextEdit::multiline(&mut self.naver.access_token).desired_rows(3));
            left.label("Refresh Token");
            left.add(TextEdit::multiline(&mut self.naver.refresh_token).desired_rows(3));

            let right = &mut columns[1];
            right.heading("AI 글쓰기");
            right.add_space(8.0);
            right.label("OpenAI API Key");
            right.add(TextEdit::singleline(&mut self.ai.api_key).password(true));
            right.label("Model");
            right.text_edit_singleline(&mut self.ai.model);
            right.label("API Key를 비워두면 OPENAI_API_KEY 환경 변수를 사용합니다.");
            right.separator();

            right.heading("티스토리 보조 발행");
            right.add_space(8.0);
            right.label("편집기 URL");
            right.text_edit_singleline(&mut self.tistory.editor_url);
            right.add_space(8.0);
            right.label(
                "기본값이 열리지 않으면 티스토리 관리자에서 새 글쓰기 화면 주소를 복사해 넣으세요.",
            );
            right.separator();
            right.heading("운영 방식");
            right.label("1. 네이버는 Access Token으로 직접 발행합니다.");
            right.label("2. 티스토리는 편집기를 열고 제목/본문/태그를 복사해 붙여넣습니다.");
            right.label("3. 같은 원고를 HTML 파일로 남겨 재발행과 백업에 사용합니다.");
        });
    }

    fn log_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("로그");
        ui.add_space(8.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in self.log.iter().rev() {
                ui.label(line);
            }
        });
    }
}

fn tab_button(ui: &mut egui::Ui, active: &mut Tab, tab: Tab, label: &str) {
    if ui.selectable_label(*active == tab, label).clicked() {
        *active = tab;
    }
}

fn naver_authorize_url(naver: &NaverConfig, state: &str) -> String {
    format!(
        "https://nid.naver.com/oauth2.0/authorize?response_type=code&client_id={}&redirect_uri={}&state={}",
        urlencoding::encode(naver.client_id.trim()),
        urlencoding::encode(naver.redirect_uri.trim()),
        urlencoding::encode(state.trim()),
    )
}

fn generate_ai_draft(ai: &AiConfig) -> Result<Draft, String> {
    let api_key = if ai.api_key.trim().is_empty() {
        env::var("OPENAI_API_KEY").map_err(|err| err.to_string())?
    } else {
        ai.api_key.trim().to_owned()
    };
    let model = if ai.model.trim().is_empty() {
        "gpt-5.2"
    } else {
        ai.model.trim()
    };
    let prompt = format!(
        "너는 한국어 블로그 전문 작가다.\n\
         주제: {}\n\
         대상 독자: {}\n\
         반드시 포함할 키워드: {}\n\
         문체: {}\n\n\
         요구사항:\n\
         - 네이버 블로그와 티스토리에 바로 붙여넣을 수 있는 HTML 본문을 작성한다.\n\
         - 제목은 클릭을 유도하되 과장하지 않는다.\n\
         - 본문은 h2, h3, p, ul, li 태그만 사용한다.\n\
         - 한국어 문장은 자연스럽고 반복을 줄인다.\n\
         - 광고성 문구, 허위 사실, 출처 없는 통계는 쓰지 않는다.\n\
         - 결과는 마크다운 코드블록 없이 JSON만 반환한다.\n\
         - JSON 형식: {{\"title\":\"...\",\"body_html\":\"...\",\"tags\":[\"...\",\"...\"]}}",
        ai.topic.trim(),
        ai.audience.trim(),
        ai.keywords.trim(),
        ai.tone.trim()
    );

    let request_body = json!({
        "model": model,
        "input": prompt,
    });

    let response = reqwest::blocking::Client::new()
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .map_err(|err| err.to_string())?;

    let status = response.status();
    let body = response.text().map_err(|err| err.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {status}: {body}"));
    }

    let value: Value =
        serde_json::from_str(&body).map_err(|err| format!("응답 JSON 파싱 실패: {err}"))?;
    let output_text = extract_response_text(&value)
        .ok_or_else(|| format!("AI 응답에서 텍스트를 찾지 못했습니다: {body}"))?;
    draft_from_ai_json(&output_text)
}

fn extract_response_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        return Some(text.to_owned());
    }

    let output = value.get("output")?.as_array()?;
    let mut parts = Vec::new();
    for item in output {
        let Some(content) = item.get("content").and_then(Value::as_array) else {
            continue;
        };
        for content_item in content {
            if let Some(text) = content_item.get("text").and_then(Value::as_str) {
                parts.push(text.to_owned());
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

fn draft_from_ai_json(text: &str) -> Result<Draft, String> {
    let trimmed = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let value: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("AI 결과 JSON 파싱 실패: {err}. 응답: {text}"))?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("AI 생성 블로그 포스트")
        .trim()
        .to_owned();
    let body = value
        .get("body_html")
        .or_else(|| value.get("body"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_owned();
    let tags = match value.get("tags") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::String(tags)) => normalize_tags(tags),
        _ => String::new(),
    };

    if body.is_empty() {
        return Err(format!("AI 결과에 body_html이 없습니다. 응답: {text}"));
    }

    Ok(Draft {
        title,
        body,
        tags,
        visibility: Visibility::Private,
    })
}

fn exchange_naver_token(
    naver: &NaverConfig,
    code: &str,
    state: &str,
) -> Result<TokenResponse, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://nid.naver.com/oauth2.0/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", naver.client_id.trim()),
            ("client_secret", naver.client_secret.trim()),
            ("code", code.trim()),
            ("state", state.trim()),
        ])
        .send()
        .map_err(|err| err.to_string())?;

    parse_json_response(response)
}

fn refresh_naver_token(naver: &NaverConfig) -> Result<TokenResponse, String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://nid.naver.com/oauth2.0/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", naver.client_id.trim()),
            ("client_secret", naver.client_secret.trim()),
            ("refresh_token", naver.refresh_token.trim()),
        ])
        .send()
        .map_err(|err| err.to_string())?;

    parse_json_response(response)
}

fn publish_naver_post(naver: &NaverConfig, draft: &Draft) -> Result<String, String> {
    let mut form = vec![
        ("title".to_owned(), draft.title.trim().to_owned()),
        ("contents".to_owned(), draft.body.clone()),
        (
            "openYn".to_owned(),
            draft.visibility.as_naver_open_bool().to_owned(),
        ),
    ];

    if !naver.category_no.trim().is_empty() {
        form.push(("categoryNo".to_owned(), naver.category_no.trim().to_owned()));
    }

    let response = reqwest::blocking::Client::new()
        .post("https://openapi.naver.com/blog/writePost.json")
        .bearer_auth(naver.access_token.trim())
        .form(&form)
        .send()
        .map_err(|err| err.to_string())?;

    response_text(response)
}

fn fetch_naver_categories(naver: &NaverConfig) -> Result<String, String> {
    let response = reqwest::blocking::Client::new()
        .get("https://openapi.naver.com/blog/listCategory.json")
        .bearer_auth(naver.access_token.trim())
        .send()
        .map_err(|err| err.to_string())?;

    response_text(response)
}

fn parse_json_response(response: reqwest::blocking::Response) -> Result<TokenResponse, String> {
    let status = response.status();
    let body = response.text().map_err(|err| err.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {status}: {body}"));
    }
    serde_json::from_str(&body).map_err(|err| format!("JSON 파싱 실패: {err}. 응답: {body}"))
}

fn response_text(response: reqwest::blocking::Response) -> Result<String, String> {
    let status = response.status();
    let body = response.text().map_err(|err| err.to_string())?;
    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("HTTP {status}: {body}"))
    }
}

fn load_state() -> Result<SavedState, String> {
    let text = fs::read_to_string(CONFIG_FILE).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn save_state(state: &SavedState) -> Result<PathBuf, String> {
    let text = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    fs::write(CONFIG_FILE, text).map_err(|err| err.to_string())?;
    Ok(PathBuf::from(CONFIG_FILE))
}

fn export_html(draft: &Draft) -> Result<PathBuf, String> {
    let dir = PathBuf::from("exports");
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let slug = slugify(&draft.title);
    let path = dir.join(format!(
        "{}_{}.html",
        Local::now().format("%Y%m%d_%H%M%S"),
        slug
    ));
    let html = format!(
        "<!doctype html>\n<html lang=\"ko\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<meta name=\"keywords\" content=\"{}\">\n</head>\n<body>\n{}\n</body>\n</html>\n",
        escape_html(&draft.title),
        escape_html(&normalize_tags(&draft.tags)),
        draft.body
    );
    fs::write(&path, html).map_err(|err| err.to_string())?;
    Ok(path)
}

fn normalize_tags(tags: &str) -> String {
    tags.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

fn slugify(title: &str) -> String {
    let slug = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(['-', '_'])
        .to_owned();

    if slug.is_empty() {
        "post".to_owned()
    } else {
        slug
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
