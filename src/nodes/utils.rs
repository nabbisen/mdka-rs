use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::RcDom;

/// parse html str
pub fn parse_html(html: &str) -> RcDom {
    let optimized_html = optimize_html_to_be_well_parsed(html);
    parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut optimized_html.as_bytes())
        .unwrap()
}

/// trim spaces and new lines between end of tag and start of next tag
/// to prevent dirtily parsed with: either `</a>\n<a ...` or `</a> <a ...`
fn optimize_html_to_be_well_parsed(html: &str) -> String {
    let mut ret = String::new();

    let chars: Vec<char> = html.chars().collect();

    let mut start = 0;
    // trim between end of tag and start of next tag
    while let Some(pos) = chars[start..].iter().position(|&c| c == '>') {
        let end = match chars[(start + pos)..].iter().position(|&c| c == '<') {
            Some(end_pos) => start + pos + end_pos,
            None => break,
        };

        let start_to_bracket_end = &chars[start..(start + pos)].iter().collect::<String>();
        ret.push_str(start_to_bracket_end);
        ret.push('>');
        let between_brackets_end_start = &chars[(start + pos + 1)..end].iter().collect::<String>();
        ret.push_str(between_brackets_end_start.trim());
        ret.push('<');

        start = end + 1;
    }
    ret.push_str(&chars[start..].iter().collect::<String>());

    ret
}
