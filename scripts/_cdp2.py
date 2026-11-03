# 用 CDP 手动驱动 WebView2 导航到 Tauri 资源 URL，观察能否加载。
import json
import time
import urllib.request

import websocket

PORT = 9222


def get_page():
    data = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json", timeout=3))
    return [t for t in data if t.get("type") == "page"][0]


def main():
    tgt = get_page()
    ws = websocket.create_connection(tgt["webSocketDebuggerUrl"], timeout=10)
    seq = [0]

    def send(method, params=None):
        seq[0] += 1
        ws.send(json.dumps({"id": seq[0], "method": method, "params": params or {}}))
        while True:
            m = json.loads(ws.recv())
            if m.get("id") == seq[0]:
                return m

    send("Page.enable")
    send("Network.enable")
    send("Runtime.enable")

    for url in ["http://tauri.localhost/index.html",
                "tauri://localhost/index.html",
                "http://localhost/index.html"]:
        print("=== navigate ->", url)
        r = send("Page.navigate", {"url": url})
        print("   result:", json.dumps(r.get("result", {}), ensure_ascii=False),
              r.get("error", ""))
        # 收集 1.5s 内的网络/加载事件
        t0 = time.time()
        while time.time() - t0 < 2.0:
            ws.settimeout(0.3)
            try:
                m = json.loads(ws.recv())
            except Exception:
                continue
            meth = m.get("method")
            if meth == "Network.loadingFailed":
                print("   [loadingFailed]", m["params"].get("errorText"),
                      "type=", m["params"].get("type"),
                      "url=", m["params"].get("requestId"))
            elif meth == "Page.frameNavigated":
                print("   [frameNavigated]", m["params"]["frame"].get("url"))
        ws.settimeout(10)
        st = send("Runtime.evaluate", {"expression":
                  "JSON.stringify({href:location.href,bodyLen:(document.body?document.body.innerHTML.length:-1),title:document.title})",
                  "returnByValue": True})
        print("   state:", st.get("result", {}).get("result", {}).get("value"))
    ws.close()


if __name__ == "__main__":
    main()
