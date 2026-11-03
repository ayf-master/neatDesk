# 通过 WebView2 远程调试端口读取页面真实状态（控制台错误 + 资源加载结果）。
import json
import time
import urllib.request

import websocket

PORT = 9222


def wait_target(timeout=25):
    t0 = time.time()
    last = None
    while time.time() - t0 < timeout:
        try:
            data = json.load(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json", timeout=2))
            for t in data:
                print("TARGET:", t.get("type"), "|", t.get("url"), "|", t.get("title"))
            pages = [t for t in data if t.get("type") == "page"]
            if pages:
                return pages[0]
        except Exception as e:
            last = e
        time.sleep(0.4)
    print("NO TARGET, last err:", last)
    return None


def main():
    tgt = wait_target()
    if not tgt:
        return
    print("TITLE:", tgt.get("title"))
    print("URL  :", tgt.get("url"))
    ws = websocket.create_connection(tgt["webSocketDebuggerUrl"], timeout=8)

    def send(idx, method, params=None):
        ws.send(json.dumps({"id": idx, "method": method, "params": params or {}}))
        while True:
            msg = json.loads(ws.recv())
            if msg.get("id") == idx:
                return msg

    send(1, "Runtime.enable")
    send(2, "Log.enable")

    expr = """(function(){
      var res = performance.getEntriesByType('resource').map(function(r){
        return (r.name||'').replace(location.origin,'') + ' | status=' + (r.responseStatus===undefined?'?':r.responseStatus) + ' | size=' + (r.transferSize||0);
      });
      var app = document.getElementById('app');
      return JSON.stringify({
        href: location.href,
        readyState: document.readyState,
        title: document.title,
        hasTauriInternals: !!window.__TAURI_INTERNALS__,
        appLen: app ? app.innerHTML.length : -1,
        appHead: app ? app.innerHTML.slice(0,200) : null,
        scripts: Array.prototype.map.call(document.scripts, function(s){return s.src||'(inline)';}),
        resources: res
      }, null, 1);
    })()"""
    r = send(3, "Runtime.evaluate", {"expression": expr, "returnByValue": True})
    val = r.get("result", {}).get("result", {}).get("value")
    print("=== PAGE STATE ===")
    print(val if val else json.dumps(r, ensure_ascii=False)[:1500])
    ws.close()


if __name__ == "__main__":
    main()
