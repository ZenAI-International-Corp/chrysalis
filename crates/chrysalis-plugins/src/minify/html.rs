//! HTML minification.

use crate::Result;

/// Minify HTML content.
pub fn minify_html(content: &[u8]) -> Result<Vec<u8>> {
    let cfg = minify_html::Cfg {
        do_not_minify_doctype: false,
        ensure_spec_compliant_unquoted_attribute_values: false,
        keep_closing_tags: false,
        keep_html_and_head_opening_tags: false,
        keep_spaces_between_attributes: false,
        keep_comments: false,
        keep_input_type_text_attr: false,
        keep_ssi_comments: false,
        preserve_brace_template_syntax: false,
        preserve_chevron_percent_template_syntax: false,
        minify_css: true,
        minify_js: true,
        remove_bangs: true,
        remove_processing_instructions: true,
    };

    let minified = minify_html::minify(content, &cfg);
    Ok(minified)
}
