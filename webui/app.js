(function () {
  "use strict";

  const LS_DISCOVERY = "messenger_webui_discovery_url";
  const LS_TIMEOUT = "messenger_webui_default_timeout_ms";

  const POLL_MS = 750;

  const state = {
    apiBaseUrl: "",
    peerId: "",
    streamId: "",
    pollTimer: null,
  };

  function $(id) {
    return document.getElementById(id);
  }

  function loadStored() {
    try {
      const d = localStorage.getItem(LS_DISCOVERY);
      const t = localStorage.getItem(LS_TIMEOUT);
      if (d) $("discovery-url").value = d;
      if (t) $("default-timeout").value = t;
    } catch (_) {
      /* ignore */
    }
  }

  function saveStored() {
    try {
      localStorage.setItem(LS_DISCOVERY, $("discovery-url").value.trim());
      localStorage.setItem(LS_TIMEOUT, $("default-timeout").value);
    } catch (_) {
      /* ignore */
    }
  }

  function normalizeBase(url) {
    const s = String(url || "").trim();
    if (!s) return "";
    return s.replace(/\/+$/, "");
  }

  function resolveApiBaseUrl() {
    return normalizeBase(window.location.origin);
  }

  function defaultTimeoutMs() {
    const n = parseInt($("default-timeout").value, 10);
    return Number.isFinite(n) && n >= 1 ? n : 60000;
  }

  function connTimeoutMs() {
    const v = $("conn-timeout").value.trim();
    if (v) {
      const n = parseInt(v, 10);
      if (Number.isFinite(n) && n >= 1) return n;
    }
    return defaultTimeoutMs();
  }

  function streamTimeoutMs() {
    const v = $("stream-timeout").value.trim();
    if (v) {
      const n = parseInt(v, 10);
      if (Number.isFinite(n) && n >= 1) return n;
    }
    return defaultTimeoutMs();
  }

  const diagEl = () => $("diagnostics");

  function logDiag(message, kind) {
    const line = document.createElement("div");
    if (kind === "err") line.className = "err";
    else if (kind === "ok") line.className = "ok";
    const ts = new Date().toISOString().slice(11, 23);
    line.textContent = "[" + ts + "] " + message;
    diagEl().appendChild(line);
    diagEl().scrollTop = diagEl().scrollHeight;
  }

  function chatLine(text, cls) {
    const line = document.createElement("div");
    line.className = "line " + (cls || "");
    const ts = new Date().toISOString().slice(11, 23);
    line.textContent = "[" + ts + "] " + text;
    $("chat-log").appendChild(line);
    $("chat-log").scrollTop = $("chat-log").scrollHeight;
  }

  function chatSys(text) {
    chatLine(text, "sys");
  }

  async function apiCall(path, method, body) {
    const base = normalizeBase(state.apiBaseUrl) || resolveApiBaseUrl();
    const url = base + path;
    const options = {
      method: method || "GET",
      headers: {},
    };
    const m = (options.method || "GET").toUpperCase();
    if (body !== undefined && m !== "GET" && m !== "HEAD") {
      options.headers["Content-Type"] = "application/json";
      options.body = JSON.stringify(body);
    }
    logDiag(m + " " + path);
    let res;
    try {
      res = await fetch(url, options);
    } catch (e) {
      let msg = "Network: " + (e && e.message ? e.message : String(e));
      if (window.location.protocol === "file:") {
        msg +=
          " — If you opened this page via file://, the browser may block fetch() to http:// before any request is sent; serve webui/ over HTTP (see banner).";
      }
      logDiag(msg, "err");
      throw new Error(msg);
    }
    const ct = res.headers.get("content-type") || "";
    let data = null;
    const text = await res.text();
    if (text && ct.includes("application/json")) {
      try {
        data = JSON.parse(text);
      } catch (e) {
        const msg = "Invalid JSON: " + text.slice(0, 200);
        logDiag(msg, "err");
        throw new Error(msg);
      }
    } else if (text && !ct.includes("application/json")) {
      data = text;
    }
    if (!res.ok) {
      let msg = res.status + " " + res.statusText;
      if (data && typeof data === "object" && data.message) {
        msg = res.status + ": " + data.message;
      } else if (typeof data === "string" && data) {
        msg = res.status + ": " + data.slice(0, 500);
      }
      logDiag(msg, "err");
      throw new Error(msg);
    }
    logDiag(m + " " + path + " → " + res.status, "ok");
    return data;
  }

  function setOnlineUi(online) {
    const dot = $("online-dot");
    $("online-label").textContent = online ? "Online" : "Offline";
    dot.classList.toggle("on", !!online);
    dot.classList.toggle("off", !online);
  }

  async function refreshOnline() {
    try {
      const data = await apiCall("/is_online", "GET");
      setOnlineUi(!!(data && data.online));
    } catch (e) {
      setOnlineUi(false);
    }
  }

  async function goOnline() {
    saveStored();
    const server_url = $("discovery-url").value.trim();
    if (!server_url) {
      logDiag("Discovery server URL required for go_online", "err");
      return;
    }
    await apiCall("/go_online", "POST", { server_url });
    await refreshOnline();
  }

  async function fetchMeId() {
    try {
      const data = await apiCall("/me/id", "GET");
      $("me-id").textContent = data && data.id ? data.id : "—";
    } catch (e) {
      $("me-id").textContent = "—";
    }
  }

  function syncPeerFields() {
    const p = $("peer-id").value.trim();
    state.peerId = p;
    if (!$("stream-peer-id").value.trim() && p) {
      $("stream-peer-id").value = p;
    }
  }

  async function doConnect() {
    syncPeerFields();
    const peer_id = $("peer-id").value.trim();
    if (!peer_id) {
      logDiag("Peer id required", "err");
      return;
    }
    await apiCall("/connect", "POST", {
      peer_id,
      timeout_ms: connTimeoutMs(),
    });
    chatSys("Connected (or no-op if already connected).");
  }

  async function listIncoming() {
    const data = await apiCall("/list_incoming_connections", "GET");
    const ids = (data && data.peer_ids) || [];
    const box = $("incoming-box");
    const ul = $("incoming-list");
    ul.innerHTML = "";
    if (ids.length === 0) {
      box.hidden = true;
      logDiag("No pending incoming connections");
      return;
    }
    box.hidden = false;
    ids.forEach(function (id) {
      const li = document.createElement("li");
      li.className = "mono";
      li.textContent = id;
      ul.appendChild(li);
    });
  }

  async function listConnections() {
    const data = await apiCall("/list_connections", "GET");
    const ids = (data && data.peer_ids) || [];
    const box = $("connected-box");
    const ul = $("connected-list");
    ul.innerHTML = "";
    if (ids.length === 0) {
      box.hidden = true;
      logDiag("No established connections");
      return;
    }
    box.hidden = false;
    ids.forEach(function (id) {
      const li = document.createElement("li");
      li.className = "mono";
      li.textContent = id;
      ul.appendChild(li);
    });
  }

  async function acceptConnection() {
    syncPeerFields();
    const peer_id = $("peer-id").value.trim();
    if (!peer_id) {
      logDiag("Peer id required for accept", "err");
      return;
    }
    await apiCall("/accept_connection", "POST", {
      peer_id,
      timeout_ms: connTimeoutMs(),
    });
    chatSys("Accepted connection from peer.");
  }

  async function openStream() {
    syncPeerFields();
    let peer_id = $("stream-peer-id").value.trim();
    if (!peer_id) peer_id = $("peer-id").value.trim();
    if (!peer_id) {
      logDiag("Stream peer id required", "err");
      return;
    }
    const data = await apiCall("/open_stream", "POST", {
      peer_id,
      timeout_ms: streamTimeoutMs(),
    });
    if (data && data.stream_id) {
      state.streamId = data.stream_id;
      $("active-stream-id").textContent = state.streamId;
      $("btn-send").disabled = false;
      $("btn-close-stream").disabled = false;
      chatSys("Stream opened: " + state.streamId);
    }
  }

  async function acceptStream() {
    syncPeerFields();
    let peer_id = $("stream-peer-id").value.trim();
    if (!peer_id) peer_id = $("peer-id").value.trim();
    if (!peer_id) {
      logDiag("Stream peer id required", "err");
      return;
    }
    const data = await apiCall("/accept_stream", "POST", {
      peer_id,
      timeout_ms: streamTimeoutMs(),
    });
    if (data && data.stream_id) {
      state.streamId = data.stream_id;
      $("active-stream-id").textContent = state.streamId;
      $("btn-send").disabled = false;
      $("btn-close-stream").disabled = false;
      chatSys("Stream accepted: " + state.streamId);
    }
  }

  async function sendMessage() {
    if (!state.streamId) {
      logDiag("No active stream", "err");
      return;
    }
    const message = $("out-message").value;
    await apiCall("/stream_send", "POST", {
      stream_id: state.streamId,
      message,
    });
    chatLine("out: " + message, "out");
    $("out-message").value = "";
  }

  async function drainOnce() {
    if (!state.streamId) return;
    try {
      const data = await apiCall("/stream_drain", "POST", {
        stream_id: state.streamId,
      });
      const messages = (data && data.messages) || [];
      messages.forEach(function (m) {
        chatLine("in: " + m, "in");
      });
    } catch (e) {
      /* apiCall already logged */
    }
  }

  function startPolling() {
    if (state.pollTimer) return;
    state.pollTimer = setInterval(function () {
      if ($("poll-enabled").checked && state.streamId) {
        drainOnce();
      }
    }, POLL_MS);
  }

  function stopPolling() {
    if (state.pollTimer) {
      clearInterval(state.pollTimer);
      state.pollTimer = null;
    }
  }

  async function closeStream() {
    if (!state.streamId) return;
    const sid = state.streamId;
    const path = "/stream/" + encodeURIComponent(sid);
    await apiCall(path, "DELETE");
    state.streamId = "";
    $("active-stream-id").textContent = "—";
    $("btn-send").disabled = true;
    $("btn-close-stream").disabled = true;
    chatSys("Stream closed: " + sid);
  }

  function onPollToggle() {
    if ($("poll-enabled").checked) {
      startPolling();
      if (state.streamId) drainOnce();
    } else {
      stopPolling();
    }
  }

  document.addEventListener("DOMContentLoaded", function () {
    loadStored();
    $("conn-timeout").placeholder = String(defaultTimeoutMs());
    $("stream-timeout").placeholder = String(defaultTimeoutMs());

    state.apiBaseUrl = resolveApiBaseUrl();
    chatSys("API base: " + state.apiBaseUrl);

    $("btn-go-online").addEventListener("click", function () {
      goOnline().catch(function () {});
    });
    $("btn-refresh-online").addEventListener("click", function () {
      refreshOnline().catch(function () {});
    });
    $("btn-me-id").addEventListener("click", function () {
      fetchMeId().catch(function () {});
    });
    $("btn-connect").addEventListener("click", function () {
      doConnect().catch(function () {});
    });
    $("btn-list-connections").addEventListener("click", function () {
      listConnections().catch(function () {});
    });
    $("btn-list-incoming").addEventListener("click", function () {
      listIncoming().catch(function () {});
    });
    $("btn-accept").addEventListener("click", function () {
      acceptConnection().catch(function () {});
    });
    $("btn-open-stream").addEventListener("click", function () {
      openStream().catch(function () {});
    });
    $("btn-accept-stream").addEventListener("click", function () {
      acceptStream().catch(function () {});
    });
    $("btn-send").addEventListener("click", function () {
      sendMessage().catch(function () {});
    });
    $("btn-close-stream").addEventListener("click", function () {
      closeStream().catch(function () {});
    });
    $("poll-enabled").addEventListener("change", onPollToggle);

    $("peer-id").addEventListener("change", syncPeerFields);
    $("default-timeout").addEventListener("change", function () {
      saveStored();
      const d = defaultTimeoutMs();
      $("conn-timeout").placeholder = String(d);
      $("stream-timeout").placeholder = String(d);
    });

    refreshOnline().catch(function () {});
  });
})();
