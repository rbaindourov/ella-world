use crate::auth::constant_time_str_eq;
use crate::device_sig::{
    active_penalties_for_user, build_machine_seal_set_cookie, extract_effective_client_ip,
    extract_machine_seal_cookie, ClientDeviceTelemetry, ClientPeerAddr,
};
use crate::economy::ITEM_CATALOG;
use crate::models::{
    normalize_chat_mode, now_ms, AvatarAppearance, UserAccount, SAFE_QUICK_CHAT_PHRASES,
};
use crate::udp_native::NativeUdpServer;
use crate::webrtc_sfu::WebRtcSfu;
use crate::world::{
    is_within_curfew_window, local_minutes_since_midnight, parse_hhmm_minutes,
    AdminManageDeviceInput, AdminModerateUserInput, BlockMutationInput, SubadminModerateInput,
    WorldEngine,
};
use crate::ws_spine::{ws_upgrade_handler, WsSpineHub};
use axum::extract::{Extension, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
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
    pub tls_enabled: bool,
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
        .route("/raygun", post(raygun_handler))
        .route("/avatar", post(avatar_handler))
        .route("/chat", post(chat_handler))
        .route("/runway", post(runway_handler))
        .route("/economy/catalog", get(economy_catalog_handler))
        .route("/economy/use", post(economy_use_handler))
        .route("/dungeon/portal", post(dungeon_portal_handler))
        .route("/dungeon/interact", post(dungeon_interact_handler))
        .route("/dungeon/attack", post(dungeon_attack_handler))
        .route("/dungeon/pickup", post(dungeon_pickup_handler))
        .route("/prison/work", post(prison_work_handler))
        .route("/trade/invite", post(trade_invite_handler))
        .route("/trade/respond", post(trade_respond_handler))
        .route("/trade/offer", post(trade_offer_handler))
        .route("/trade/lock", post(trade_lock_handler))
        .route("/trade/confirm", post(trade_confirm_handler))
        .route("/trade/cancel", post(trade_cancel_handler))
        .route("/admin/login", post(admin_login_handler))
        .route("/admin/state", get(admin_state_handler))
        .route("/admin/chats", get(admin_chats_handler))
        .route("/admin/users", post(admin_upsert_user_handler))
        .route("/admin/users/:username", delete(admin_delete_user_handler))
        .route("/admin/moderate", post(admin_moderate_handler))
        .route("/subadmin/state", get(subadmin_state_handler))
        .route("/subadmin/moderate", post(subadmin_moderate_handler))
        .route("/admin/devices", post(admin_devices_handler))
        .route(
            "/admin/sessions/revoke",
            post(admin_revoke_sessions_handler),
        )
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

async fn extract_player_claims(world: &WorldEngine, token: &str) -> Option<(String, String)> {
    let (self_id, username, _, _, _, _) = world.verify_active_player_token(token).await?;
    Some((self_id, username))
}

async fn verify_admin_bearer(world: &WorldEngine, headers: &HeaderMap) -> bool {
    let Some(raw) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    world.verify_active_admin_token(token).await
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
            let device_count = cfg
                .devices
                .iter()
                .filter(|d| {
                    d.associated_users
                        .iter()
                        .any(|au| au.eq_ignore_ascii_case(&u.username))
                })
                .count();
            json!({
                "username": u.username,
                "displayName": if u.display_name.is_empty() { &u.username } else { &u.display_name },
                "password": u.password,
                "role": u.role,
                "tokenEpoch": u.token_epoch,
                "coins": u.coins,
                "itemCount": u.inventory.values().sum::<u32>(),
                "violationStrikes": u.violation_strikes,
                "inDebtorsPrison": u.in_debtors_prison,
                "prisonReason": u.prison_reason,
                "prisonTasksRemaining": u.prison_tasks_remaining,
                "voiceMutedByParent": u.voice_muted_by_parent,
                "chatMode": normalize_chat_mode(&u.chat_mode),
                "suspended": u.suspended,
                "canBuild": u.can_build,
                "canTrade": u.can_trade,
                "deviceCount": device_count,
                "avatar": u.avatar,
                "online": online
            })
        })
        .collect();

    let mut devices_list: Vec<Value> = cfg
        .devices
        .iter()
        .map(|d| {
            let online_count = active
                .values()
                .filter(|p| p.device_id.as_deref() == Some(d.device_id.as_str()))
                .count();
            let mut locked_by_user: Option<String> = None;
            let mut active_pens: Vec<String> = Vec::new();
            if cfg.enforce_machine_signatures && !d.trusted_shared {
                for au in &d.associated_users {
                    if let Some(u_acc) = cfg
                        .users
                        .iter()
                        .find(|u| u.username.eq_ignore_ascii_case(au))
                    {
                        let pens = active_penalties_for_user(u_acc);
                        if !pens.is_empty() {
                            locked_by_user = Some(u_acc.username.clone());
                            active_pens = pens;
                            break;
                        }
                    }
                }
            }
            json!({
                "deviceId": d.device_id,
                "shortSig": d.short_sig,
                "label": d.label,
                "osFamily": d.os_family,
                "gpuSummary": d.gpu_summary,
                "screenSummary": d.screen_summary,
                "lastIp": d.last_ip,
                "associatedUsers": d.associated_users,
                "lastUsername": d.last_username,
                "firstSeenMs": d.first_seen_ms,
                "lastSeenMs": d.last_seen_ms,
                "trustedShared": d.trusted_shared,
                "banned": d.banned,
                "lockedByUser": locked_by_user,
                "activePenalties": active_pens,
                "onlineCount": online_count
            })
        })
        .collect();
    devices_list.sort_by(|a, b| {
        let ta = a.get("lastSeenMs").and_then(Value::as_u64).unwrap_or(0);
        let tb = b.get("lastSeenMs").and_then(Value::as_u64).unwrap_or(0);
        tb.cmp(&ta)
    });

    let recent_evasions = world.store.load_recent_device_evasions(80);

    json!({
        "worldName": cfg.world_name,
        "sessionEpoch": cfg.session_epoch,
        "voiceDefaultMode": cfg.voice_default_mode,
        "globalVoiceEnabled": cfg.global_voice_enabled,
        "globalChatMode": normalize_chat_mode(&cfg.global_chat_mode),
        "worldLocked": cfg.world_locked,
        "curfewEnabled": cfg.curfew_enabled,
        "curfewStart": cfg.curfew_start,
        "curfewEnd": cfg.curfew_end,
        "profanityFineBase": cfg.profanity_fine_base,
        "debtPrisonThreshold": cfg.debt_prison_threshold,
        "prisonStrikeThreshold": cfg.prison_strike_threshold,
        "enforceMachineSignatures": cfg.enforce_machine_signatures,
        "safeQuickChatPhrases": SAFE_QUICK_CHAT_PHRASES,
        "blockCount": blocks_count,
        "chatFilterEnabled": cfg.chat_filter_enabled,
        "blockedWords": cfg.blocked_words,
        "chatCount": world.cached_chat_count.load(std::sync::atomic::Ordering::Relaxed),
        "blockedChatCount": world.cached_blocked_chat_count.load(std::sync::atomic::Ordering::Relaxed),
        "deviceEvasionCount": world.cached_device_evasion_count.load(std::sync::atomic::Ordering::Relaxed),
        "devices": devices_list,
        "recentDeviceEvasions": recent_evasions,
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
        "globalVoiceEnabled": cfg.global_voice_enabled,
        "worldLocked": cfg.world_locked,
        "curfewEnabled": cfg.curfew_enabled,
        "curfewStart": cfg.curfew_start,
        "curfewEnd": cfg.curfew_end,
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
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

fn attach_machine_seal_cookie(mut resp: Response, seal_token: &str, tls_enabled: bool) -> Response {
    let cookie_val = build_machine_seal_set_cookie(seal_token, tls_enabled);
    if let Ok(hv) = HeaderValue::from_str(&cookie_val) {
        resp.headers_mut().insert(header::SET_COOKIE, hv);
    }
    resp
}

async fn login_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<LoginPayload>,
) -> Response {
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

    let cookie_seal = extract_machine_seal_cookie(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let peer_ip = extract_effective_client_ip(&headers, peer_ext.map(|e| e.0 .0));

    let resolved_machine = match st
        .world
        .resolve_and_authorize_machine_login(
            &user.username,
            cookie_seal.as_deref(),
            body.machine_seal.as_deref(),
            None,
            body.device_telemetry.as_ref(),
            user_agent,
            peer_ip,
        )
        .await
    {
        Ok(m) => m,
        Err(lock_reason) => return json_err(StatusCode::FORBIDDEN, &lock_reason),
    };

    if let Err(reason) = st.world.check_world_access_for_user(&user).await {
        let mut payload = json!({ "ok": false, "error": reason });
        if let Some(ref m) = resolved_machine {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("machineSeal".into(), json!(m.seal_token));
                obj.insert("machineShortSig".into(), json!(m.short_sig));
            }
            let resp = (StatusCode::FORBIDDEN, Json(payload)).into_response();
            return attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled);
        }
        return (StatusCode::FORBIDDEN, Json(payload)).into_response();
    }

    let session_doc = st
        .world
        .create_or_resume_session(&user, body.client_id.as_deref(), resolved_machine.as_ref())
        .await;
    let resp = Json(session_doc).into_response();
    if let Some(ref m) = resolved_machine {
        attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled)
    } else {
        resp
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResumePayload {
    token: String,
    client_id: Option<String>,
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

async fn resume_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<ResumePayload>,
) -> Response {
    let Some((_old_id, _username, user, _, _, claim_device_id)) =
        st.world.verify_player_token_claims_only(&body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Your session has expired or your password was updated. Please sign in again.",
        );
    };

    let cookie_seal = extract_machine_seal_cookie(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let peer_ip = extract_effective_client_ip(&headers, peer_ext.map(|e| e.0 .0));

    let resolved_machine = match st
        .world
        .resolve_and_authorize_machine_login(
            &user.username,
            cookie_seal.as_deref(),
            body.machine_seal.as_deref(),
            claim_device_id.as_deref(),
            body.device_telemetry.as_ref(),
            user_agent,
            peer_ip,
        )
        .await
    {
        Ok(m) => m,
        Err(lock_reason) => return json_err(StatusCode::FORBIDDEN, &lock_reason),
    };

    if let Err(reason) = st.world.check_world_access_for_user(&user).await {
        let mut payload = json!({ "ok": false, "error": reason });
        if let Some(ref m) = resolved_machine {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("machineSeal".into(), json!(m.seal_token));
                obj.insert("machineShortSig".into(), json!(m.short_sig));
            }
            let resp = (StatusCode::FORBIDDEN, Json(payload)).into_response();
            return attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled);
        }
        return (StatusCode::FORBIDDEN, Json(payload)).into_response();
    }

    let session_doc = st
        .world
        .create_or_resume_session(&user, body.client_id.as_deref(), resolved_machine.as_ref())
        .await;
    let resp = Json(session_doc).into_response();
    if let Some(ref m) = resolved_machine {
        attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled)
    } else {
        resp
    }
}

#[derive(Deserialize)]
struct TokenOnlyPayload {
    token: String,
}

async fn logout_handler(
    State(st): State<AppState>,
    Json(body): Json<TokenOnlyPayload>,
) -> Response {
    if let Some((self_id, _)) = extract_player_claims(&st.world, &body.token).await {
        st.sfu.cleanup_session(&self_id).await;
        st.native_udp.evict_sessions_for_peer(&self_id).await;
        st.world
            .kick_and_remove_peer(&self_id, "Signed out of ELLA World.")
            .await;
    } else if let Some(unverified) = st.world.verify_session_token(body.token.trim()) {
        if let Some(self_id) = unverified.self_id {
            st.sfu.cleanup_session(&self_id).await;
            st.native_udp.evict_sessions_for_peer(&self_id).await;
            st.world.remove_player_immediately(&self_id).await;
        }
    }
    // Intentionally do NOT send Clear-Site-Data here so `__Host-ewld_machine_seal` /
    // `ewld_machine_seal` and IndexedDB/localStorage machine seals persist across sign-outs!
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
    let Some((self_id, username, _, _, _, device_id)) =
        st.world.verify_active_player_token(&body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token for UDP WebRTC.",
        );
    };
    if body.sdp.trim().is_empty() {
        return json_err(StatusCode::BAD_REQUEST, "Missing WebRTC SDP offer.");
    }

    match st
        .sfu
        .accept_browser_offer(self_id, username, device_id, body.sdp)
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
#[serde(rename_all = "camelCase")]
struct BlockMutationPayload {
    token: String,
    #[serde(default = "default_place_action")]
    action: String,
    x: i32,
    y: i32,
    z: i32,
    color: Option<String>,
    kind: Option<String>,
    rot: Option<u8>,
    #[serde(alias = "loot_item_id")]
    loot_item_id: Option<String>,
}

fn default_place_action() -> String {
    "place".into()
}

async fn block_handler(
    State(st): State<AppState>,
    Json(body): Json<BlockMutationPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match st
        .world
        .mutate_block_ext(
            &self_id,
            &username,
            BlockMutationInput {
                action: &body.action,
                coords: (body.x, body.y, body.z),
                color: body.color.as_deref(),
                kind: body.kind.as_deref(),
                rot: body.rot,
                loot_item_id: body.loot_item_id.as_deref(),
            },
        )
        .await
    {
        Ok((blocks_version, blocks)) => {
            let economy = {
                let cfg = st.world.users_cfg.read().await;
                cfg.users
                    .iter()
                    .find(|u| u.username.eq_ignore_ascii_case(&username))
                    .map(|u| u.economy_snapshot())
            };
            Json(json!({
                "ok": true,
                "blocksVersion": blocks_version,
                "blocks": blocks,
                "economy": economy
            }))
            .into_response()
        }
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RayGunPayload {
    token: String,
    #[serde(default)]
    x: f32,
    #[serde(default = "default_raygun_y")]
    y: f32,
    #[serde(default = "default_raygun_z")]
    z: f32,
}

fn default_raygun_y() -> f32 {
    1.0
}

fn default_raygun_z() -> f32 {
    8.0
}

async fn raygun_handler(State(st): State<AppState>, Json(body): Json<RayGunPayload>) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .fire_plaza_raygun(&self_id, &username, [body.x, body.y, body.z])
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
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
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
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
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
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
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
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

    let admin_token = st.world.mint_admin_token().await;
    let dashboard = build_admin_dashboard(&st.world).await;

    Json(json!({
        "ok": true,
        "adminToken": admin_token,
        "dashboard": dashboard
    }))
    .into_response()
}

async fn admin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
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
    if !verify_admin_bearer(&st.world, &headers).await {
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

    let clean_pw = body.password.trim().to_string();
    if clean_username.len() < 2 || clean_pw.is_empty() {
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

    let mut password_rotated = false;
    {
        let mut cfg = st.world.users_cfg.write().await;
        if let Some(existing) = cfg
            .users
            .iter_mut()
            .find(|u| u.username.eq_ignore_ascii_case(&clean_username))
        {
            if !constant_time_str_eq(&existing.password, &clean_pw) {
                password_rotated = true;
                existing.token_epoch = existing.token_epoch.wrapping_add(1);
            }
            existing.display_name = display_name;
            existing.password = clean_pw;
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
                username: clean_username.clone(),
                display_name,
                password: clean_pw,
                role: body.role.unwrap_or_else(|| "friend".into()),
                token_epoch: 0,
                avatar: av,
                coins: 0,
                inventory: HashMap::new(),
                violation_strikes: 0,
                in_debtors_prison: false,
                prison_reason: None,
                prison_tasks_remaining: 0,
                voice_muted_by_parent: false,
                chat_mode: "normal".into(),
                suspended: false,
                can_build: true,
                can_trade: true,
                last_pos: None,
                last_pos_saved_at: None,
            });
        }
    }
    st.world.persist_users_async().await;

    let mut kicked_sessions = 0usize;
    if password_rotated {
        st.sfu.cleanup_sessions_for_username(&clean_username).await;
        st.native_udp
            .evict_sessions_for_username(&clean_username)
            .await;
        let kicked = st
            .world
            .kick_and_remove_user_sessions(
                &clean_username,
                "Your password was updated by a parent. Please sign in with your new password.",
            )
            .await;
        kicked_sessions = kicked.len();
    }

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({
        "ok": true,
        "passwordRotated": password_rotated,
        "kickedSessions": kicked_sessions,
        "dashboard": dashboard
    }))
    .into_response()
}

async fn admin_delete_user_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let target = username.trim().to_lowercase();
    {
        let mut cfg = st.world.users_cfg.write().await;
        cfg.users
            .retain(|u| !u.username.eq_ignore_ascii_case(&target));
    }
    st.world.persist_users_async().await;

    st.sfu.cleanup_sessions_for_username(&target).await;
    st.native_udp.evict_sessions_for_username(&target).await;
    let kicked = st
        .world
        .kick_and_remove_user_sessions(&target, "Your account was removed by a parent.")
        .await;

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({
        "ok": true,
        "kickedSessions": kicked.len(),
        "dashboard": dashboard
    }))
    .into_response()
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AdminRevokeSessionsPayload {
    username: Option<String>,
    reason: Option<String>,
}

async fn admin_revoke_sessions_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminRevokeSessionsPayload>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let target_user = body
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase);

    if let Some(uname) = target_user {
        let maybe_epoch = {
            let mut cfg = st.world.users_cfg.write().await;
            if let Some(u) = cfg
                .users
                .iter_mut()
                .find(|u| u.username.eq_ignore_ascii_case(&uname))
            {
                u.token_epoch = u.token_epoch.wrapping_add(1);
                Some(u.token_epoch)
            } else {
                None
            }
        };
        let Some(token_epoch) = maybe_epoch else {
            return json_err(StatusCode::NOT_FOUND, "User account not found.");
        };
        st.world.persist_users_async().await;

        let reason = body
            .reason
            .as_deref()
            .unwrap_or("Your session was signed out by a parent. Please log in again.");
        st.sfu.cleanup_sessions_for_username(&uname).await;
        st.native_udp.evict_sessions_for_username(&uname).await;
        let kicked = st.world.kick_and_remove_user_sessions(&uname, reason).await;
        let dashboard = build_admin_dashboard(&st.world).await;

        Json(json!({
            "ok": true,
            "scope": "user",
            "username": uname,
            "tokenEpoch": token_epoch,
            "kickedCount": kicked.len(),
            "dashboard": dashboard
        }))
        .into_response()
    } else {
        let new_session_epoch = {
            let mut cfg = st.world.users_cfg.write().await;
            cfg.session_epoch = cfg.session_epoch.wrapping_add(1).max(1);
            for u in &mut cfg.users {
                u.token_epoch = u.token_epoch.wrapping_add(1);
            }
            cfg.session_epoch
        };
        st.world.persist_users_async().await;

        let reason = body
            .reason
            .as_deref()
            .unwrap_or("All active sessions were expired by a parent. Please sign in again.");
        st.sfu.cleanup_all_sessions().await;
        st.native_udp.evict_all_sessions().await;
        let kicked = st.world.kick_and_remove_all_sessions(reason).await;
        let admin_token = st.world.mint_admin_token().await;
        let dashboard = build_admin_dashboard(&st.world).await;

        Json(json!({
            "ok": true,
            "scope": "all",
            "sessionEpoch": new_session_epoch,
            "kickedCount": kicked.len(),
            "adminToken": admin_token,
            "dashboard": dashboard
        }))
        .into_response()
    }
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
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let limit = query.limit.unwrap_or(200).clamp(1, 1000);
    let user_filter = query.user.as_deref();
    let search_filter = query.search.as_deref();
    let (total, chats) = st
        .world
        .store
        .query_chats(limit, user_filter, search_filter);
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
    global_voice_enabled: Option<bool>,
    global_chat_mode: Option<String>,
    world_locked: Option<bool>,
    curfew_enabled: Option<bool>,
    curfew_start: Option<String>,
    curfew_end: Option<String>,
    profanity_fine_base: Option<u64>,
    debt_prison_threshold: Option<i64>,
    prison_strike_threshold: Option<u32>,
    enforce_machine_signatures: Option<bool>,
    new_admin_password: Option<String>,
    chat_filter_enabled: Option<bool>,
    blocked_words: Option<Vec<String>>,
}

async fn admin_settings_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminSettingsPayload>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let mut admin_pw_changed = false;
    let mut should_evict_all = false;
    let mut penalized_users_to_sweep = Vec::new();
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
        if let Some(gv) = body.global_voice_enabled {
            cfg.global_voice_enabled = gv;
        }
        if let Some(gcm) = body.global_chat_mode {
            cfg.global_chat_mode = normalize_chat_mode(&gcm);
        }
        if let Some(wl) = body.world_locked {
            cfg.world_locked = wl;
        }
        if let Some(ce) = body.curfew_enabled {
            cfg.curfew_enabled = ce;
        }
        if let Some(cs) = body.curfew_start {
            if parse_hhmm_minutes(&cs).is_some() {
                cfg.curfew_start = cs.trim().to_string();
            }
        }
        if let Some(ce_str) = body.curfew_end {
            if parse_hhmm_minutes(&ce_str).is_some() {
                cfg.curfew_end = ce_str.trim().to_string();
            }
        }
        if let Some(fine) = body.profanity_fine_base {
            cfg.profanity_fine_base = fine.clamp(1, 500);
        }
        if let Some(debt_thresh) = body.debt_prison_threshold {
            cfg.debt_prison_threshold = debt_thresh.clamp(-5_000, -5);
        }
        if let Some(strike_thresh) = body.prison_strike_threshold {
            cfg.prison_strike_threshold = strike_thresh.clamp(1, 20);
        }
        if let Some(ems) = body.enforce_machine_signatures {
            cfg.enforce_machine_signatures = ems;
            if ems {
                penalized_users_to_sweep = cfg
                    .users
                    .iter()
                    .filter(|u| !active_penalties_for_user(u).is_empty())
                    .map(|u| u.username.clone())
                    .collect();
            }
        }
        if let Some(new_pw) = body
            .new_admin_password
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() >= 4)
        {
            if !constant_time_str_eq(&cfg.admin_password, &new_pw) {
                admin_pw_changed = true;
            }
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
        if cfg.world_locked
            || (cfg.curfew_enabled
                && is_within_curfew_window(
                    local_minutes_since_midnight(),
                    &cfg.curfew_start,
                    &cfg.curfew_end,
                ))
        {
            should_evict_all = true;
        }
    }
    st.world.persist_users_async().await;
    st.world.broadcast_parental_world_settings().await;
    if should_evict_all {
        st.sfu.cleanup_all_sessions().await;
        st.native_udp.evict_all_sessions().await;
    }
    for pu in penalized_users_to_sweep {
        let evicted_pids = st.world.evict_other_accounts_on_user_machines(&pu).await;
        for pid in evicted_pids {
            st.sfu.cleanup_session(&pid).await;
            st.native_udp.evict_sessions_for_peer(&pid).await;
        }
    }
    st.world.enforce_live_world_access().await;

    let dashboard = build_admin_dashboard(&st.world).await;
    if admin_pw_changed {
        let admin_token = st.world.mint_admin_token().await;
        Json(json!({
            "ok": true,
            "adminToken": admin_token,
            "dashboard": dashboard
        }))
        .into_response()
    } else {
        Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
    }
}

async fn admin_moderate_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminModerateUserInput>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let target_uname = body.username.trim().to_lowercase();
    let mut res = match st.world.admin_moderate_user(body).await {
        Ok(v) => v,
        Err(msg) => return json_err(StatusCode::BAD_REQUEST, &msg),
    };

    if let Some(arr) = res.get("evictedAltPeers").and_then(Value::as_array) {
        for pid_val in arr {
            if let Some(pid) = pid_val.as_str() {
                st.sfu.cleanup_session(pid).await;
                st.native_udp.evict_sessions_for_peer(pid).await;
            }
        }
    }

    if res.get("suspended").and_then(Value::as_bool) == Some(true) {
        st.sfu.cleanup_sessions_for_username(&target_uname).await;
        st.native_udp
            .evict_sessions_for_username(&target_uname)
            .await;
        let kicked = st
            .world
            .kick_and_remove_user_sessions(&target_uname, "🚫 Your account was paused by a parent.")
            .await;
        if let Some(obj) = res.as_object_mut() {
            obj.insert("kickedCount".into(), json!(kicked.len()));
        }
    }

    let dashboard = build_admin_dashboard(&st.world).await;
    if let Some(obj) = res.as_object_mut() {
        obj.insert("dashboard".into(), dashboard);
    }
    Json(res).into_response()
}

async fn extract_bearer_player_username(
    world: &WorldEngine,
    headers: &HeaderMap,
) -> Option<String> {
    let raw = headers.get("authorization").and_then(|v| v.to_str().ok())?;
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    let (_, username) = extract_player_claims(world, token).await?;
    Some(username)
}

async fn subadmin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some(actor_username) = extract_bearer_player_username(&st.world, &headers).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st.world.build_subadmin_friends_state(&actor_username).await {
        Ok(v) => Json(v).into_response(),
        Err((code, msg)) => json_err(
            StatusCode::from_u16(code).unwrap_or(StatusCode::FORBIDDEN),
            &msg,
        ),
    }
}

async fn subadmin_moderate_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubadminModerateInput>,
) -> Response {
    let Some(actor_username) = extract_bearer_player_username(&st.world, &headers).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let mut res = match st
        .world
        .subadmin_moderate_friend(&actor_username, body)
        .await
    {
        Ok(v) => v,
        Err((code, msg)) => {
            return json_err(
                StatusCode::from_u16(code).unwrap_or(StatusCode::FORBIDDEN),
                &msg,
            )
        }
    };
    for key in ["kickedPeers", "evictedAltPeers"] {
        if let Some(arr) = res.get(key).and_then(Value::as_array) {
            for pid_val in arr {
                if let Some(pid) = pid_val.as_str() {
                    st.sfu.cleanup_session(pid).await;
                    st.native_udp.evict_sessions_for_peer(pid).await;
                }
            }
        }
    }
    if let Ok(state_val) = st.world.build_subadmin_friends_state(&actor_username).await {
        if let Some(obj) = res.as_object_mut() {
            if let Some(friends) = state_val.get("friends").cloned() {
                obj.insert("friends".into(), friends);
            }
        }
    }
    Json(res).into_response()
}

async fn admin_devices_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminManageDeviceInput>,
) -> Response {
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    let (updated, kicked_peers) = match st.world.admin_manage_device(body).await {
        Ok(res) => res,
        Err(msg) => return json_err(StatusCode::BAD_REQUEST, &msg),
    };
    for pid in &kicked_peers {
        st.sfu.cleanup_session(pid).await;
        st.native_udp.evict_sessions_for_peer(pid).await;
    }

    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({
        "ok": true,
        "device": updated,
        "kickedCount": kicked_peers.len(),
        "dashboard": dashboard
    }))
    .into_response()
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
    if !verify_admin_bearer(&st.world, &headers).await {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    }

    st.world.reset_world_blocks(body.mode == "clear").await;
    let dashboard = build_admin_dashboard(&st.world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

// ============================================================================
// Crystal Cavern Dungeon, Persistent Economy & Live P2P Trade HTTP Endpoints
// ============================================================================

async fn economy_catalog_handler(State(st): State<AppState>) -> Json<Value> {
    let dungeon = st.world.dungeon.read().await.snapshot();
    Json(json!({
        "ok": true,
        "itemCatalog": ITEM_CATALOG,
        "items": ITEM_CATALOG,
        "dungeon": dungeon,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EconomyUsePayload {
    token: String,
    item_id: String,
}

async fn economy_use_handler(
    State(st): State<AppState>,
    Json(body): Json<EconomyUsePayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .use_inventory_item(&self_id, &username, body.item_id.trim())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DungeonPortalPayload {
    token: String,
    #[serde(default)]
    zone: Option<u8>,
    #[serde(default)]
    target_zone: Option<u8>,
}

async fn dungeon_portal_handler(
    State(st): State<AppState>,
    Json(body): Json<DungeonPortalPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let Some(target_zone) = body.zone.or(body.target_zone) else {
        return json_err(StatusCode::BAD_REQUEST, "Target zone is required.");
    };
    match st
        .world
        .transition_player_zone(&self_id, &username, target_zone)
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DungeonInteractPayload {
    token: String,
    node_id: String,
}

async fn dungeon_interact_handler(
    State(st): State<AppState>,
    Json(body): Json<DungeonInteractPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .dungeon_interact_node(&self_id, &username, body.node_id.trim())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DungeonAttackPayload {
    token: String,
    mob_id: String,
    #[serde(default = "default_blast_mode")]
    mode: String,
}

fn default_blast_mode() -> String {
    "blast".into()
}

async fn dungeon_attack_handler(
    State(st): State<AppState>,
    Json(body): Json<DungeonAttackPayload>,
) -> Response {
    let Some((self_id, _username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .dungeon_attack_mob(&self_id, body.mob_id.trim(), body.mode.trim())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DungeonPickupPayload {
    token: String,
    drop_id: String,
}

async fn dungeon_pickup_handler(
    State(st): State<AppState>,
    Json(body): Json<DungeonPickupPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .dungeon_pickup_drop(&self_id, &username, body.drop_id.trim())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrisonWorkPayload {
    token: String,
    node_id: String,
}

async fn prison_work_handler(
    State(st): State<AppState>,
    Json(body): Json<PrisonWorkPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .prison_perform_work(&self_id, &username, body.node_id.trim())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeInvitePayload {
    token: String,
    target_peer_id: String,
}

async fn trade_invite_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeInvitePayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .trade_invite(&self_id, &username, &body.target_peer_id)
        .await
    {
        Ok(trade) => Json(json!({ "ok": true, "trade": trade })).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeRespondPayload {
    token: String,
    trade_id: String,
    accept: bool,
}

async fn trade_respond_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeRespondPayload>,
) -> Response {
    let Some((self_id, _username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .trade_respond(&self_id, body.trade_id.trim(), body.accept)
        .await
    {
        Ok(trade) => {
            Json(json!({ "ok": true, "accepted": body.accept, "trade": trade })).into_response()
        }
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeOfferPayload {
    token: String,
    trade_id: String,
    #[serde(default)]
    coins: u64,
    #[serde(default)]
    items: HashMap<String, u32>,
}

async fn trade_offer_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeOfferPayload>,
) -> Response {
    let Some((self_id, username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .trade_update_offer(
            &self_id,
            &username,
            body.trade_id.trim(),
            body.coins,
            &body.items,
        )
        .await
    {
        Ok(trade) => Json(json!({ "ok": true, "trade": trade })).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeLockPayload {
    token: String,
    trade_id: String,
    locked: bool,
}

async fn trade_lock_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeLockPayload>,
) -> Response {
    let Some((self_id, _username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .trade_set_lock(&self_id, body.trade_id.trim(), body.locked)
        .await
    {
        Ok(trade) => Json(json!({ "ok": true, "trade": trade })).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeIdOnlyPayload {
    token: String,
    trade_id: String,
}

async fn trade_confirm_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeIdOnlyPayload>,
) -> Response {
    let Some((self_id, _username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st.world.trade_confirm(&self_id, body.trade_id.trim()).await {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TradeCancelPayload {
    token: String,
    trade_id: Option<String>,
}

async fn trade_cancel_handler(
    State(st): State<AppState>,
    Json(body): Json<TradeCancelPayload>,
) -> Response {
    let Some((self_id, _username)) = extract_player_claims(&st.world, &body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match st
        .world
        .trade_cancel(&self_id, body.trade_id.as_deref())
        .await
    {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}
