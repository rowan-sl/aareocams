use anyhow::Result;


fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    println!("Hello, World!");

    Ok(())
}
