/// Original: https://github.com/TeamDman/Cursor-Hero/blob/51611380997d74f74f76fa776be4892a9906c005/crates/winutils/src/win_icons.rs
/// License: MPL-2.0 (https://github.com/TeamDman/Cursor-Hero/blob/main/LICENSE)
mod errors;
mod math;

use errors::*;
use math::bgra_to_rgba;

use image::ImageBuffer;
use image::RgbaImage;
use itertools::Itertools;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfoExW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFOEXW;

pub fn get_images_from_exe(executable_path: &str) -> Result<Vec<RgbaImage>> {
    unsafe {
        let path_cstr = U16CString::from_str(executable_path)?;
        let path_pcwstr = PCWSTR(path_cstr.as_ptr());
        let num_icons_total = ExtractIconExW(path_pcwstr, -1, None, None, 0);
        if num_icons_total == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let mut large_icons = vec![HICON::default(); num_icons_total as usize];
        let mut small_icons = vec![HICON::default(); num_icons_total as usize];
        let num_icons_fetched = ExtractIconExW(
            path_pcwstr,
            0,
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            num_icons_total,
        );

        if num_icons_fetched == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let images = large_icons
            .iter()
            .chain(small_icons.iter())
            .map(|icon| convert_hicon_to_rgba_image(icon))
            .filter_map(|r| match r {
                Ok(img) => Some(img),
                Err(e) => {
                    eprintln!("Failed to convert HICON to RgbaImage: {:?}", e);
                    None
                }
            })
            .collect_vec();

        large_icons
            .iter()
            .chain(small_icons.iter())
            .filter(|icon| !icon.is_invalid())
            .map(|icon| DestroyIcon(*icon))
            .filter_map(|r| r.err())
            .for_each(|e| eprintln!("Failed to destroy icon: {:?}", e));

        Ok(images)
    }
}

// https://stackoverflow.com/a/23390460/11141271 -- How to extract 128x128 icon bitmap data from EXE in python
// https://stackoverflow.com/a/22885412/11141271 -- Save HICON as a png (Java reference)
pub fn convert_hicon_to_rgba_image(hicon: &HICON) -> Result<RgbaImage> {
    unsafe {
        let mut icon_info = ICONINFOEXW::default();
        icon_info.cbSize = std::mem::size_of::<ICONINFOEXW>() as u32;

        if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
            return Err(Error::from_win32().with_description(format!(
                "icon • GetIconInfoExW: {} {}:{}",
                file!(),
                line!(),
                column!()
            )));
        }
        let hdc_screen = CreateCompatibleDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor);

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: icon_info.xHotspot as i32 * 2,
                // https://discord.com/channels/1071140253181673492/1219071553929740328/1219359761389064273
                // Rafael: ImageBuffer::from_raw expects top-down bitmap, so your biHeight in the BITMAPINFOHEADER should be negative.
                biHeight: -(icon_info.yHotspot as i32 * 2),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: DIB_RGB_COLORS.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer: Vec<u8> =
            vec![0; (icon_info.xHotspot * 2 * icon_info.yHotspot * 2 * 4) as usize];

        if GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            icon_info.yHotspot * 2,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return Err(Error::from_win32().with_description(format!(
                "GetDIBits: {} {}:{}",
                file!(),
                line!(),
                column!()
            )));
        }
        // Clean up
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem);
        DeleteDC(hdc_screen);
        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);

        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(icon_info.xHotspot * 2, icon_info.yHotspot * 2, buffer)
            .ok_or_else(|| Error::ImageContainerNotBigEnough)?;
        return Ok(image);
    }
}
