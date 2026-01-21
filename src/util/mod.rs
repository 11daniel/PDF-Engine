use crate::env::Env;

pub fn get_file_url(env: &Env, filename: &str, bucket: &str) -> String {
    let mut str = env.s3_public_url_format.clone();
    str = str.replace("%f", filename);
    str = str.replace("%b", bucket);
    str = str.replace("%%", "%");
    str
}
