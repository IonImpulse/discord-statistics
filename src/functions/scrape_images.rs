use std::io::copy;
use std::fs::File;
use reqwest;

async fn download_image(url: String, path: String, prefix: String, sep: String) -> bool {
    
    let response = reqwest::get(url).await;
    
    if response.is_ok() {

        let response = response.unwrap();

        let dest = {
            let file_name = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("untitled.bin");
    
            println!("File to download: '{}'", file_name);
            let file_name = format!("{}{}{}-{}", path, sep, prefix, file_name);
            println!("Location: '{:?}'", file_name);
            File::create(file_name)
        };

        let content = response.text().await;

        if dest.is_ok() && content.is_ok() {
            let copied_result = copy(&mut content.unwrap().as_bytes(), &mut dest.unwrap());

            if copied_result.is_ok() {
                return true;
            }
        }
    }

    return false
}