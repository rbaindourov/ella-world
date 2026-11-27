use crate::auth::{constant_time_str_eq, DEFAULT_WORLD_SLUG};
use crate::device_sig::{
    active_penalties_for_user, build_machine_seal_set_cookie, extract_effective_client_ip,
    extract_machine_seal_cookie, ClientDeviceTelemetry, ClientPeerAddr,
};
use crate::economy::{
    CRAFTING_RECIPES, ITEM_CATALOG, MOUNT_CATALOG, PET_CATALOG, PET_EGG_SHOP_PRICE,
};
use crate::models::{
    normalize_chat_mode, now_ms, AvatarAppearance, UserAccount, SAFE_QUICK_CHAT_PHRASES,
};
use crate::playground::{
    AdminInviteActionInput, PlaygroundRegistry, RedeemFamilyInviteInput, RedeemFriendInviteInput,
    SuperAdminInviteActionInput,
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
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<WorldEngine>,
    pub registry: Arc<PlaygroundRegistry>,
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
        .route("/register", post(uninvited_register_guard_handler))
        .route("/resume", post(resume_handler))
        .route("/logout", post(logout_handler))
        .route("/rtc/offer", post(rtc_offer_handler))
        .route("/block", post(block_handler))
        .route("/raygun", post(raygun_handler))
        .route("/avatar", post(avatar_handler))
        .route("/chat", post(chat_handler))
        .route("/runway", post(runway_handler))
        .route("/companions", post(companions_handler))
        .route("/homes", get(homes_get_handler).post(homes_post_handler))
        .route("/craft", get(craft_get_handler).post(craft_post_handler))
        .route("/obby", get(obby_get_handler).post(obby_post_handler))
        .route("/sky", get(sky_get_handler).post(sky_post_handler))
        .route("/arena/boss", post(arena_boss_handler))
        .route("/economy/catalog", get(economy_catalog_handler))
        .route("/economy/use", post(economy_use_handler))
        .route("/economy/clone", post(economy_clone_handler))
        .route("/dungeon/portal", post(dungeon_portal_handler))
        .route("/dungeon/interact", post(dungeon_interact_handler))
        .route("/dungeon/attack", post(dungeon_attack_handler))
        .route("/dungeon/pickup", post(dungeon_pickup_handler))
        .route("/arena/respawn", post(arena_respawn_handler))
        .route("/prison/work", post(prison_work_handler))
        .route("/trade/invite", post(trade_invite_handler))
        .route("/trade/respond", post(trade_respond_handler))
        .route("/trade/offer", post(trade_offer_handler))
        .route("/trade/lock", post(trade_lock_handler))
        .route("/trade/confirm", post(trade_confirm_handler))
        .route("/trade/cancel", post(trade_cancel_handler))
        .route("/playgrounds/public", get(playgrounds_public_handler))
        .route("/playgrounds/mine", get(playgrounds_mine_handler))
        .route("/playgrounds/switch", post(playgrounds_switch_handler))
        .route("/invites/inspect", get(invites_inspect_handler))
        .route("/invites/redeem", post(invites_redeem_handler))
        .route("/invites/join-world", post(invites_join_world_handler))
        .route(
            "/invites/redeem-family",
            post(invites_redeem_family_handler),
        )
        .route("/invites/host-draft", post(invites_host_draft_handler))
        .route("/admin/login", post(admin_login_handler))
        .route("/admin/state", get(admin_state_handler))
        .route(
            "/admin/invites",
            get(admin_invites_get_handler).post(admin_invites_post_handler),
        )
        .route("/admin/chats", get(admin_chats_handler))
        .route("/admin/users", post(admin_upsert_user_handler))
        .route("/admin/users/:username", delete(admin_delete_user_handler))
        .route("/admin/moderate", post(admin_moderate_handler))
        .route("/admin/gift-item", post(admin_gift_item_handler))
        .route("/subadmin/state", get(subadmin_state_handler))
        .route("/subadmin/moderate", post(subadmin_moderate_handler))
        .route("/admin/devices", post(admin_devices_handler))
        .route(
            "/admin/sessions/revoke",
            post(admin_revoke_sessions_handler),
        )
        .route("/admin/settings", post(admin_settings_handler))
        .route("/admin/world/reset", post(admin_reset_world_handler))
        .route("/superadmin/login", post(superadmin_login_handler))
        .route("/superadmin/state", get(superadmin_state_handler))
        .route("/superadmin/invites", post(superadmin_invites_handler))
        .route("/android/release", get(android_release_handler));

    let static_service = ServeDir::new(&public_dir).not_found_service(ServeFile::new(&index_file));

    Router::new()
        .route("/health", get(health_handler))
        .route("/download/ella-world.apk", get(download_apk_handler))
        .route(
            "/ella-world/download/ella-world.apk",
            get(download_apk_handler),
        )
        .nest("/api/ella-world", api_routes.clone())
        .nest("/api", api_routes)
        .nest_service("/ella-world", static_service.clone())
        .fallback_service(static_service)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn download_apk_handler(method: axum::http::Method, State(st): State<AppState>) -> Response {
    let apk_path = st
        .world
        .config
        .public_dir
        .join("download")
        .join("ella-world.apk");
    let Ok(meta) = tokio::fs::metadata(&apk_path).await else {
        return json_err(
            StatusCode::NOT_FOUND,
            "ella-world.apk has not been published to public/download/ yet.",
        );
    };
    let len_str = meta.len().to_string();
    let headers = [
        (
            header::CONTENT_TYPE,
            "application/vnd.android.package-archive",
        ),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"ella-world.apk\"",
        ),
        (header::CACHE_CONTROL, "no-cache"),
        (header::CONTENT_LENGTH, len_str.as_str()),
    ];
    if method == axum::http::Method::HEAD {
        return (StatusCode::OK, headers, Vec::<u8>::new()).into_response();
    }
    match tokio::fs::read(&apk_path).await {
        Ok(bytes) => (StatusCode::OK, headers, bytes).into_response(),
        Err(_) => json_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read APK file.",
        ),
    }
}

fn json_err(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "ok": false, "error": msg }))).into_response()
}

async fn extract_player_world_and_claims(
    st: &AppState,
    token: &str,
) -> Option<(String, Arc<WorldEngine>, String, String)> {
    let (world_slug, world) = st.registry.resolve_world_for_token(token).await?;
    let (self_id, username, _, _, _, _) = world.verify_active_player_token(token).await?;
    Some((world_slug, world, self_id, username))
}

async fn resolve_admin_world(
    st: &AppState,
    headers: &HeaderMap,
) -> Option<(String, Arc<WorldEngine>)> {
    let raw = headers.get("authorization").and_then(|v| v.to_str().ok())?;
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    let (world_slug, world) = st.registry.resolve_world_for_token(token).await?;
    if world.verify_active_admin_token(token).await {
        Some((world_slug, world))
    } else {
        None
    }
}

fn verify_superadmin_bearer(st: &AppState, headers: &HeaderMap) -> bool {
    let Some(raw) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    st.registry.verify_superadmin_token(token)
}

async fn enrich_session_doc(st: &AppState, doc: &mut Value, username: &str, world_slug: &str) {
    let my_playgrounds = st.registry.my_playgrounds_for_user(username).await;
    let webrtc_port = st.sfu.udp_port();
    let native_port = st.native_udp.local_addr().port();
    if let Some(obj) = doc.as_object_mut() {
        obj.insert("worldSlug".into(), json!(world_slug));
        obj.insert("inviteOnly".into(), json!(true));
        obj.insert("myPlaygrounds".into(), json!(my_playgrounds));
        if let Some(wm) = obj.get_mut("worldMeta").and_then(Value::as_object_mut) {
            wm.insert("worldSlug".into(), json!(world_slug));
            wm.insert("webrtcUdpPort".into(), json!(webrtc_port));
            wm.insert("nativeUdpPort".into(), json!(native_port));
        }
    }
}

async fn build_admin_dashboard(st: &AppState, world: &WorldEngine) -> Value {
    let cfg = world.users_cfg.read().await;
    let active = world.players.read().await;
    let blocks_count = world.blocks_data.read().await.blocks.len();
    let homes_views = world.homes_data.read().await.public_views();
    let obby_boards = world.obby_leaderboards.read().await.clone();
    let sky_snapshot = world.sky_state.read().await.snapshot(now_ms());
    let invites = st.registry.admin_list_invites(&world.world_slug).await;

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
                "inventory": u.inventory,
                "cloneableItems": u.cloneable_items,
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
                "companions": u.companions_snapshot(),
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
        "worldSlug": world.world_slug,
        "worldName": cfg.world_name,
        "inviteOnlyRegistration": true,
        "invites": invites,
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
        "itemCatalog": ITEM_CATALOG,
        "petCatalog": PET_CATALOG,
        "petEggShopPrice": PET_EGG_SHOP_PRICE,
        "mountCatalog": MOUNT_CATALOG,
        "craftingRecipes": CRAFTING_RECIPES,
        "homes": homes_views,
        "obbyLeaderboards": obby_boards,
        "sky": sky_snapshot,
        "users": users_list
    })
}

async fn health_handler(State(st): State<AppState>) -> Json<Value> {
    let active_players = st.registry.total_active_players().await;
    let playground_count = st.registry.all_worlds().await.len();
    let blocks_version = st.world.blocks_data.read().await.version;
    Json(json!({
        "status": "ok",
        "engine": "ella-world-rust-udp",
        "version": env!("CARGO_PKG_VERSION"),
        "inviteOnlyRegistration": true,
        "multiPlaygroundEnabled": true,
        "playgroundCount": playground_count,
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

#[derive(Deserialize, Default)]
struct OptionalWorldQuery {
    world: Option<String>,
}

async fn resolve_optional_world_query(st: &AppState, world_opt: Option<&str>) -> Arc<WorldEngine> {
    if let Some(slug) = world_opt.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(w) = st.registry.resolve_world(slug).await {
            return w;
        }
    }
    Arc::clone(&st.world)
}

async fn meta_handler(
    State(st): State<AppState>,
    Query(q): Query<OptionalWorldQuery>,
) -> Json<Value> {
    let target_world = resolve_optional_world_query(&st, q.world.as_deref()).await;
    target_world.prune_stale_players().await;
    let cfg = target_world.users_cfg.read().await;
    let online_count = target_world.players.read().await.len();
    let runway = target_world.runway.read().await.summary();
    let playgrounds = st.registry.list_public_playgrounds().await;
    Json(json!({
        "worldSlug": target_world.world_slug,
        "worldName": cfg.world_name,
        "inviteOnlyRegistration": true,
        "playgrounds": playgrounds,
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
        "androidDownloadUrl": "/download/ella-world.apk",
        "sseEnabled": false,
        "serverTimeMs": now_ms(),
        "webrtcUdpPort": st.sfu.udp_port(),
        "nativeUdpPort": st.native_udp.local_addr().port()
    }))
}

async fn uninvited_register_guard_handler() -> Response {
    json_err(
        StatusCode::FORBIDDEN,
        "Ella's World is strictly invite-only! Redeem an active STAR-... Friend Invite or FAMILY-... Parent Founder Invite code to join.",
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginPayload {
    username: String,
    password: String,
    #[serde(default)]
    world_slug: Option<String>,
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

#[allow(clippy::too_many_arguments)]
async fn finalize_world_player_login(
    st: &AppState,
    world_slug: &str,
    world: &Arc<WorldEngine>,
    user: &UserAccount,
    client_id: Option<&str>,
    machine_seal: Option<&str>,
    claim_device_id: Option<&str>,
    device_telemetry: Option<&ClientDeviceTelemetry>,
    headers: &HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
) -> Response {
    let cookie_seal = extract_machine_seal_cookie(headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let peer_ip = extract_effective_client_ip(headers, peer_ext.map(|e| e.0 .0));

    let resolved_machine = match world
        .resolve_and_authorize_machine_login(
            &user.username,
            cookie_seal.as_deref(),
            machine_seal,
            claim_device_id,
            device_telemetry,
            user_agent,
            peer_ip,
        )
        .await
    {
        Ok(m) => m,
        Err(lock_reason) => return json_err(StatusCode::FORBIDDEN, &lock_reason),
    };

    if let Err(reason) = world.check_world_access_for_user(user).await {
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

    let mut session_doc = world
        .create_or_resume_session(user, client_id, resolved_machine.as_ref())
        .await;
    enrich_session_doc(st, &mut session_doc, &user.username, world_slug).await;
    let resp = Json(session_doc).into_response();
    if let Some(ref m) = resolved_machine {
        attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled)
    } else {
        resp
    }
}

async fn login_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<LoginPayload>,
) -> Response {
    let (world_slug, world, user) = match st
        .registry
        .resolve_login_world_and_user(&body.username, &body.password, body.world_slug.as_deref())
        .await
    {
        Ok(res) => res,
        Err((status, msg)) => return json_err(status, &msg),
    };

    finalize_world_player_login(
        &st,
        &world_slug,
        &world,
        &user,
        body.client_id.as_deref(),
        body.machine_seal.as_deref(),
        None,
        body.device_telemetry.as_ref(),
        &headers,
        peer_ext,
    )
    .await
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
    let Some((world_slug, world)) = st.registry.resolve_world_for_token(&body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Your session has expired or your password was updated. Please sign in again.",
        );
    };
    let Some((_old_id, _username, user, _, _, claim_device_id)) =
        world.verify_player_token_claims_only(&body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Your session has expired or your password was updated. Please sign in again.",
        );
    };

    finalize_world_player_login(
        &st,
        &world_slug,
        &world,
        &user,
        body.client_id.as_deref(),
        body.machine_seal.as_deref(),
        claim_device_id.as_deref(),
        body.device_telemetry.as_ref(),
        &headers,
        peer_ext,
    )
    .await
}

#[derive(Deserialize)]
struct TokenOnlyPayload {
    token: String,
}

async fn logout_handler(
    State(st): State<AppState>,
    Json(body): Json<TokenOnlyPayload>,
) -> Response {
    if let Some((world_slug, world, self_id, _)) =
        extract_player_world_and_claims(&st, &body.token).await
    {
        st.sfu.cleanup_session(&world_slug, &self_id).await;
        st.native_udp
            .evict_sessions_for_peer(&world_slug, &self_id)
            .await;
        world
            .kick_and_remove_peer(&self_id, "Signed out of ELLA World.")
            .await;
    } else if let Some((world_slug, world)) =
        st.registry.resolve_world_for_token(body.token.trim()).await
    {
        if let Some(claims) = st.registry.verify_token_claims(body.token.trim()) {
            if let Some(self_id) = claims.self_id {
                st.sfu.cleanup_session(&world_slug, &self_id).await;
                st.native_udp
                    .evict_sessions_for_peer(&world_slug, &self_id)
                    .await;
                world.remove_player_immediately(&self_id).await;
            }
        }
    }
    Json(json!({ "ok": true })).into_response()
}

// ============================================================================
// Multi-Playground Switcher & Two-Tier Invite-Only Endpoints
// ============================================================================

async fn playgrounds_public_handler(State(st): State<AppState>) -> Json<Value> {
    let playgrounds = st.registry.list_public_playgrounds().await;
    Json(json!({
        "ok": true,
        "inviteOnlyRegistration": true,
        "playgrounds": playgrounds,
    }))
}

async fn playgrounds_mine_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some(raw) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return json_err(StatusCode::UNAUTHORIZED, "Missing Bearer session token.");
    };
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    let Some((world_slug, _world, _self_id, username)) =
        extract_player_world_and_claims(&st, token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let my_playgrounds = st.registry.my_playgrounds_for_user(&username).await;
    Json(json!({
        "ok": true,
        "currentWorldSlug": world_slug,
        "myPlaygrounds": my_playgrounds,
    }))
    .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwitchPlaygroundPayload {
    token: String,
    world_slug: String,
    client_id: Option<String>,
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

async fn playgrounds_switch_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<SwitchPlaygroundPayload>,
) -> Response {
    let Some((old_slug, old_world)) = st.registry.resolve_world_for_token(&body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let Some((old_self_id, username, _old_user, _, _, claim_device_id)) =
        old_world.verify_player_token_claims_only(&body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    let (target_slug, target_world, target_user) = match st
        .registry
        .prepare_world_switch(&username, &body.world_slug)
        .await
    {
        Ok(res) => res,
        Err((status, msg)) => return json_err(status, &msg),
    };

    if old_slug != target_slug {
        st.sfu.cleanup_session(&old_slug, &old_self_id).await;
        st.native_udp
            .evict_sessions_for_peer(&old_slug, &old_self_id)
            .await;
        old_world.remove_player_immediately(&old_self_id).await;
    }

    finalize_world_player_login(
        &st,
        &target_slug,
        &target_world,
        &target_user,
        body.client_id.as_deref(),
        body.machine_seal.as_deref(),
        claim_device_id.as_deref(),
        body.device_telemetry.as_ref(),
        &headers,
        peer_ext,
    )
    .await
}

#[derive(Deserialize)]
struct InspectInviteQuery {
    code: String,
}

async fn invites_inspect_handler(
    State(st): State<AppState>,
    Query(q): Query<InspectInviteQuery>,
) -> Response {
    match st.registry.inspect_invite(&q.code).await {
        Ok(val) => Json(val).into_response(),
        Err((status, msg)) => json_err(status, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedeemFriendHttpPayload {
    code: String,
    username: String,
    #[serde(default)]
    display_name: String,
    password: String,
    #[serde(default)]
    avatar: Option<Value>,
    client_id: Option<String>,
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

async fn invites_redeem_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<RedeemFriendHttpPayload>,
) -> Response {
    let (world_slug, world, user, ticket) = match st
        .registry
        .redeem_friend_invite(RedeemFriendInviteInput {
            code: body.code,
            username: body.username,
            display_name: body.display_name,
            password: body.password,
            avatar: body.avatar,
        })
        .await
    {
        Ok(res) => res,
        Err((status, msg)) => return json_err(status, &msg),
    };

    let mut resp = finalize_world_player_login(
        &st,
        &world_slug,
        &world,
        &user,
        body.client_id.as_deref(),
        body.machine_seal.as_deref(),
        None,
        body.device_telemetry.as_ref(),
        &headers,
        peer_ext,
    )
    .await;
    if let Ok(hv) = HeaderValue::from_str(&ticket.code) {
        resp.headers_mut().insert("x-ella-redeemed-invite", hv);
    }
    resp
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinWorldWithInvitePayload {
    token: String,
    code: String,
    client_id: Option<String>,
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

async fn invites_join_world_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<JoinWorldWithInvitePayload>,
) -> Response {
    let Some((old_slug, old_world)) = st.registry.resolve_world_for_token(&body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let Some((old_self_id, username, _, _, _, claim_device_id)) =
        old_world.verify_player_token_claims_only(&body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    let (target_slug, target_world, target_user, _ticket) = match st
        .registry
        .join_world_with_friend_invite(&username, &body.code)
        .await
    {
        Ok(res) => res,
        Err((status, msg)) => return json_err(status, &msg),
    };

    if old_slug != target_slug {
        st.sfu.cleanup_session(&old_slug, &old_self_id).await;
        st.native_udp
            .evict_sessions_for_peer(&old_slug, &old_self_id)
            .await;
        old_world.remove_player_immediately(&old_self_id).await;
    }

    finalize_world_player_login(
        &st,
        &target_slug,
        &target_world,
        &target_user,
        body.client_id.as_deref(),
        body.machine_seal.as_deref(),
        claim_device_id.as_deref(),
        body.device_telemetry.as_ref(),
        &headers,
        peer_ext,
    )
    .await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedeemFamilyHttpPayload {
    code: String,
    world_name: String,
    #[serde(default)]
    world_slug: Option<String>,
    admin_password: String,
    host_username: String,
    #[serde(default)]
    host_display_name: String,
    host_password: String,
    #[serde(default)]
    avatar: Option<Value>,
    client_id: Option<String>,
    machine_seal: Option<String>,
    device_telemetry: Option<ClientDeviceTelemetry>,
}

async fn invites_redeem_family_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    peer_ext: Option<Extension<ClientPeerAddr>>,
    Json(body): Json<RedeemFamilyHttpPayload>,
) -> Response {
    let (slug, new_world, host_user, _ticket) = match st
        .registry
        .redeem_family_founder_invite(RedeemFamilyInviteInput {
            code: body.code,
            world_name: body.world_name,
            world_slug: body.world_slug,
            admin_password: body.admin_password,
            host_username: body.host_username,
            host_display_name: body.host_display_name,
            host_password: body.host_password,
            avatar: body.avatar,
        })
        .await
    {
        Ok(res) => res,
        Err((status, msg)) => return json_err(status, &msg),
    };

    // Dynamically attach the new Playground onto both single-port UDP engines without restart!
    st.sfu
        .attach_world(slug.clone(), Arc::clone(&new_world))
        .await;
    st.native_udp
        .attach_world(slug.clone(), Arc::clone(&new_world))
        .await;

    let admin_token = new_world.mint_admin_token().await;
    let cookie_seal = extract_machine_seal_cookie(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let peer_ip = extract_effective_client_ip(&headers, peer_ext.map(|e| e.0 .0));
    let resolved_machine = match new_world
        .resolve_and_authorize_machine_login(
            &host_user.username,
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

    let mut session_doc = new_world
        .create_or_resume_session(
            &host_user,
            body.client_id.as_deref(),
            resolved_machine.as_ref(),
        )
        .await;
    enrich_session_doc(&st, &mut session_doc, &host_user.username, &slug).await;
    if let Some(obj) = session_doc.as_object_mut() {
        obj.insert("ok".into(), json!(true));
        obj.insert("adminToken".into(), json!(admin_token));
        obj.insert("createdPlaygroundSlug".into(), json!(slug));
    }

    let resp = Json(session_doc).into_response();
    if let Some(ref m) = resolved_machine {
        attach_machine_seal_cookie(resp, &m.seal_token, st.tls_enabled)
    } else {
        resp
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostDraftInvitePayload {
    token: String,
    #[serde(default)]
    label_note: Option<String>,
}

async fn invites_host_draft_handler(
    State(st): State<AppState>,
    Json(body): Json<HostDraftInvitePayload>,
) -> Response {
    let Some((world_slug, _world, _self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match st
        .registry
        .host_draft_friend_invite(&world_slug, &username, body.label_note.as_deref())
        .await
    {
        Ok(ticket) => Json(json!({
            "ok": true,
            "worldSlug": world_slug,
            "invite": ticket,
            "message": "Friend invite drafted! Ask your parent to click Approve in the Parent Admin Panel so your friend can use it."
        }))
        .into_response(),
        Err((status, msg)) => json_err(status, &msg),
    }
}

// ============================================================================
// Platform Super-Admin & Android APK Metadata Endpoints
// ============================================================================

#[derive(Deserialize)]
struct SuperAdminLoginPayload {
    password: String,
}

async fn superadmin_login_handler(
    State(st): State<AppState>,
    Json(body): Json<SuperAdminLoginPayload>,
) -> Response {
    if !st.registry.verify_superadmin_password(&body.password) {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Incorrect Platform Super-Admin password.",
        );
    }
    let token = st.registry.mint_superadmin_token();
    let state_doc = st.registry.superadmin_state().await;
    Json(json!({
        "ok": true,
        "superadminToken": token,
        "platform": state_doc,
    }))
    .into_response()
}

async fn superadmin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    if !verify_superadmin_bearer(&st, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized superadmin token.");
    }
    Json(st.registry.superadmin_state().await).into_response()
}

async fn superadmin_invites_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SuperAdminInviteActionInput>,
) -> Response {
    if !verify_superadmin_bearer(&st, &headers) {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized superadmin token.");
    }
    match st.registry.superadmin_manage_invite(body).await {
        Ok(ticket) => {
            let platform = st.registry.superadmin_state().await;
            Json(json!({
                "ok": true,
                "invite": ticket,
                "platform": platform,
            }))
            .into_response()
        }
        Err((status, msg)) => json_err(status, &msg),
    }
}

async fn android_release_handler(State(st): State<AppState>) -> Json<Value> {
    let dl_dir = st.world.config.public_dir.join("download");
    let apk_path = dl_dir.join("ella-world.apk");
    let meta_path = dl_dir.join("ella-world-release.json");

    if let Ok(raw_meta) = tokio::fs::read_to_string(&meta_path).await {
        if let Ok(mut parsed) = serde_json::from_str::<Value>(&raw_meta) {
            let apk_exists = tokio::fs::metadata(&apk_path).await.is_ok();
            if let Some(obj) = parsed.as_object_mut() {
                obj.insert("ok".into(), json!(true));
                obj.insert("available".into(), json!(apk_exists));
                obj.entry("downloadUrl")
                    .or_insert_with(|| json!("/download/ella-world.apk"));
            }
            return Json(parsed);
        }
    }

    if let Ok(bytes) = tokio::fs::read(&apk_path).await {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest = hasher.finalize();
        let mut sha_hex = String::with_capacity(64);
        for b in digest {
            use std::fmt::Write;
            let _ = write!(&mut sha_hex, "{b:02x}");
        }
        return Json(json!({
            "ok": true,
            "available": true,
            "applicationId": "com.robertscreations.ellaworld",
            "versionName": env!("CARGO_PKG_VERSION"),
            "versionCode": 4,
            "minSdk": 26,
            "targetSdk": 34,
            "apkBytes": bytes.len(),
            "sha256": sha_hex,
            "aabConfigured": true,
            "downloadUrl": "/download/ella-world.apk"
        }));
    }

    Json(json!({
        "ok": true,
        "available": false,
        "applicationId": "com.robertscreations.ellaworld",
        "versionName": env!("CARGO_PKG_VERSION"),
        "versionCode": 4,
        "minSdk": 26,
        "targetSdk": 34,
        "aabConfigured": true,
        "downloadUrl": "/download/ella-world.apk"
    }))
}

// ============================================================================
// Core Gameplay & Per-Playground Admin Handlers
// ============================================================================

#[derive(Deserialize)]
struct RtcOfferPayload {
    token: String,
    sdp: String,
}

async fn rtc_offer_handler(
    State(st): State<AppState>,
    Json(body): Json<RtcOfferPayload>,
) -> Response {
    let Some((world_slug, world)) = st.registry.resolve_world_for_token(&body.token).await else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token for UDP WebRTC.",
        );
    };
    let Some((self_id, username, _, _, _, device_id)) =
        world.verify_active_player_token(&body.token).await
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
        .accept_browser_offer(world_slug, world, self_id, username, device_id, body.sdp)
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match world
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
                let cfg = world.users_cfg.read().await;
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((world_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match world.update_avatar(&self_id, &username, &body.avatar).await {
        Some(sanitized) => {
            // Synchronously mirror global avatar across all Playgrounds before returning HTTP 200
            st.registry
                .sync_avatar_across_worlds(&username, &sanitized.avatar, Some(&world_slug))
                .await;
            Json(json!({
                "ok": true,
                "user": sanitized
            }))
            .into_response()
        }
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
    #[serde(default)]
    trophy_tier: Option<String>,
}

async fn chat_handler(State(st): State<AppState>, Json(body): Json<ChatPostPayload>) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match world
        .post_chat(
            &self_id,
            &username,
            body.text.as_deref(),
            body.is_trophy_win,
            body.trophy_tier.as_deref(),
        )
        .await
    {
        Ok((message, coins_awarded)) => Json(json!({
            "ok": true,
            "message": message,
            "coinsAwarded": coins_awarded
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };

    match world
        .mutate_runway(
            &self_id,
            &username,
            &body.action,
            body.theme.as_deref(),
            body.target_username.as_deref(),
            body.stars,
        )
        .await
    {
        Ok(runway) => Json(json!({
            "ok": true,
            "runway": runway
        }))
        .into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminLoginPayload {
    password: String,
    #[serde(default)]
    world_slug: Option<String>,
}

async fn admin_login_handler(
    State(st): State<AppState>,
    Json(body): Json<AdminLoginPayload>,
) -> Response {
    let target_world = if let Some(req_slug) = body
        .world_slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let Some(w) = st.registry.resolve_world(req_slug).await else {
            return json_err(
                StatusCode::NOT_FOUND,
                &format!("Playground '{req_slug}' not found."),
            );
        };
        let valid = {
            let cfg = w.users_cfg.read().await;
            constant_time_str_eq(&cfg.admin_password, &body.password)
        };
        if !valid {
            return json_err(
                StatusCode::UNAUTHORIZED,
                "Incorrect Parent Master Password for that Playground.",
            );
        }
        w
    } else {
        let ella_valid = {
            let cfg = st.world.users_cfg.read().await;
            constant_time_str_eq(&cfg.admin_password, &body.password)
        };
        if ella_valid {
            Arc::clone(&st.world)
        } else {
            let mut matched_other: Vec<Arc<WorldEngine>> = Vec::new();
            for (slug, w) in st.registry.all_worlds().await {
                if slug == DEFAULT_WORLD_SLUG {
                    continue;
                }
                let ok = {
                    let cfg = w.users_cfg.read().await;
                    constant_time_str_eq(&cfg.admin_password, &body.password)
                };
                if ok {
                    matched_other.push(w);
                }
            }
            if matched_other.len() == 1 {
                matched_other.remove(0)
            } else {
                return json_err(
                    StatusCode::UNAUTHORIZED,
                    "Incorrect Parent Master Password.",
                );
            }
        }
    };

    let admin_token = target_world.mint_admin_token().await;
    let dashboard = build_admin_dashboard(&st, &target_world).await;

    Json(json!({
        "ok": true,
        "worldSlug": target_world.world_slug,
        "adminToken": admin_token,
        "dashboard": dashboard
    }))
    .into_response()
}

async fn admin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };
    let dashboard = build_admin_dashboard(&st, &world).await;
    Json(json!({ "ok": true, "worldSlug": world_slug, "dashboard": dashboard })).into_response()
}

async fn admin_invites_get_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some((world_slug, _world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };
    let invites = st.registry.admin_list_invites(&world_slug).await;
    Json(json!({
        "ok": true,
        "worldSlug": world_slug,
        "invites": invites,
    }))
    .into_response()
}

async fn admin_invites_post_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminInviteActionInput>,
) -> Response {
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };
    match st.registry.admin_manage_invite(&world_slug, body).await {
        Ok(invite) => {
            let dashboard = build_admin_dashboard(&st, &world).await;
            Json(json!({
                "ok": true,
                "worldSlug": world_slug,
                "invite": invite,
                "dashboard": dashboard,
            }))
            .into_response()
        }
        Err((status, msg)) => json_err(status, &msg),
    }
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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

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
    let final_avatar: AvatarAppearance;
    {
        let mut cfg = world.users_cfg.write().await;
        if let Some(existing) = cfg
            .users
            .iter_mut()
            .find(|u| u.username.eq_ignore_ascii_case(&clean_username))
        {
            if !constant_time_str_eq(&existing.password, &clean_pw) {
                password_rotated = true;
                existing.token_epoch = existing.token_epoch.wrapping_add(1);
            }
            existing.display_name = display_name.clone();
            existing.password = clean_pw.clone();
            if let Some(r) = &body.role {
                existing.role = r.clone();
            }
            if let Some(av_patch) = &body.avatar {
                existing.avatar.apply_patch(av_patch);
            }
            final_avatar = existing.avatar.clone();
        } else {
            let mut av = AvatarAppearance::default();
            if let Some(av_patch) = &body.avatar {
                av.apply_patch(av_patch);
            }
            final_avatar = av.clone();
            let mut new_user = UserAccount::new_basic(
                clean_username.clone(),
                display_name.clone(),
                clean_pw.clone(),
                body.role.unwrap_or_else(|| "friend".into()),
            );
            new_user.avatar = av;
            cfg.users.push(new_user);
        }
    }
    world.persist_users_async().await;
    st.registry
        .sync_admin_upserted_user(
            &world_slug,
            &clean_username,
            &display_name,
            &clean_pw,
            &final_avatar,
        )
        .await;

    let mut kicked_sessions = 0usize;
    if password_rotated {
        st.sfu
            .cleanup_sessions_for_username(&world_slug, &clean_username)
            .await;
        st.native_udp
            .evict_sessions_for_username(&world_slug, &clean_username)
            .await;
        let kicked = world
            .kick_and_remove_user_sessions(
                &clean_username,
                "Your password was updated by a parent. Please sign in with your new password.",
            )
            .await;
        kicked_sessions = kicked.len();
    }

    let dashboard = build_admin_dashboard(&st, &world).await;
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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let target = username.trim().to_lowercase();
    {
        let mut cfg = world.users_cfg.write().await;
        cfg.users
            .retain(|u| !u.username.eq_ignore_ascii_case(&target));
    }
    world.persist_users_async().await;
    st.registry
        .sync_admin_deleted_user(&world_slug, &target)
        .await;

    st.sfu
        .cleanup_sessions_for_username(&world_slug, &target)
        .await;
    st.native_udp
        .evict_sessions_for_username(&world_slug, &target)
        .await;
    let kicked = world
        .kick_and_remove_user_sessions(&target, "Your account was removed by a parent.")
        .await;

    let dashboard = build_admin_dashboard(&st, &world).await;
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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let target_user = body
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase);

    if let Some(uname) = target_user {
        let maybe_epoch = {
            let mut cfg = world.users_cfg.write().await;
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
        world.persist_users_async().await;

        let reason = body
            .reason
            .as_deref()
            .unwrap_or("Your session was signed out by a parent. Please log in again.");
        st.sfu
            .cleanup_sessions_for_username(&world_slug, &uname)
            .await;
        st.native_udp
            .evict_sessions_for_username(&world_slug, &uname)
            .await;
        let kicked = world.kick_and_remove_user_sessions(&uname, reason).await;
        let dashboard = build_admin_dashboard(&st, &world).await;

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
            let mut cfg = world.users_cfg.write().await;
            cfg.session_epoch = cfg.session_epoch.wrapping_add(1).max(1);
            for u in &mut cfg.users {
                u.token_epoch = u.token_epoch.wrapping_add(1);
            }
            cfg.session_epoch
        };
        world.persist_users_async().await;

        let reason = body
            .reason
            .as_deref()
            .unwrap_or("All active sessions were expired by a parent. Please sign in again.");
        st.sfu.cleanup_all_sessions(&world_slug).await;
        st.native_udp.evict_all_sessions(&world_slug).await;
        let kicked = world.kick_and_remove_all_sessions(reason).await;
        let admin_token = world.mint_admin_token().await;
        let dashboard = build_admin_dashboard(&st, &world).await;

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
    let Some((_world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let limit = query.limit.unwrap_or(200).clamp(1, 1000);
    let user_filter = query.user.as_deref();
    let search_filter = query.search.as_deref();
    let (total, chats) = world.store.query_chats(limit, user_filter, search_filter);
    let (blocked_total, blocked_chats) = if query.include_blocked {
        world
            .store
            .query_blocked_chats(limit, user_filter, search_filter)
    } else {
        (world.store.count_blocked_chats(), Vec::new())
    };
    let cfg = world.users_cfg.read().await;

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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let mut admin_pw_changed = false;
    let mut should_evict_all = false;
    let mut penalized_users_to_sweep = Vec::new();
    {
        let mut cfg = world.users_cfg.write().await;
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
    world.persist_users_async().await;
    let _ = st.registry.sync_all_worlds_into_registry().await;
    world.broadcast_parental_world_settings().await;
    if should_evict_all {
        st.sfu.cleanup_all_sessions(&world_slug).await;
        st.native_udp.evict_all_sessions(&world_slug).await;
    }
    for pu in penalized_users_to_sweep {
        let evicted_pids = world.evict_other_accounts_on_user_machines(&pu).await;
        for pid in evicted_pids {
            st.sfu.cleanup_session(&world_slug, &pid).await;
            st.native_udp
                .evict_sessions_for_peer(&world_slug, &pid)
                .await;
        }
    }
    world.enforce_live_world_access().await;

    let dashboard = build_admin_dashboard(&st, &world).await;
    if admin_pw_changed {
        let admin_token = world.mint_admin_token().await;
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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let target_uname = body.username.trim().to_lowercase();
    let mut res = match world.admin_moderate_user(body).await {
        Ok(v) => v,
        Err(msg) => return json_err(StatusCode::BAD_REQUEST, &msg),
    };

    if let Some(arr) = res.get("evictedAltPeers").and_then(Value::as_array) {
        for pid_val in arr {
            if let Some(pid) = pid_val.as_str() {
                st.sfu.cleanup_session(&world_slug, pid).await;
                st.native_udp
                    .evict_sessions_for_peer(&world_slug, pid)
                    .await;
            }
        }
    }

    if res.get("suspended").and_then(Value::as_bool) == Some(true) {
        st.sfu
            .cleanup_sessions_for_username(&world_slug, &target_uname)
            .await;
        st.native_udp
            .evict_sessions_for_username(&world_slug, &target_uname)
            .await;
        let kicked = world
            .kick_and_remove_user_sessions(&target_uname, "🚫 Your account was paused by a parent.")
            .await;
        if let Some(obj) = res.as_object_mut() {
            obj.insert("kickedCount".into(), json!(kicked.len()));
        }
    }

    let dashboard = build_admin_dashboard(&st, &world).await;
    if let Some(obj) = res.as_object_mut() {
        obj.insert("dashboard".into(), dashboard);
    }
    Json(res).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminGiftItemPayload {
    username: String,
    item_id: String,
    #[serde(default)]
    qty: u32,
    set_qty: Option<u32>,
    allow_clone: Option<bool>,
}

async fn admin_gift_item_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminGiftItemPayload>,
) -> Response {
    let Some((_world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };
    let mut res = match world
        .admin_gift_item(
            &body.username,
            &body.item_id,
            body.qty,
            body.set_qty,
            body.allow_clone,
        )
        .await
    {
        Ok(v) => v,
        Err(msg) => return json_err(StatusCode::BAD_REQUEST, &msg),
    };
    let dashboard = build_admin_dashboard(&st, &world).await;
    if let Some(obj) = res.as_object_mut() {
        obj.insert("dashboard".into(), dashboard);
    }
    Json(res).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EconomyClonePayload {
    token: String,
    item_id: String,
}

async fn economy_clone_handler(
    State(st): State<AppState>,
    Json(body): Json<EconomyClonePayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
        .clone_inventory_item(&self_id, &username, &body.item_id)
        .await
    {
        Ok(v) => Json(v).into_response(),
        Err((code, msg)) => json_err(
            StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST),
            &msg,
        ),
    }
}

async fn extract_bearer_player_world_and_username(
    st: &AppState,
    headers: &HeaderMap,
) -> Option<(String, Arc<WorldEngine>, String)> {
    let raw = headers.get("authorization").and_then(|v| v.to_str().ok())?;
    let token = raw.strip_prefix("Bearer ").unwrap_or(raw).trim();
    let (slug, world, _, username) = extract_player_world_and_claims(st, token).await?;
    Some((slug, world, username))
}

async fn subadmin_state_handler(State(st): State<AppState>, headers: HeaderMap) -> Response {
    let Some((_slug, world, actor_username)) =
        extract_bearer_player_world_and_username(&st, &headers).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world.build_subadmin_friends_state(&actor_username).await {
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
    let Some((world_slug, world, actor_username)) =
        extract_bearer_player_world_and_username(&st, &headers).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let mut res = match world.subadmin_moderate_friend(&actor_username, body).await {
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
                    st.sfu.cleanup_session(&world_slug, pid).await;
                    st.native_udp
                        .evict_sessions_for_peer(&world_slug, pid)
                        .await;
                }
            }
        }
    }
    if let Ok(state_val) = world.build_subadmin_friends_state(&actor_username).await {
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
    let Some((world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    let (updated, kicked_peers) = match world.admin_manage_device(body).await {
        Ok(res) => res,
        Err(msg) => return json_err(StatusCode::BAD_REQUEST, &msg),
    };
    for pid in &kicked_peers {
        st.sfu.cleanup_session(&world_slug, pid).await;
        st.native_udp
            .evict_sessions_for_peer(&world_slug, pid)
            .await;
    }

    let dashboard = build_admin_dashboard(&st, &world).await;
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
    let Some((_world_slug, world)) = resolve_admin_world(&st, &headers).await else {
        return json_err(StatusCode::UNAUTHORIZED, "Unauthorized admin token.");
    };

    world.reset_world_blocks(body.mode == "clear").await;
    let dashboard = build_admin_dashboard(&st, &world).await;
    Json(json!({ "ok": true, "dashboard": dashboard })).into_response()
}

// ============================================================================
// Crystal Cavern Dungeon, Persistent Economy & Live P2P Trade HTTP Endpoints
// ============================================================================

async fn economy_catalog_handler(
    State(st): State<AppState>,
    Query(q): Query<OptionalWorldQuery>,
) -> Json<Value> {
    let w = resolve_optional_world_query(&st, q.world.as_deref()).await;
    let dungeon = w.dungeon.read().await.snapshot();
    Json(json!({
        "ok": true,
        "itemCatalog": ITEM_CATALOG,
        "items": ITEM_CATALOG,
        "petCatalog": PET_CATALOG,
        "petEggShopPrice": PET_EGG_SHOP_PRICE,
        "mountCatalog": MOUNT_CATALOG,
        "craftingRecipes": CRAFTING_RECIPES,
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    let Some(target_zone) = body.zone.or(body.target_zone) else {
        return json_err(StatusCode::BAD_REQUEST, "Target zone is required.");
    };
    match world
        .transition_player_zone(&self_id, &username, target_zone)
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArenaRespawnPayload {
    token: String,
}

async fn arena_respawn_handler(
    State(st): State<AppState>,
    Json(body): Json<ArenaRespawnPayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world.respawn_in_arena(&self_id, &username).await {
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, _username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, _username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, _username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
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
    let Some((_slug, world, self_id, _username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world.trade_confirm(&self_id, body.trade_id.trim()).await {
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
    let Some((_slug, world, self_id, _username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world.trade_cancel(&self_id, body.trade_id.as_deref()).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

// ============================================================================
// Expansion Pack HTTP Endpoints (Pets/Boards, Homes, Craft, Obby, Sky, Boss)
// ============================================================================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompanionsPayload {
    token: String,
    action: String,
    #[serde(default)]
    pet_id: Option<String>,
    #[serde(default)]
    mount_id: Option<String>,
    #[serde(default)]
    nickname: Option<String>,
}

async fn companions_handler(
    State(st): State<AppState>,
    Json(body): Json<CompanionsPayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
        .mutate_companions(
            &self_id,
            &username,
            &body.action,
            body.pet_id.as_deref(),
            body.mount_id.as_deref(),
            body.nickname.as_deref(),
        )
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

async fn homes_get_handler(
    State(st): State<AppState>,
    Query(q): Query<OptionalWorldQuery>,
) -> Json<Value> {
    let w = resolve_optional_world_query(&st, q.world.as_deref()).await;
    let homes = w.homes_data.read().await.public_views();
    Json(json!({
        "ok": true,
        "worldSlug": w.world_slug,
        "homes": homes,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HomesMutationPayload {
    #[serde(default)]
    token: Option<String>,
    action: String,
    #[serde(default)]
    plot_id: Option<String>,
    #[serde(default)]
    custom_title: Option<String>,
    #[serde(default)]
    door_locked: Option<bool>,
    #[serde(default)]
    co_builders: Option<Vec<String>>,
}

async fn homes_post_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<HomesMutationPayload>,
) -> Response {
    let (world, is_admin, self_id, username) = if let Some((_slug, admin_w)) =
        resolve_admin_world(&st, &headers).await
    {
        (admin_w, true, "admin".to_string(), "admin".to_string())
    } else {
        let Some(tok) = body.token.as_deref() else {
            return json_err(StatusCode::UNAUTHORIZED, "Missing player or admin token.");
        };
        let Some((_slug, player_w, sid, uname)) = extract_player_world_and_claims(&st, tok).await
        else {
            return json_err(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired session token.",
            );
        };
        (player_w, false, sid, uname)
    };

    match world
        .mutate_home(
            &self_id,
            &username,
            is_admin,
            &body.action,
            body.plot_id.as_deref(),
            body.custom_title.as_deref(),
            body.door_locked,
            body.co_builders,
        )
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

async fn craft_get_handler() -> Json<Value> {
    Json(json!({
        "ok": true,
        "craftingRecipes": CRAFTING_RECIPES,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CraftMutationPayload {
    token: String,
    recipe_id: String,
}

async fn craft_post_handler(
    State(st): State<AppState>,
    Json(body): Json<CraftMutationPayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
        .craft_recipe(&self_id, &username, &body.recipe_id)
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

async fn obby_get_handler(
    State(st): State<AppState>,
    Query(q): Query<OptionalWorldQuery>,
) -> Json<Value> {
    let w = resolve_optional_world_query(&st, q.world.as_deref()).await;
    let leaderboards = w.obby_leaderboards.read().await.clone();
    Json(json!({
        "ok": true,
        "worldSlug": w.world_slug,
        "obbyLeaderboards": leaderboards,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObbySpeedrunPayload {
    token: String,
    action: String,
    #[serde(default)]
    course: Option<String>,
}

async fn obby_post_handler(
    State(st): State<AppState>,
    Json(body): Json<ObbySpeedrunPayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
        .obby_speedrun_action(&self_id, &username, &body.action, body.course.as_deref())
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

async fn sky_get_handler(
    State(st): State<AppState>,
    Query(q): Query<OptionalWorldQuery>,
) -> Json<Value> {
    let w = resolve_optional_world_query(&st, q.world.as_deref()).await;
    let sky = w.sky_state.read().await.snapshot(now_ms());
    Json(json!({
        "ok": true,
        "worldSlug": w.world_slug,
        "sky": sky,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkyMutationPayload {
    #[serde(default)]
    token: Option<String>,
    action: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    star_id: Option<String>,
}

async fn sky_post_handler(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SkyMutationPayload>,
) -> Response {
    let (world, is_admin, self_id, username_opt) = if let Some((_slug, admin_w)) =
        resolve_admin_world(&st, &headers).await
    {
        (admin_w, true, "admin".to_string(), None)
    } else {
        let Some(tok) = body.token.as_deref() else {
            return json_err(StatusCode::UNAUTHORIZED, "Missing player or admin token.");
        };
        let Some((_slug, player_w, sid, uname)) = extract_player_world_and_claims(&st, tok).await
        else {
            return json_err(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired session token.",
            );
        };
        (player_w, false, sid, Some(uname))
    };

    match world
        .mutate_sky(
            &self_id,
            username_opt.as_deref(),
            is_admin,
            &body.action,
            body.mode.as_deref(),
            body.star_id.as_deref(),
        )
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArenaBossSummonPayload {
    token: String,
    #[serde(default)]
    use_beacon: bool,
}

async fn arena_boss_handler(
    State(st): State<AppState>,
    Json(body): Json<ArenaBossSummonPayload>,
) -> Response {
    let Some((_slug, world, self_id, username)) =
        extract_player_world_and_claims(&st, &body.token).await
    else {
        return json_err(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired session token.",
        );
    };
    match world
        .summon_arena_boss(&self_id, &username, body.use_beacon)
        .await
    {
        Ok(res) => Json(res).into_response(),
        Err(msg) => json_err(StatusCode::BAD_REQUEST, &msg),
    }
}
