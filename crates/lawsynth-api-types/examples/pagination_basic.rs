use lawsynth_api_types::{Page, PageRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = PageRequest::new(None, 2, 250)?;
    let page = Page::new(
        vec!["run-a", "run-b"],
        Some("run-b".to_owned()),
        request.limit,
    )?;
    println!(
        "returned {} runs; next cursor: {:?}",
        page.items.len(),
        page.next
    );
    Ok(())
}
