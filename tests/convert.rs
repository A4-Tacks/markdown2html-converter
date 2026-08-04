use markdown2html_converter::{ConvertOptions, Theme, convert};

fn convert_to_string(markdown: &str, options: &ConvertOptions) -> String {
    String::from_utf8(convert(markdown, options).unwrap()).unwrap()
}

fn title_of(html: &str) -> String {
    let start = html.find("<title>").unwrap() + "<title>".len();
    let length = html[start..].find("</title>").unwrap();

    html[start..(start + length)].to_string()
}

#[test]
fn title_comes_from_the_first_heading() {
    let html =
        convert_to_string("# Hello world!\n\n# Another heading\n", &ConvertOptions::default());

    assert_eq!("Hello world!", title_of(html.as_str()));
}

#[test]
fn title_comes_from_the_front_matter() {
    let markdown = "---\ntitle: \"From the front matter\"\n---\n\n# Hello world!\n";

    let html = convert_to_string(markdown, &ConvertOptions::default());

    assert_eq!("From the front matter", title_of(html.as_str()));
    assert!(!html.contains("From the front matter</h"));
}

#[test]
fn the_given_title_wins() {
    let options = ConvertOptions {
        title: Some("The given title"),
        default_title: Some("The default title"),
        ..ConvertOptions::default()
    };

    let html = convert_to_string("# Hello world!", &options);

    assert_eq!("The given title", title_of(html.as_str()));
}

#[test]
fn title_falls_back_to_the_default_title() {
    let options = ConvertOptions {
        default_title: Some("The default title"),
        ..ConvertOptions::default()
    };

    let html = convert_to_string("Just a paragraph.", &options);

    assert_eq!("The default title", title_of(html.as_str()));
}

#[test]
fn math_is_embedded_only_when_the_document_has_math() {
    let options = ConvertOptions::default();

    let without_math = convert_to_string("It costs $5 and $10.", &options);
    let with_math = convert_to_string("The famous $E = mc^2$ equation.", &options);

    assert!(!without_math.contains("MathJax"));
    assert!(with_math.contains("MathJax"));
    assert!(with_math.contains("data-math-style=\"inline\""));
}

#[test]
fn highlight_is_embedded_only_when_a_code_block_has_a_language() {
    let options = ConvertOptions::default();

    let without_language = convert_to_string("```\nplain text\n```\n", &options);
    let with_language = convert_to_string("```rust\nfn main() {}\n```\n", &options);

    assert!(!without_language.contains("hljs"));
    assert!(with_language.contains("hljs"));
    assert!(with_language.contains("language-rust"));
}

#[test]
fn a_fixed_theme_is_written_to_the_html_element() {
    let options = ConvertOptions {
        theme: Theme::Dark,
        ..ConvertOptions::default()
    };

    let html = convert_to_string("# Hello world!", &options);

    assert!(html.contains("<html lang=\"en\" data-theme=\"dark\">"));
}

#[test]
fn the_lang_attribute_is_configurable() {
    let options = ConvertOptions {
        lang: "zh-TW",
        ..ConvertOptions::default()
    };

    let html = convert_to_string("# 你好，世界！", &options);

    assert!(html.contains("<html lang=\"zh-TW\">"));
}

#[test]
fn alerts_and_task_lists_are_rendered() {
    let markdown = "> [!NOTE]\n> Take note.\n\n- [x] Done\n- [ ] Not yet\n";

    let html = convert_to_string(markdown, &ConvertOptions::default());

    assert!(html.contains("markdown-alert-note"));
    assert!(html.contains("markdown-alert-title"));
    assert!(html.contains("task-list-item"));
}

#[test]
fn headings_get_an_anchor() {
    let html = convert_to_string("# Hello world!", &ConvertOptions::default());

    assert!(html.contains("<h1 id=\"hello-world\">"));
    assert!(html.contains("href=\"#hello-world\""));
}

#[test]
fn raw_html_is_only_kept_when_it_is_allowed() {
    let safe = convert_to_string("<b>bold</b>", &ConvertOptions::default());

    let options = ConvertOptions {
        allow_unsafe: true,
        ..ConvertOptions::default()
    };
    let unsafe_ = convert_to_string("<b>bold</b>", &options);

    assert!(!safe.contains("<b>bold</b>"));
    assert!(unsafe_.contains("<b>bold</b>"));
}

#[test]
fn the_output_can_be_left_unminified() {
    let options = ConvertOptions {
        minify: false,
        ..ConvertOptions::default()
    };

    let minified = convert_to_string("# Hello world!", &ConvertOptions::default());
    let plain = convert_to_string("# Hello world!", &options);

    assert!(plain.len() > minified.len());
    assert!(plain.contains("<article class=\"markdown-body\">"));
}
