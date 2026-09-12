use super::{Entry, Model};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, html};

pub(crate) const MARKER: &str = "<!-- meowy-doc-site:1 -->\n";

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn prose(entry: &Entry) -> String {
    let mut source = entry.text.clone();
    for link in entry.links.iter().rev() {
        let label = link
            .label
            .chars()
            .flat_map(|ch| {
                if ch.is_ascii_punctuation() {
                    vec!['\\', ch]
                } else {
                    vec![ch]
                }
            })
            .collect::<String>();
        let text = link
            .resolved
            .map_or(label.clone(), |id| format!("[{label}](#s{id})"));
        source.replace_range(link.range.clone(), &text);
    }
    let mut output = String::new();
    let events = Parser::new(&source).filter_map(|mut event| {
        match event {
            Event::Html(text) | Event::InlineHtml(text) => return Some(Event::Text(text)),
            Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image) => return None,
            _ => {}
        }
        if let Event::Start(Tag::Link { dest_url, .. }) = &mut event {
            let url = dest_url.trim().to_ascii_lowercase();
            if !(url.starts_with("https://")
                || url.starts_with("http://")
                || url.starts_with("mailto:")
                || url.starts_with("#s"))
            {
                *dest_url = "#".into();
            }
        }
        Some(event)
    });
    html::push_html(&mut output, events);
    output
}

pub(crate) fn page(model: &Model, name: &str) -> String {
    let mut out = format!(
        "{MARKER}<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'\"><title>{} | meowy API</title><style>{}</style><body><a class=\"skip\" href=\"#main\">Skip to reference</a><header><p>meowy / API REFERENCE</p><h1>{}</h1><p>Compiler-checked declarations. Examples report checking and execution separately.</p></header><div class=\"layout\"><nav aria-label=\"Declarations\"><h2>In this file</h2><ul>",
        escape(name),
        STYLE,
        escape(name)
    );
    for (id, entry) in model
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.public)
    {
        out.push_str(&format!(
            "<li><a href=\"#s{id}\">{}</a></li>",
            escape(&entry.name)
        ));
    }
    out.push_str("</ul></nav><main id=\"main\">");
    for (id, entry) in model
        .entries
        .iter()
        .enumerate()
        .filter(|(id, entry)| *id == 0 || entry.public)
    {
        if id == 0 && entry.doc.is_none() {
            continue;
        }
        out.push_str(&format!("<section id=\"s{id}\"><h2>{}</h2><p class=\"kind\">{:?}</p><pre class=\"signature\"><code>{}{}</code></pre>{}", escape(&entry.name), entry.kind, escape(&entry.name), if id == 0 { String::new() } else { format!("&lt;{}&gt;", escape(&entry.signature)) }, prose(entry)));
        for (index, example) in entry.examples.iter().enumerate() {
            out.push_str(&format!(
                "<p class=\"evidence\">Example {}: checked; {}</p>",
                index + 1,
                if example.ran {
                    "execution passed"
                } else {
                    "not executed"
                }
            ));
        }
        out.push_str("</section>");
    }
    out.push_str("</main></div><footer>Generated locally by the meowy bootstrap. No application initialization ran during documentation generation. Full LSP and package documentation remain separate capabilities.</footer></body></html>");
    out
}

pub(crate) const STYLE: &str = r#"
:root{color-scheme:light;--ink:#1e322a;--paper:#f7f4e9;--line:#c9cbbb;--accent:#27644c}*{box-sizing:border-box}body{margin:0;color:var(--ink);background:linear-gradient(140deg,#e7eddf,var(--paper) 42%);font:17px/1.65 Georgia,Cambria,serif}header,.layout,footer{max-width:1100px;margin:auto;padding:2rem}header{border-bottom:1px solid var(--line)}header>p:first-child,.kind{font:12px/1.4 monospace;letter-spacing:.13em}h1{font-weight:normal;font-size:clamp(2rem,5vw,4rem);line-height:1.1;margin:.8rem 0}h2{font-size:1.5rem;font-weight:normal}.layout{display:grid;grid-template-columns:220px minmax(0,1fr);gap:3rem}nav{align-self:start;position:sticky;top:1rem}nav ul{padding-left:1rem}a{color:var(--accent);text-underline-offset:.2em}a:focus-visible{outline:3px solid var(--accent);outline-offset:4px}section{padding:0 0 2rem;margin:0 0 2rem;border-bottom:1px solid var(--line)}pre{overflow:auto;padding:1rem;background:#ecefe4;border:1px solid var(--line);font-size:14px;line-height:1.5}code{font-family:"Courier New",monospace}.signature{background:#1e322a;color:#f7f4e9}.evidence{font-size:13px;color:#375d43}footer{font-size:13px;border-top:1px solid var(--line)}.skip{position:absolute;left:-9999px}.skip:focus{left:1rem;top:1rem;background:white;padding:.5rem}@media(max-width:700px){header,.layout,footer{padding:1.2rem}.layout{display:block}nav{position:static;border-bottom:1px solid var(--line);margin-bottom:2rem}nav ul{columns:2}pre{max-width:100%}}
"#;
