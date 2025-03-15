mod anime_hug;
mod blame_serena;
mod random_capy_image;
mod random_cat_image;
mod flux_image;
mod createimage;

use crate::error::Error;
use crate::Data;

// Re-export with category = "Fun" explicitly set
pub fn animehug() -> poise::Command<Data, Error> {
    let mut cmd = anime_hug::animehug();
    cmd.category = Some("Fun".to_string());
    cmd
}

pub fn blame() -> poise::Command<Data, Error> {
    let mut cmd = blame_serena::blame();
    cmd.category = Some("Fun".to_string());
    cmd
}

pub fn randomcapyimage() -> poise::Command<Data, Error> {
    let mut cmd = random_capy_image::randomcapyimage();
    cmd.category = Some("Fun".to_string());
    cmd
}

pub fn randomcatimage() -> poise::Command<Data, Error> {
    let mut cmd = random_cat_image::randomcatimage();
    cmd.category = Some("Fun".to_string());
    cmd
}

pub fn fluximage() -> poise::Command<Data, Error> {
    let mut cmd = flux_image::fluximage();
    cmd.category = Some("Fun".to_string());
    cmd
}

pub fn createimage() -> poise::Command<Data, Error> {
    let mut cmd = createimage::createimage();
    cmd.category = Some("Fun".to_string());
    cmd
}
