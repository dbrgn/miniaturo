use assert_cmd::Command;
use image::io::Reader as ImageReader;

fn get_test_files() -> Vec<String> {
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let index = std::fs::read_to_string(format!("{}/tests/testimages/index.txt", root_dir))
        .expect("Could not read testimages/index.txt");
    let mut files = Vec::new();
    for line in index.lines() {
        files.push(format!("{}/tests/testimages/{}", root_dir, line))
    }
    files
}

/// Ensure that thumbnails for all input files can be generated.
#[test]
fn generate_thumbnails() {
    let test_files = get_test_files();
    assert!(!test_files.is_empty());
    for filepath in test_files {
        let thumbpath = format!("{}.png", &filepath);

        // Generate thumb
        Command::cargo_bin("miniaturo")
            .unwrap()
            .arg("-i")
            .arg(&filepath)
            .arg("-o")
            .arg(&thumbpath)
            .assert()
            .success();

        // Load file (to ensure it's well-formed)
        let _img = ImageReader::open(&thumbpath).unwrap().decode().unwrap();
    }
}

/// Ensure that the proper output size is used.
#[test]
fn output_size() {
    let test_files = get_test_files();
    assert!(!test_files.is_empty());
    let filepath = &test_files[0];
    let sizes = [128, 256, 512];
    for size in sizes.iter() {
        let thumbpath = format!("{}.{}.png", &filepath, size);

        // Generate thumb
        Command::cargo_bin("miniaturo")
            .unwrap()
            .arg("-i")
            .arg(&filepath)
            .arg("-o")
            .arg(&thumbpath)
            .arg("-s")
            .arg(&format!("{}", size))
            .assert()
            .success();

        // Measure dimensions
        let s = *size;
        let (w, h) = image::image_dimensions(&thumbpath).unwrap();
        assert_eq!(w, s);
        assert!(h <= s);
    }
}
