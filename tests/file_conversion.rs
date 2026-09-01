//! Integration tests: file-based conversion API
//! Covers: html_file_to_markdown, html_files_to_markdown (bulk parallel)

use mdka::options::{ConversionMode, ConversionOptions};

// ─── html_file_to_markdown ────────────────────────────────────────────────

#[test]
fn file_to_markdown_same_dir() {
    let dir = std::env::temp_dir().join("mdka_test_same_dir");
    std::fs::create_dir_all(&dir).unwrap();

    let src = dir.join("page.html");
    std::fs::write(&src, "<h1>Same Dir</h1><p>Content</p>").unwrap();

    let result = mdka::html_file_to_markdown(&src, None::<&str>).unwrap();

    assert_eq!(result.src, src);
    assert_eq!(result.dest, dir.join("page.md"));
    assert!(result.dest.exists(), "output file not created");
    let content = std::fs::read_to_string(&result.dest).unwrap();
    assert!(content.contains("# Same Dir"), "got: {content}");
    assert!(content.contains("Content"), "got: {content}");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn file_to_markdown_with_out_dir() {
    let dir = std::env::temp_dir().join("mdka_test_out_dir");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();

    let src = dir.join("article.html");
    std::fs::write(&src, "<h2>Article</h2><p>Body</p>").unwrap();

    let result = mdka::html_file_to_markdown(&src, Some(&out)).unwrap();

    assert_eq!(result.dest, out.join("article.md"));
    let content = std::fs::read_to_string(&result.dest).unwrap();
    assert!(content.contains("## Article"), "got: {content}");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn file_to_markdown_with_opts() {
    let dir = std::env::temp_dir().join("mdka_test_opts");
    std::fs::create_dir_all(&dir).unwrap();

    let src = dir.join("spa.html");
    std::fs::write(&src, "<nav>nav</nav><h1>Title</h1><p>Body</p>").unwrap();

    let mut opts = ConversionOptions::for_mode(ConversionMode::Minimal);
    opts.drop_interactive_shell = true;

    let result = mdka::html_file_to_markdown_with(&src, None::<&str>, &opts).unwrap();
    let content = std::fs::read_to_string(&result.dest).unwrap();

    assert!(content.contains("# Title"), "got: {content}");
    assert!(
        !content.to_lowercase().contains("nav"),
        "nav leaked: {content}"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn file_to_markdown_nonexistent_returns_err() {
    let result = mdka::html_file_to_markdown("/no/such/file.html", None::<&str>);
    assert!(result.is_err(), "expected Err for missing file");
}

#[test]
fn file_to_markdown_same_dir_vs_bulk_consistency() {
    let dir = std::env::temp_dir().join("mdka_test_consistency");
    let out1 = dir.join("out1");
    let out2 = dir.join("out2");
    std::fs::create_dir_all(&out1).unwrap();
    std::fs::create_dir_all(&out2).unwrap();

    let src = dir.join("test.html");
    std::fs::write(&src, "<h1>Hello</h1><p>World <strong>bold</strong></p>").unwrap();

    let r1 = mdka::html_file_to_markdown(&src, Some(&out1)).unwrap();
    let paths = vec![src.clone()];
    let r2 = mdka::html_files_to_markdown(&paths, &out2);

    let c1 = std::fs::read_to_string(&r1.dest).unwrap();
    let c2 = std::fs::read_to_string(r2[0].1.as_ref().unwrap()).unwrap();
    assert_eq!(c1, c2, "single and bulk outputs differ");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn html_files_to_markdown_stem_collision_first_wins() {
    // RFC 021 (S-02): two inputs whose output stems collide used to let
    // whichever rayon worker wrote last silently discard the other's
    // content while both reported success. First occurrence in input order
    // must now win deterministically; the later collider must error without
    // writing anything, naming both sources and the destination.
    let dir = std::env::temp_dir().join("mdka_test_collision");
    let dir_a = dir.join("a");
    let dir_b = dir.join("b");
    let out = dir.join("out");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();
    std::fs::create_dir_all(&out).unwrap();

    let src_a = dir_a.join("index.html");
    let src_b = dir_b.join("index.html");
    std::fs::write(&src_a, "<h1>FROM A</h1>").unwrap();
    std::fs::write(&src_b, "<h1>FROM B</h1>").unwrap();

    let paths = vec![src_a.clone(), src_b.clone()];
    let results = mdka::html_files_to_markdown(&paths, &out);

    assert_eq!(results.len(), 2);
    let (first_src, first_res) = &results[0];
    let (second_src, second_res) = &results[1];
    assert_eq!(*first_src, &src_a);
    assert_eq!(*second_src, &src_b);

    let dest = first_res
        .as_ref()
        .unwrap_or_else(|e| panic!("first occurrence must win, got: {e}"));
    assert_eq!(dest, &out.join("index.md"));

    let err = second_res
        .as_ref()
        .expect_err("later collider must return an error, not overwrite");
    let msg = err.to_string();
    assert!(msg.contains(&src_a.display().to_string()), "got: {msg}");
    assert!(msg.contains(&src_b.display().to_string()), "got: {msg}");
    assert!(msg.contains(&dest.display().to_string()), "got: {msg}");

    let content = std::fs::read_to_string(dest).unwrap();
    assert_eq!(
        content.trim(),
        "# FROM A",
        "surviving file must hold the first input's content, got: {content}"
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn html_files_to_markdown_stem_collision_deterministic_over_many_runs() {
    // The defect was a race: the winner depended on which rayon worker
    // finished last. Run the collision case repeatedly and confirm the
    // first input wins every time.
    let dir = std::env::temp_dir().join("mdka_test_collision_repeated");
    let dir_a = dir.join("a");
    let dir_b = dir.join("b");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();

    let src_a = dir_a.join("index.html");
    let src_b = dir_b.join("index.html");
    std::fs::write(&src_a, "<h1>FROM A</h1>").unwrap();
    std::fs::write(&src_b, "<h1>FROM B</h1>").unwrap();
    let paths = vec![src_a, src_b];

    for i in 0..20 {
        let out = dir.join(format!("out{i}"));
        std::fs::create_dir_all(&out).unwrap();
        let results = mdka::html_files_to_markdown(&paths, &out);
        assert!(results[0].1.is_ok(), "run {i}: first input must win");
        assert!(
            results[1].1.is_err(),
            "run {i}: second input must be rejected"
        );
        let content = std::fs::read_to_string(results[0].1.as_ref().unwrap()).unwrap();
        assert_eq!(content.trim(), "# FROM A", "run {i}: got {content}");
    }

    std::fs::remove_dir_all(&dir).unwrap();
}

// ─── html_files_to_markdown (parallel bulk) ───────────────────────────────

#[test]
fn html_files_to_markdown_parallel() {
    let dir = std::env::temp_dir().join("mdka_parallel_test");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).unwrap();

    let files: Vec<_> = (0..4)
        .map(|i| {
            let p = dir.join(format!("f{i}.html"));
            std::fs::write(&p, format!("<h1>File {i}</h1>")).unwrap();
            p
        })
        .collect();

    let results = mdka::html_files_to_markdown(&files, &out);

    assert_eq!(results.len(), 4);
    for (src, res) in &results {
        let dest = res
            .as_ref()
            .unwrap_or_else(|e| panic!("{}: {e}", src.display()));
        let content = std::fs::read_to_string(dest).unwrap();
        assert!(content.contains("File"), "output missing: {content}");
    }

    std::fs::remove_dir_all(&dir).unwrap();
}
