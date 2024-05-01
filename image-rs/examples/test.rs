//! An example of listing the file names of entries in an archive.
//!
//! Takes a tarball on stdin and prints out all of the entries inside.

use image_rs::image::ImageClient;

#[tokio::main]
async fn main() {
    let image = "cgr.dev/chainguard/busybox@sha256:19f02276bf8dbdd62f069b922f10c65262cc34b710eea26ff928129a736be791";
    let mut image_client = ImageClient::default();

    let bundle_dir = tempfile::tempdir().unwrap();
    image_client
            .pull_image(image, bundle_dir.path(), &None, &None)
            .await
            .expect("failed to download image");
    println!("done.");
}

