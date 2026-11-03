# 健壮版 CDP 探针：抓 WebView2 控制台错误 + 网络失败 + 页面真实状态。
# 用法：先以 WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS 启动 NeatDesk，再运行本脚本。
import json
import time
import threading
import urllib.request
import websocket

PORT = 9333


def wait_up(timeout=40):
    t0 = time.time()
    last = None
    while time.time() - t0 < timeout:
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json/version", timeout=2) as r:
                return json.loads(r.read())
        except Exception as e:
            last = e
        time.sleep(0.4)
    print("DEBUG PORT NOT UP, last:", last)
    return None


def main():
    ver = wait_up()
    if not ver:
        return
    print("WEBVIEW2:", ver.get("Browser"))
    data = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json", timeout=3))
    for t in data:
        print("TARGET:", t.get("type"), "|", t.get("url"), "|", t.get("title"))
    pages = [t for t in data if t.get("type") == "page"]
    if not pages:
        print("NO PAGE TARGET")
        return
    tgt = pages[0]
    print("\nUSING PAGE:", tgt.get("url"), "|", tgt.get("title"))

    ws = websocket.create_connection(tgt["webSocketDebuggerUrl"], timeout=10)
    msgs = []
    stop = False

    def reader():
        while not stop:
            try:
                ws.settimeout(0.5)
                m = json.loads(ws.recv())
                msgs.append(m)
            except websocket.WebSocketTimeoutException:
                continue
            except Exception:
                break

    th = threading.Thread(target=reader, daemon=True)
    th.start()

    seq = [0]

    def send(method, params=None, wait_id=True):
        seq[0] += 1
        ws.send(json.dumps({"id": seq[0], "method": method, "params": params or {}}))
        if not wait_id:
            return None
        deadline = time.time() + 8
        while time.time() < deadline:
            for m in list(msgs):
                if m.get("id") == seq[0]:
                    msgs.remove(m)
                    return m
            time.sleep(0.05)
        return None

    send("Runtime.enable")
    send("Log.enable")
    send("Network.enable")
    send("Page.enable")

    def dump_events(tag=""):
        evs = [m for m in msgs if "method" in m]
        print(f"\n=== EVENTS {tag} ({len(evs)}) ===")
        for m in evs:
            meth = m["method"]
            p = m.get("params", {})
            if meth == "Runtime.consoleAPICalled":
                args = " ".join(str(a.get("value", a.get("description", ""))) for a in p.get("args", []))
                print(f"  [console.{p.get('type')}] {args}")
            elif meth == "Runtime.exceptionThrown":
                d = p.get("exceptionDetails", {})
                print(f"  [exception] {d.get('text')} :: {d.get('exception', {}).get('description') or d.get('exception', {}).get('value')}")
            elif meth == "Log.entryAdded":
                e = p.get("entry", {})
                print(f"  [log.{e.get('level')}] {e.get('text')}  ({e.get('url')}:{e.get('lineNumber')})")
            elif meth == "Network.loadingFailed":
                print(f"  [net.fail] {p.get('errorText')} type={p.get('type')} req={p.get('requestId')}")
            elif meth == "Network.responseReceived":
                r = p.get("response", {})
                print(f"  [net.resp] {r.get('status')} {r.get('url')}")
            elif meth == "Page.frameNavigated":
                print(f"  [nav] {p.get('frame', {}).get('url')}")
            elif meth == "Page.loadEventFired":
                print(f"  [loadEvent]")
        # 清空事件，避免重复
        for m in list(msgs):
            if "method" in m:
                msgs.remove(m)

    print("\n--- collect 5s of initial events ---")
    time.sleep(5)
    dump_events("initial")

    expr = """(function(){
      var app = document.getElementById('app');
      var rs = performance.getEntriesByType('resource').map(function(r){return (r.name||'').replace(location.origin,'')+'|st='+(r.responseStatus===undefined?'?':r.responseStatus);});
      return JSON.stringify({
        href: location.href, readyState: document.readyState, title: document.title,
        hasTauri: !!window.__TAURI_INTERNALS__,
        appLen: app?app.innerHTML.length:-1,
        appHead: app?app.innerHTML.slice(0,160):null,
        scripts: Array.prototype.map.call(document.scripts,function(s){return s.src||'(inline)';}),
        resources: rs
      }, null, 1);
    })()"""
    r = send("Runtime.evaluate", {"expression": expr, "returnByValue": True})
    val = r.get("result", {}).get("result", {}).get("value") if r else None
    print("\n=== PAGE STATE ===")
    print(val if val else json.dumps(r, ensure_ascii=False)[:1500])

    print("\n--- try navigate to tauri asset url ---")
    send("Page.navigate", {"url": "http://tauri.localhost/index.html"}, wait_id=False)
    time.sleep(4)
    dump_events("after-navigate")
    r2 = send("Runtime.evaluate", {"expression": "JSON.stringify({href:location.href,bodyLen:document.body?document.body.innerHTML.length:-1,title:document.title,hasTauri:!!window.__TAURI_INTERNALS__})", "returnByValue": True})
    print("STATE AFTER NAV:", (r2.get("result", {}).get("result", {}).get("value") if r2 else None))

    stop = True
    try:
        ws.close()
    except Exception:
        pass


if __name__ == "__main__":
    main()
