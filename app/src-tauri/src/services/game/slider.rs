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
            color: rgb!(21, 27, 49),
            picture: include_bytes!("./res/0.png"),
        },
        Template {
            color: rgb!(43, 119, 190),
            picture: include_bytes!("./res/1.png"),
        },
        Template {
            color: rgb!(154, 206, 220),
            picture: include_bytes!("./res/2.png"),
        },
        Template {
            color: rgb!(101, 75, 52),
            picture: include_bytes!("./res/3.png"),
        },
        Template {
            color: rgb!(135, 92, 74),
            picture: include_bytes!("./res/4.png"),
        },
        Template {
            color: rgb!(198, 214, 234),
            picture: include_bytes!("./res/5.png"),
        },
        Template {
            color: rgb!(59, 161, 75),
            picture: include_bytes!("./res/6.png"),
        },
        Template {
            color: rgb!(111, 178, 94),
            picture: include_bytes!("./res/7.png"),
        },
        Template {
            color: rgb!(76, 66, 120),
            picture: include_bytes!("./res/8.png"),
        },
        Template {
            color: rgb!(2, 20, 118),
            picture: include_bytes!("./res/9.png"),
        },
        Template {
            color: rgb!(53, 32, 23),
            picture: include_bytes!("./res/10.png"),
        },
        Template {
            color: rgb!(130, 81, 48),
            picture: include_bytes!("./res/11.png"),
        },
        Template {
            color: rgb!(90, 84, 102),
            picture: include_bytes!("./res/12.png"),
        },
        Template {
            color: rgb!(242, 242, 241),
            picture: include_bytes!("./res/13.png"),
        },
        Template {
            color: rgb!(34, 18, 19),
            picture: include_bytes!("./res/14.png"),
        },
        Template {
            color: rgb!(99, 103, 140),
            picture: include_bytes!("./res/15.png"),
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
