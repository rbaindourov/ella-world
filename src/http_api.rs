use crate::auth::{constant_time_str_eq, TokenClaims};
use crate::models::{now_ms, AvatarAppearance, UserAccount};
use crate::udp_native::NativeUdpServer;
use crate::webrtc_sfu::WebRtcSfu;
use crate::world::WorldEngine;
use crate::ws_spine::{ws_upgrade_handler, WsSpineHub};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<WorldEngine>,
    pub sfu: Arc<WebRtcSfu>,
    pub native_udp: Arc<NativeUdpServer>,
    pub ws_hub: Arc<WsSpineHub>,
}

pub fn build_router(state: AppState) -> Router {
    let public_dir = state.world.config.public_dir.clone();
    let index_file = public_dir.join("index.html");

    let api_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/meta", get(meta_handler))
        .route("/ws", get(ws_upgrade_handler))
        .route("/login", post(login_handler))
        .route("/resume", post(resume_handler))
        .route("/logout", post(logout_handler))
        .route("/rtc/offer", post(rtc_offer_handler))
        .route("/block", post(block_handler))
        .route("/avatar", post(avatar_handler))
        .route("/chat", post(chat_handler))
        .route("/runway", post(runway_handler))
        .route("/admin/login", post(admin_login_handler))
        .route("/admin/state", get(admin_state_handler))
        .route("/admin/chats", get(admin_chats_handler))
        .route("/admin/users", post(admin_upsert_user_handler))
        .route("/admin/users/:username", delete(admin_delete_user_handler))
        .route("/admin/settings", post(admin_settings_handler))
        .route("/admin/world/reset", post(admin_reset_world_handler));

    let static_service = ServeDir::new(&public_dir).not_found_service(ServeFile::new(&index_file));

    Router::new()
        .route("/health", get(health_handler))
        .nest("/api/ella-world", api_routes.clone())
        .nest("/api", api_routes)
        .nest_service("/ella-world", static_service.clone())
        .fallback_service(static_service)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn json_err(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "ok": false, "error": msg }))).into_response()
}

fn extract_player_claims(world: &WorldEngine, token: &str) -> Option<(String, String)> {
    let claims = world.verify_session_token(token)?;
    let self_id = claims.self_id?;
    let username = claims.username?;
    Some((self_id, username))
}

fn verify_admin_bearer(world: &WorldEngine, headers: &HeaderMap) -> bool {
    let Some(raw) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    let Some(claims) = world.verify_session_token(token) else {
        return false;
    };
    claims.role.as_deref() == Some("admin")
}

async fn build_admin_dashboard(world: &WorldEngine) -> Value {
    let cfg = world.users_cfg.read().await;
    let active = world.players.read().await;
    let blocks_count = world.blocks_data.read().await.blocks.len();

    let users_list: Vec<Value> = cfg
        .users
        .iter()
        .map(|u| {
            let online = active
                .values()
                .any(|p| p.user.username.eq_ignore_ascii_case(&u.username));
            json!({
                "username": u.username,
                "displayName": if u.display_name.is_empty() { &u.username } else { &u.display_name },
                "password": u.password,
                "role": u.role,
                "avatar": u.avatar,
                "online": online
            })
        })
        .collect();

    json!({
        "worldName": cfg.world_name,
        "voiceDefaultMode": cfg.voice_default_mode,
        "blockCount": blocks_count,
        "chatFilterEnabled": cfg.chat_filter_enabled,
        "blockedWords": cfg.blocked_words,
        "chatCount": world.cached_chat_count.load(std::sync::atomic::Ordering::Relaxed),
        "blockedChatCount": world.cached_blocked_chat_count.load(std::sync::atomic::Ordering::Relaxed),
        "users": users_list
    })
}

async fn health_handler(State(st): State<AppState>) -> Json<Value> {
    let active_players = st.world.players.read().await.len();
    let blocks_version = st.world.blocks_data.read().await.version;
    Json(json!({
        "status": "ok",
        "engine": "ella-world-rust-udp",
        "version": env!("CARGO_PKG_VERSION"),
        "wsSpineEnabled": true,
        "zeroRenegotiationSfu": true,
        "sseEnabled": false,
        "httpPollingSyncEnabled": false,
        "transport": {
            "spine": "wss-binary-and-json-nodelay + 3s-self-healing-digest + 12s-grace",
            "voice": "udp-webrtc-sfu-8slot-zero-renegotiate + native-udp-opus",
            "gameState": "udp-webrtc-unreliable-datachannel-20hz + wss-binary-20hz-fallback + native-udp-ewld-v1",
            "wsPath": "/api/ella-world/ws",
            "webrtcUdpPort": st.sfu.udp_port(),
            "nativeUdpPort": st.native_udp.local_addr().port()
        },
        "webrtcUdpPort": st.sfu.udp_port(),
        "nativeUdpPort": st.native_udp.local_addr().port(),
        "activePlayers": active_players,
        "activeWsSessions": st.ws_hub.active_session_count(),
        "activeWebrtcSessions": st.sfu.active_session_count(),
        "activeNativeUdpSessions": st.native_udp.active_session_count(),
        "blocksVersion": blocks_version,
        "stateSeq": st.world.current_state_seq(),
        "serverTimeMs": now_ms(),
        "serverTick": st.world.current_tick()
    }))
}

async fn meta_handler(State(st): State<AppState>) -> Json<Value> {
    st.world.prune_stale_players().await;
    let cfg = st.world.users_cfg.read().await;
    let online_count = st.world.players.read().await.len();
    let runway = st.world.runway.read().await.summary();
    Json(json!({
        "worldName": cfg.world_name,
        "onlineCount": online_count,
        "voiceDefaultMode": cfg.voice_default_mode,
        "iceServers": cfg.ice_servers,
        "runway": runway,
        "transport": "hybrid-ws-spine-plus-udp-sfu",
        "wsUrl": "/api/ella-world/ws",
        "sseEnabled": false,
        "serverTimeMs": now_ms(),
        "webrtcUdpPort": st.sfu.udp_port(),
        "nativeUdpPort": st.native_udp.local_addr().port()
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginPayload {
    username: String,
    password: String,
    client_id: Option<String>,
}

async fn login_handler(State(st): State<AppState>, Json(body): Json<LoginPayload>) -> Response {
    let uname = body.username.trim();
    if uname.is_empty() || body.password.is_empty() {
        return json_err(
            StatusCode::BAD_REQUEST,
            "Please enter your username and password.",
        );
    }

    let found_user = {
        let cfg = st.world.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| {
                u.username.eq_ignore_ascii_case(uname)
                    && constant_time_str_eq(&u.password, &body.password)
            })
            .cloned()
    };

    let Some(user) = found_user else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid username or password. Ask a parent to check the Admin Panel!",
        );
    };

    let session_doc = st
        .world
        .create_or_resume_session(&user, body.client_id.as_deref())
        .await;
    Json(session_doc).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResumePayload {
    token: String,
    client_id: Option<String>,
}

async fn resume_handler(State(st): State<AppState>, Json(body): Json<ResumePayload>) -> Response {
    let Some((_old_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    let found_user = {
        let cfg = st.world.users_cfg.read().await;
        cfg.users
            .iter()
            .find(|u| u.username.eq_ignore_ascii_case(&username))
            .cloned()
    };

    let Some(user) = found_user else {
        return json_err(StatusCode::UNAUTHORIZED, "Account no longer active.");
    };

    let session_doc = st
        .world
        .create_or_resume_session(&user, body.client_id.as_deref())
        .await;
    Json(session_doc).into_response()
}

#[derive(Deserialize)]
struct TokenOnlyPayload {
    token: String,
}

async fn logout_handler(
    State(st): State<AppState>,
    Json(body): Json<TokenOnlyPayload>,
) -> Response {
    if let Some((self_id, _)) = extract_player_claims(&st.world, &body.token) {
        st.sfu.cleanup_session(&self_id).await;
        st.world.remove_player_immediately(&self_id).await;
    }
    Json(json!({ "ok": true })).into_response()
}

#[derive(Deserialize)]
struct RtcOfferPayload {
    token: String,
    sdp: String,
}

async fn rtc_offer_handler(
    State(st): State<AppState>,
    Json(body): Json<RtcOfferPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid session token for UDP WebRTC.",
        );
    };
    if body.sdp.trim().is_empty() {
        return json_err(StatusCode::BAD_REQUEST, "Missing WebRTC SDP offer.");
    }

    match st
        .sfu
        .accept_browser_offer(self_id, username, body.sdp)
        .await
    {
        Ok(answer_sdp) => Json(json!({
            "ok": true,
            "sdp": answer_sdp,
            "udpPort": st.sfu.udp_port(),
            "transport": "udp"
        }))
        .into_response(),
        Err(err) => json_err(
            StatusCode::BAD_REQUEST,
            &format!("Failed to negotiate UDP WebRTC session: {err}"),
        ),
    }
}

#[derive(Deserialize)]
struct BlockMutationPayload {
    token: String,
    #[serde(default = "default_place_action")]
    action: String,
    x: i32,
    y: i32,
    z: i32,
    color: Option<String>,
}

fn default_place_action() -> String {
    "place".into()
}

async fn block_handler(
    State(st): State<AppState>,
    Json(body): Json<BlockMutationPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(StatusCode::UNAUTHORIZED, "Invalid session token.");
    };

    match st
        .world
        .mutate_block(
            &self_id,
            &username,
            &body.action,
            (body.x, body.y, body.z),
            body.color.as_deref(),
        )
        .await
    {
        Ok((blocks_version, blocks)) => Json(json!({
            "ok": true,
            "blocksVersion": blocks_version,
            "blocks": blocks
        }))
        .into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, msg),
    }
}

#[derive(Deserialize)]
struct AvatarUpdatePayload {
    token: String,
    avatar: Value,
}

async fn avatar_handler(
    State(st): State<AppState>,
    Json(body): Json<AvatarUpdatePayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(StatusCode::UNAUTHORIZED, "Invalid session token.");
    };

    match st
        .world
        .update_avatar(&self_id, &username, &body.avatar)
        .await
    {
        Some(sanitized) => Json(json!({
            "ok": true,
            "user": sanitized
        }))
        .into_response(),
        None => json_err(StatusCode::NOT_FOUND, "User account not found."),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChatPostPayload {
    token: String,
    text: Option<String>,
    #[serde(default)]
    is_trophy_win: bool,
}

async fn chat_handler(State(st): State<AppState>, Json(body): Json<ChatPostPayload>) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(StatusCode::UNAUTHORIZED, "Invalid session token.");
    };

    match st
        .world
        .post_chat(
            &self_id,
            &username,
            body.text.as_deref(),
            body.is_trophy_win,
        )
        .await
    {
        Ok(message) => Json(json!({
            "ok": true,
            "message": message
        }))
        .into_response(),
        Err(reason) => json_err(StatusCode::BAD_REQUEST, &reason),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunwayActionPayload {
    token: String,
    #[serde(default = "default_vote_action")]
    action: String,
    theme: Option<String>,
    target_username: Option<String>,
    stars: Option<u8>,
}

fn default_vote_action() -> String {
    "vote".into()
}

async fn runway_handler(
    State(st): State<AppState>,
    Json(body): Json<RunwayActionPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token) else {
        return json_err(StatusCode::UNAUTHORIZED, "Invalid session token.");
    };

    let runway = st
        .world
        .mutate_runway(
            &self_id,
            &username,
            &body.action,
            body.theme.as_deref(),
            body.target_username.as_deref(),
            body.stars,
        )
        .await;

    Json(json!({
        "ok": true,
        "runway": runway
    }))
    .into_response()
}

#[derive(Deserialize)]
struct AdminLoginPayload {
    password: String,
}

async fn admin_login_handler(
    State(st): State<AppState>,
    Json(body): Json<AdminLoginPayload>,
) -> Response {
    let valid = {
        let cfg = st.world.users_cfg.read().await;
        constant_time_str_eq(&cfg.admin_password, &body.password)
    };
    if !valid {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Incorrect Parent Master Password.",
        );
    }

    let admin_token = st.world.sign_session_token(&TokenClaims {
        self_id: None,
        username: Some("admin".into()),
        role: Some("admin".into()),
        iat: now_ms(),
    });
    let dashboard = build_admin_dashboard(&st.world).await;

    Json(json!({
        "ok": true,
        "adminToken": admin_token,
        "dashboard": dashboard
    }))
    .into_response()
}

async fn admin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }
    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminUpsertUserPayload {
    username: String,
    #[serde(default)]
    display_name: String,
    password: String,
    role: Option<String>,
    avatar: Option<Value>,
}

async fn admin_upsert_user_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminUpsertUserPayload>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let clean_username: String = body
        .username
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(24)
        .collect();

    if clean_username.len() < 2 || body.password.trim().is_empty() {
        return json_err(
            StatusCode::BAD_REQUEST,
            "Username (2+ alphanumeric chars) and password are required.",
        );
    }

    let display_name = if body.display_name.trim().is_empty() {
        clean_username.clone()
    } else {
        body.display_name.trim().chars().take(32).collect()
    };

    {
        let mut cfg = st.world.users_cfg.write().await;
        if let Some(existing) = cfg
            .users
            .iter_mut()
            .find(|u| u.username.eq_ignore_ascii_case(&clean_username))
        {
            existing.display_name = display_name;
            existing.password = body.password.trim().to_string();
            if let Some(r) = &body.role {
                existing.role = r.clone();
            }
            if let Some(av_patch) = &body.avatar {
                existing.avatar.apply_patch(av_patch);
            }
        } else {
            let mut av = AvatarAppearance::default();
            if let Some(av_patch) = &body.avatar {
                av.apply_patch(av_patch);
            }
            cfg.users.push(UserAccount {
                username: clean_username,
                display_name,
                password: body.password.trim().to_string(),
                role: body.role.unwrap_or_else(|| "friend".into()),
                avatar: av,
                last_pos: None,
                last_pos_saved_at: None,
            });
        }
    }
    st.world.persist_users_async().await;

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

async fn admin_delete_user_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let target = username.trim().to_lowercase();
    {
        let mut cfg = st.world.users_cfg.write().await;
        cfg.users
            .retain(|u| !u.username.eq_ignore_ascii_case(&target));
    }
    st.world.persist_users_async().await;

    let peer_ids_to_kick: Vec<String> = {
        let map = st.world.players.read().await;
        map.values()
            .filter(|p| p.user.username.eq_ignore_ascii_case(&target))
            .map(|p| p.id.clone())
            .collect()
    };
    for pid in peer_ids_to_kick {
        st.sfu.cleanup_session(&pid).await;
        st.world.remove_player_immediately(&pid).await;
    }

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminChatsQuery {
    limit: Option<usize>,
    user: Option<String>,
    search: Option<String>,
    #[serde(default)]
    include_blocked: bool,
}

async fn admin_chats_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminChatsQuery>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let limit = query.limit.unwrap_or(200).clamp(1, 1000);
    let user_filter = query.user.as_deref();
    let search_filter = query.search.as_deref();
    let (total, chats) = st.world.store.query_chats(limit, user_filter, search_filter);
    let (blocked_total, blocked_chats) = if query.include_blocked {
        st.world
            .store
            .query_blocked_chats(limit, user_filter, search_filter)
    } else {
        (st.world.store.count_blocked_chats(), Vec::new())
    };
    let cfg = st.world.users_cfg.read().await;

    Json(json!({
        "ok": true,
        "total": total,
        "chats": chats,
        "blockedTotal": blocked_total,
        "blockedChats": blocked_chats,
        "chatFilterEnabled": cfg.chat_filter_enabled,
        "blockedWords": cfg.blocked_words
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminSettingsPayload {
    world_name: Option<String>,
    voice_default_mode: Option<String>,
    new_admin_password: Option<String>,
    chat_filter_enabled: Option<bool>,
    blocked_words: Option<Vec<String>>,
}

async fn admin_settings_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminSettingsPayload>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    {
        let mut cfg = st.world.users_cfg.write().await;
        if let Some(name) = body
            .world_name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            cfg.world_name = name.chars().take(64).collect();
        }
        if let Some(mode) = body.voice_default_mode {
            if mode == "proximity" || mode == "party" {
                cfg.voice_default_mode = mode;
            }
        }
        if let Some(new_pw) = body
            .new_admin_password
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() >= 4)
        {
            cfg.admin_password = new_pw;
        }
        if let Some(enabled) = body.chat_filter_enabled {
            cfg.chat_filter_enabled = enabled;
        }
        if let Some(words) = body.blocked_words {
            let mut seen = HashSet::new();
            let mut clean = Vec::new();
            for raw in words {
                let w = raw.trim().to_lowercase();
                if w.len() >= 2 && w.len() <= 48 && seen.insert(w.clone()) {
                    clean.push(w);
                }
            }
            cfg.blocked_words = clean;
        }
    }
    st.world.persist_users_async().await;

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

#[derive(Deserialize)]
struct AdminResetWorldPayload {
    #[serde(default)]
    mode: String,
}

async fn admin_reset_world_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminResetWorldPayload>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    st.world.reset_world_blocks(body.mode == "clear").await;
    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}
