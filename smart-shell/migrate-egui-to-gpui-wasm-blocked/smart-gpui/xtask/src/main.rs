//! Build tooling in Rust: embeds the wasm-bindgen output into the single-file page served by Vite.
//! usage: xtask bundle <wasm-bindgen-out-dir> <index.html>
use base64::Engine;
use std::{env, fs};

const PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Smart Shell — Rust · GPUI · WebAssembly</title>
<style>
html,body{margin:0;height:100%;overflow:hidden;background:#0b0f18;color:#c9d6ee;font:500 13px system-ui,sans-serif}
canvas{display:block;outline:none;touch-action:none}
#boot{position:fixed;inset:0;display:grid;place-items:center;pointer-events:none;transition:opacity .5s;text-align:center;z-index:2}
#boot i{width:34px;height:34px;border-radius:50%;border:3px solid #ffffff22;border-top-color:#6e9bff;animation:s .8s linear infinite;display:block;margin:0 auto 14px}
@keyframes s{to{transform:rotate(360deg)}}
</style>
</head>
<body>
<div id="boot"><div><i></i>Starting Smart Shell (GPUI · wasm)…</div></div>
<script>/*GLUE*/</script>
<script>
(function(){
  var b=atob("/*WASM*/"),u=new Uint8Array(b.length);
  for(var i=0;i<b.length;i++)u[i]=b.charCodeAt(i);
  wasm_bindgen({module_or_path:u}).then(function(){
    wasm_bindgen.start();
    var e=document.getElementById("boot");e.style.opacity=0;setTimeout(function(){e.remove()},600);window.__ready=true;
  }).catch(function(e){document.getElementById("boot").textContent="Failed to start: "+e;console.error(e);});
})();
</script>
</body>
</html>
"##;

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 4 || a[1] != "bundle" {
        eprintln!("usage: xtask bundle <wasm-bindgen-out-dir> <index.html>");
        std::process::exit(2);
    }
    let dir = &a[2];
    let glue = fs::read_to_string(format!("{dir}/smart_gpui.js")).expect("glue js");
    let wasm = fs::read(format!("{dir}/smart_gpui_bg.wasm")).expect("wasm");
    let b64 = base64::engine::general_purpose::STANDARD.encode(&wasm);
    // keep `</script` out of the inline glue
    let glue = glue.replace("</script", "<\\/script");
    let html = PAGE.replace("/*GLUE*/", &glue).replace("/*WASM*/", &b64);
    fs::write(&a[3], html).expect("write index.html");
    println!("bundled {} KB wasm -> {}", wasm.len() / 1024, a[3]);
}
