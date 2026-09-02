use anyhow::Result;

use crate::render::DiagramRenderData;

const CSS: &str = include_str!("../../templates/styles.css");
const JS: &str = include_str!("../../templates/renderer.js");

pub fn assemble_html_with_favicon(data: &DiagramRenderData, favicon_data_uri: Option<&str>) -> Result<String> {
    let data_json = serde_json::to_string(data)?;
    let theme_attr = match data.meta.theme.as_str() {
        "dark" => "dark",
        "auto" => "auto",
        _ => "light",
    };

    let title_html = html_escape(&data.meta.title);
    let desc_html = data
        .meta
        .description
        .as_deref()
        .map(|d| format!("<p>{}</p>", html_escape(d)))
        .unwrap_or_default();

    let favicon_html = favicon_data_uri
        .map(|uri| format!("\n  <link rel=\"icon\" href=\"{}\">", uri))
        .unwrap_or_default();

    let brand_css = build_brand_css(data);
    let brand_header_html = build_brand_header(data);
    let footer_html = build_footer(data);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>{favicon}
  <style>
{css}
{brand_css}
  </style>
</head>
<body data-theme="{theme}">
  <header id="diagram-header">
    {brand_header}<h1>{title}</h1>
    {desc}
  </header>

  <main id="diagram-container"></main>

  <aside id="flow-controls" hidden></aside>
  <aside id="magnification-controls" aria-label="Magnification controls"></aside>

  <div id="tooltip" hidden></div>
{footer}
  <script type="application/json" id="diagram-data">
{data}
  </script>

  <script>
{js}
  </script>
</body>
</html>"#,
        title = title_html,
        theme = theme_attr,
        desc = desc_html,
        favicon = favicon_html,
        css = CSS,
        brand_css = brand_css,
        brand_header = brand_header_html,
        footer = footer_html,
        data = data_json,
        js = JS,
    );

    Ok(html)
}

fn build_brand_css(data: &DiagramRenderData) -> String {
    let branding = match &data.branding {
        Some(b) => b,
        None => return String::new(),
    };

    let mut vars = Vec::new();
    if let Some(ref c) = branding.primary_color {
        vars.push(format!("  --brand-primary: {};", c));
    }
    if let Some(ref c) = branding.secondary_color {
        vars.push(format!("  --brand-secondary: {};", c));
    }

    if vars.is_empty() {
        return String::new();
    }

    format!(":root {{\n{}\n}}", vars.join("\n"))
}

fn build_brand_header(data: &DiagramRenderData) -> String {
    let branding = match &data.branding {
        Some(b) => b,
        None => return String::new(),
    };

    let logo_uri = match &branding.logo_data_uri {
        Some(uri) => uri,
        None => return String::new(),
    };

    let alt = branding
        .logo_alt
        .as_deref()
        .unwrap_or("Logo");

    format!(
        "<div class=\"brand-logo brand-logo--{placement}\"><img src=\"{src}\" alt=\"{alt}\" height=\"{height}\"></div>\n    ",
        placement = html_escape(&branding.logo_placement),
        src = logo_uri,
        alt = html_escape(alt),
        height = branding.logo_height,
    )
}

fn build_footer(data: &DiagramRenderData) -> String {
    let branding = match &data.branding {
        Some(b) => b,
        None => return String::new(),
    };

    let text = match &branding.footer_text {
        Some(t) => t.clone(),
        None => match &branding.organization {
            Some(org) => org.clone(),
            None => return String::new(),
        },
    };

    let date_span = if branding.show_generated_date {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                let secs = d.as_secs();
                let days = secs / 86400;
                let years = (days as f64 / 365.25) as u64;
                let year = 1970 + years;
                let remaining = days - (years as f64 * 365.25) as u64;
                let month = remaining / 30 + 1;
                let day = remaining % 30 + 1;
                format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
            })
            .unwrap_or_default();
        format!(" <span class=\"footer-date\">Generated {}</span>", now)
    } else {
        String::new()
    };

    format!(
        "  <footer id=\"diagram-footer\"><span>{}</span>{}</footer>\n",
        html_escape(&text),
        date_span
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
