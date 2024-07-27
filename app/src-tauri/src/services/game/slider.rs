use windows_sys::Win32::Foundation::HWND;

use super::*;

#[derive(thiserror::Error, Debug)]
#[allow(unused)]
pub enum SlideError {
    #[error("image error:`{0}`")]
    Image(#[from] image::ImageError),
    #[error("io error:`{0}`")]
    Io(#[from] std::io::Error),
    #[error("custom error:`{0}`")]
    Custom(String),
}

/// 坑位匹配。
#[allow(unused)]
pub fn slide_comparison(
    target_image: image::DynamicImage,
    background_image: image::DynamicImage,
) -> Result<(u32, u32), SlideError> {
    if target_image.width() != background_image.width()
        && target_image.height() != background_image.height()
    {
        eprintln!(
            "图片尺寸不相等 {}x{} != {}x{}",
            target_image.width(),
            target_image.height(),
            background_image.width(),
            background_image.height()
        );
        return Err(SlideError::Custom("图片尺寸不相等".to_string()));
    }

    let image = image::RgbImage::from_vec(
        target_image.width(),
        target_image.height(),
        target_image
            .as_bytes()
            .iter()
            .zip(background_image.as_bytes().iter())
            .map(|(a, b)| if a.abs_diff(*b) > 80 { 255 } else { 0 })
            .collect(),
    )
    .unwrap();

    let mut start_x = 0;
    let mut start_y = 0;

    for i in 0..image.width() {
        let mut count = 0;

        for j in 0..image.height() {
            let pixel = image[(i, j)];

            if pixel != image::Rgb([0, 0, 0]) {
                count += 1;
            }

            if count >= 5 && start_y == 0 {
                start_y = j - 5;
            }
        }

        if count >= 5 {
            start_x = i + 2;
            break;
        }
    }

    Ok((start_x, start_y))
}

#[allow(unused)]
pub fn cap_slide(hwnd: HWND) -> u32 {
    let target = capture_rect(hwnd, (738, 352, 738 + 170, 352 + 130));
    let base_color = get_color(hwnd, 738, 352);
    let tpl = match_template_index(base_color);
    if let Ok((x, y)) = slide_comparison(image::DynamicImage::ImageRgb8(target), tpl) {
        println!("x:{}, y:{}", x, y);
        x
    } else {
        0
    }
}

macro_rules! rgb {
    ($r: expr, $g: expr, $b: expr) => {
        $r << 16 | $g << 8 | $b
    };
}

#[allow(unused)]
struct Template {
    color: u32,
    picture: &'static [u8],
}

#[allow(unused)]
fn match_template_index(bgr: u32) -> image::DynamicImage {
    let tpls = vec![
        Template {
            color: rgb!(52, 127, 197),
            picture: include_bytes!("./res/0.png"),
        },
        Template {
            color: rgb!(238, 233, 232),
            picture: include_bytes!("./res/1.png"),
        },
        Template {
            color: rgb!(70, 105, 203),
            picture: include_bytes!("./res/2.png"),
        },
        Template {
            color: rgb!(142, 67, 74),
            picture: include_bytes!("./res/3.png"),
        },
        Template {
            color: rgb!(190, 203, 231),
            picture: include_bytes!("./res/4.png"),
        },
        Template {
            color: rgb!(7, 10, 38),
            picture: include_bytes!("./res/5.png"),
        },
        Template {
            color: rgb!(1, 21, 113),
            picture: include_bytes!("./res/6.png"),
        },
        Template {
            color: rgb!(162, 79, 49),
            picture: include_bytes!("./res/7.png"),
        },
    ];

    let mut min = 10000.;
    let mut index = 0;

    for (i, tpl) in tpls.iter().enumerate() {
        let diff = hsl_sim(bgr, tpl.color);

        if diff < min {
            min = diff;
            index = i;
        }
    }
    println!("min:{}, index:{}", min, index);
    image::load_from_memory(tpls[index].picture).unwrap()
}
