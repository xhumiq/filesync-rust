use chrono::prelude::*;
use chrono::{DateTime, Utc, Local, NaiveDate, NaiveDateTime, Duration};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use crate::models::files::MediaEntry;

lazy_static! {
    static ref RE_MULTIPLE_SPACES: Regex = Regex::new(r" +").expect("Invalid regex RE_MULTIPLE_SPACES");
    static ref RE_ALC588WMM: Regex = Regex::new(r"ALC 588 WMM").expect("Invalid regex RE_ALC588WMM");
    static ref RE_ALC588: Regex = Regex::new(r"ALC 588").expect("Invalid regex RE_ALC588");
    static ref RE_DATE_DIGITS: Regex = Regex::new(r"\b(\d{6})\b").expect("Invalid regex RE_DATE_DIGITS");
}

pub fn clean_pub_date(entries: Vec<MediaEntry>) -> Vec<MediaEntry> {
    let mut groups: HashMap<NaiveDateTime, Vec<MediaEntry>> = HashMap::new();
    for entry in entries {
        groups.entry(entry.pub_date).or_insert(Vec::new()).push(entry);
    }
    let mut result = Vec::new();
    for (pub_date_datetime, mut group) in groups {
        group.sort_by_key(|e| e.modified);
        if let Some(first) = group.first() {
            let first_modified = first.modified;
            let cutoff = first_modified + Duration::hours(1).to_std().expect("Invalid duration");
            let base_entry = group.iter().rev().find(|e| e.modified <= cutoff).unwrap_or(first);
            let mut base_time = base_entry.modified;
            let base_date = DateTime::<Utc>::from(base_time).date_naive();
            if base_date.day() != pub_date_datetime.day() {
                let adjusted_date = pub_date_datetime;
                let adjusted_datetime = adjusted_date.with_hour(23).unwrap().with_minute(55).unwrap();
                base_time = DateTime::<Utc>::from_naive_utc_and_offset(adjusted_datetime, Utc).into();
            }
            base_time = base_time - std::time::Duration::from_secs((group.len() + 1) as u64);
            // if base_time has a time of day at 0 zero hours and zero minuites and zero seconds then add 5 minutes to it
            let dt = DateTime::<Local>::from(base_time);
            if dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0 {
                base_time = (dt + chrono::Duration::minutes(5)).into();
            }
            for mut entry in group {
                //println!("{} {} {}", entry.file_name, DateTime::<Utc>::from(entry.modified).format("%m/%d %H:%M:%S"), DateTime::<Utc>::from(base_time).format("%m/%d %H:%M:%S"));
                entry.pub_date = DateTime::<Utc>::from(base_time).naive_utc();
                result.push(entry);
            }
        }
    }
    result
}

// ---------------------------------------------------------------------
// Helper: check if a character is Chinese
// ---------------------------------------------------------------------
pub fn is_chinese(c: char) -> bool {
    // CJK Unified Ideographs
    ('\u{4e00}' <= c && c <= '\u{9fff}') ||
    // CJK Extension A
    ('\u{3400}' <= c && c <= '\u{4dbf}') ||
    // CJK Extension B
    ('\u{20000}' <= c && c <= '\u{2a6df}') ||
    // CJK Extension C
    ('\u{2a700}' <= c && c <= '\u{2b73f}')
}

// ---------------------------------------------------------------------
// Helper: extract Chinese characters from a string
// ---------------------------------------------------------------------
pub fn extract_chinese(s: &str) -> String {
    s.chars()
        .filter(|c| is_chinese(*c))
        .collect()
}

// ---------------------------------------------------------------------
// Helper: format name by inserting spaces before capital letters followed by lowercase, between letters and digits, replacing dashes with spaces, and formatting 6-digit dates
// ---------------------------------------------------------------------
pub fn format_eng_descr(s: &str) -> String {
    // Now apply spacing
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut prev_is_digit = false;
    let mut prev_is_letter = false;
    for i in 0..chars.len() {
        let c = chars[i];
        let is_digit = c.is_ascii_digit();
        let is_letter = c.is_alphabetic();
        if i > 0 && chars[i-1].is_lowercase() && c.is_uppercase() {
            result.push(' ');
        }
        if i > 0 && ((prev_is_letter && is_digit) || (prev_is_digit && is_letter)) {
            result.push(' ');
        }
        if c == '-' {
            result.push(' ');
        } else if c == ',' {
            result.push(',');
            result.push(' ');
        } else if c == '&' {
            result.push(' ');
            result.push('&');
            result.push(' ');
        } else {
            result.push(c);
        }
        prev_is_digit = is_digit;
        prev_is_letter = is_letter;
    }
    // Replace multiple spaces with single space
    let result = RE_MULTIPLE_SPACES.replace_all(&result, " ").to_string();
    let result = RE_ALC588WMM.replace_all(&result, "ALC/588/WMM").to_string();
    let result = RE_ALC588.replace_all(&result, "ALC/588").to_string();

    // First, replace 6-digit dates
    RE_DATE_DIGITS.replace_all(&result, |caps: &regex::Captures| {
        let digits = &caps[1];
        if let Ok(date) = NaiveDate::parse_from_str(digits, "%y%m%d") {
            date.format("%Y-%m-%d").to_string()
        } else {
            digits.to_string()
        }
    }).to_string()

}

pub fn format_event_date(ed: &str) -> String {
    if ed.len() == 6 {
        format!(" 20{}-{}-{}", &ed[0..2], &ed[2..4], &ed[4..6])
    } else {
        String::new()
    }
}

pub fn normalize_location(loc: &str) -> String {
    match loc {
        "MH" => "MtHermon".to_string(),
        "KL" => "Kuala Lumper".to_string(),
        "KK" => "Kota Kinabalu".to_string(),
        "CL" => "Canaan Land".to_string(),
        "IL" => "Isaac Land".to_string(),
        "DL" => "Dawnlight".to_string(),
        "AU" => "Australia".to_string(),
        "US" => "United States".to_string(),
        "CA" => "Canada".to_string(),
        "LA" => "Los Angeles".to_string(),
        "Joseph" => "Joseph Land".to_string(),
        "Olive" => "MtOlive".to_string(),
        "Carmel" => "MtCarmel".to_string(),
        _ => loc.to_string(),
    }
}

pub fn parse_mime_type(filename: &str) -> Option<String> {
    let path = std::path::Path::new(filename);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    MIME_TYPE_MAP.get(ext.as_str()).map(|s| s.to_string())
}

pub fn parse_media_type(filename: &str) -> String {
    if let Some(mime) = parse_mime_type(filename) {
        parse_media_type_from_mime(&mime)
    } else {
        "unknown".to_string()
    }
}

pub fn parse_media_type_from_mime(mime_type: &str) -> String {
    MEDIA_TYPE_MAP.get(mime_type)
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Fallback logic for MIME types not in the map
            if mime_type.starts_with("video/") {
                "video".to_string()
            } else if mime_type.starts_with("audio/") {
                "audio".to_string()
            } else if mime_type.starts_with("image/") {
                "image".to_string()
            } else if mime_type.starts_with("text/") {
                "document".to_string()
            } else if mime_type.starts_with("application/") {
                if mime_type.contains("zip") || mime_type.contains("tar") ||
                   mime_type.contains("rar") || mime_type.contains("compress") {
                    "archive".to_string()
                } else if mime_type.contains("json") {
                    "json".to_string()
                } else if mime_type.contains("xml") {
                    "xml".to_string()
                } else {
                    "document".to_string()
                }
            } else {
                "unknown".to_string()
            }
        })
}

lazy_static! {
    pub static ref MIME_TYPE_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        // Video formats
        map.insert("mp4", "video/mp4");
        map.insert("avi", "video/x-msvideo");
        map.insert("wmv", "video/x-ms-wmv");
        map.insert("mkv", "video/x-matroska");
        map.insert("mov", "video/quicktime");
        map.insert("flv", "video/x-flv");
        map.insert("webm", "video/webm");
        map.insert("m4v", "video/x-m4v");
        map.insert("3gp", "video/3gpp");
        map.insert("3g2", "video/3gpp2");
        map.insert("mpg", "video/mpeg");
        map.insert("mpeg", "video/mpeg");
        map.insert("m2v", "video/mpeg");
        map.insert("mpe", "video/mpeg");
        map.insert("mpv", "video/mpeg");
        map.insert("mp2", "video/mpeg");
        map.insert("m1v", "video/mpeg");
        map.insert("m2ts", "video/mp2t");
        map.insert("mts", "video/mp2t");
        map.insert("ts", "video/mp2t");
        map.insert("vob", "video/dvd");
        map.insert("asf", "video/x-ms-asf");
        map.insert("rm", "video/x-pn-realvideo");
        map.insert("rmvb", "video/x-pn-realvideo");
        map.insert("ogv", "video/ogg");
        map.insert("divx", "video/x-divx");
        map.insert("xvid", "video/x-xvid");
        map.insert("f4v", "video/x-f4v");
        map.insert("mxf", "application/mxf");
        map.insert("dv", "video/x-dv");
        map.insert("qt", "video/quicktime");
        map.insert("yuv", "video/x-raw-yuv");
        map.insert("y4m", "video/x-yuv4mpeg");
        map.insert("264", "video/h264");
        map.insert("h264", "video/h264");
        map.insert("265", "video/h265");
        map.insert("h265", "video/h265");
        map.insert("hevc", "video/h265");
        map.insert("av1", "video/av01");
        map.insert("ivf", "video/x-ivf");
        
        // Audio formats
        map.insert("mp3", "audio/mpeg");
        map.insert("wav", "audio/wav");
        map.insert("wave", "audio/wav");
        map.insert("flac", "audio/flac");
        map.insert("aac", "audio/aac");
        map.insert("ogg", "audio/ogg");
        map.insert("oga", "audio/ogg");
        map.insert("wma", "audio/x-ms-wma");
        map.insert("m4a", "audio/mp4");
        map.insert("m4b", "audio/mp4");
        map.insert("m4p", "audio/mp4");
        map.insert("opus", "audio/opus");
        map.insert("webm", "audio/webm");
        map.insert("3ga", "audio/3gpp");
        map.insert("amr", "audio/amr");
        map.insert("awb", "audio/amr-wb");
        map.insert("au", "audio/basic");
        map.insert("snd", "audio/basic");
        map.insert("mid", "audio/midi");
        map.insert("midi", "audio/midi");
        map.insert("kar", "audio/midi");
        map.insert("rmi", "audio/midi");
        map.insert("mp2", "audio/mpeg");
        map.insert("mp1", "audio/mpeg");
        map.insert("mpa", "audio/mpeg");
        map.insert("m2a", "audio/mpeg");
        map.insert("m3a", "audio/mpeg");
        map.insert("ra", "audio/x-pn-realaudio");
        map.insert("ram", "audio/x-pn-realaudio");
        map.insert("rm", "audio/x-pn-realaudio");
        map.insert("aif", "audio/x-aiff");
        map.insert("aiff", "audio/x-aiff");
        map.insert("aifc", "audio/x-aiff");
        map.insert("gsm", "audio/gsm");
        map.insert("wv", "audio/x-wavpack");
        map.insert("ape", "audio/x-ape");
        map.insert("tak", "audio/x-tak");
        map.insert("tta", "audio/x-tta");
        map.insert("weba", "audio/webm");
        map.insert("dts", "audio/vnd.dts");
        map.insert("dtshd", "audio/vnd.dts.hd");
        map.insert("ac3", "audio/ac3");
        map.insert("eac3", "audio/eac3");
        map.insert("mlp", "audio/x-mlp");
        map.insert("thd", "audio/x-truehd");
        map.insert("pcm", "audio/pcm");
        map.insert("adpcm", "audio/adpcm");
        map.insert("s3m", "audio/s3m");
        map.insert("xm", "audio/xm");
        map.insert("it", "audio/it");
        map.insert("mod", "audio/mod");
        map.insert("669", "audio/669");
        map.insert("amf", "audio/amf");
        map.insert("ams", "audio/ams");
        map.insert("dbm", "audio/dbm");
        map.insert("dmf", "audio/dmf");
        map.insert("dsm", "audio/dsm");
        map.insert("far", "audio/far");
        map.insert("mdl", "audio/mdl");
        map.insert("med", "audio/med");
        map.insert("mtm", "audio/mtm");
        map.insert("okt", "audio/okt");
        map.insert("ptm", "audio/ptm");
        map.insert("stm", "audio/stm");
        map.insert("ult", "audio/ult");
        map.insert("umx", "audio/umx");
        map.insert("mt2", "audio/mt2");
        map.insert("psm", "audio/psm");
        
        // Image formats
        map.insert("jpg", "image/jpeg");
        map.insert("jpeg", "image/jpeg");
        map.insert("jpe", "image/jpeg");
        map.insert("jfif", "image/jpeg");
        map.insert("png", "image/png");
        map.insert("gif", "image/gif");
        map.insert("bmp", "image/bmp");
        map.insert("dib", "image/bmp");
        map.insert("tiff", "image/tiff");
        map.insert("tif", "image/tiff");
        map.insert("svg", "image/svg+xml");
        map.insert("svgz", "image/svg+xml");
        map.insert("webp", "image/webp");
        map.insert("ico", "image/x-icon");
        map.insert("cur", "image/x-icon");
        map.insert("pbm", "image/x-portable-bitmap");
        map.insert("pgm", "image/x-portable-graymap");
        map.insert("ppm", "image/x-portable-pixmap");
        map.insert("pnm", "image/x-portable-anymap");
        map.insert("xbm", "image/x-xbitmap");
        map.insert("xpm", "image/x-xpixmap");
        map.insert("pcx", "image/x-pcx");
        map.insert("tga", "image/x-tga");
        map.insert("ras", "image/x-cmu-raster");
        map.insert("psd", "image/vnd.adobe.photoshop");
        map.insert("xcf", "image/x-xcf");
        map.insert("pat", "image/x-gimp-pat");
        map.insert("gbr", "image/x-gimp-gbr");
        map.insert("xwd", "image/x-xwindowdump");
        map.insert("rgb", "image/x-rgb");
        map.insert("rgba", "image/x-rgb");
        map.insert("sgi", "image/x-sgi");
        map.insert("bw", "image/x-sgi");
        map.insert("int", "image/x-sgi");
        map.insert("inta", "image/x-sgi");
        map.insert("pic", "image/x-pict");
        map.insert("pct", "image/x-pict");
        map.insert("pict", "image/x-pict");
        map.insert("sun", "image/x-sun-raster");
        map.insert("sr", "image/x-sun-raster");
        map.insert("im1", "image/x-sun-raster");
        map.insert("im8", "image/x-sun-raster");
        map.insert("im24", "image/x-sun-raster");
        map.insert("im32", "image/x-sun-raster");
        map.insert("rs", "image/x-sun-raster");
        map.insert("scr", "image/x-sun-raster");
        map.insert("fits", "image/fits");
        map.insert("fit", "image/fits");
        map.insert("fts", "image/fits");
        map.insert("hdr", "image/vnd.radiance");
        map.insert("exr", "image/x-exr");
        map.insert("dpx", "image/x-dpx");
        map.insert("cin", "image/x-cineon");
        map.insert("jp2", "image/jp2");
        map.insert("j2k", "image/jp2");
        map.insert("jpf", "image/jp2");
        map.insert("jpx", "image/jp2");
        map.insert("jpm", "image/jp2");
        map.insert("mj2", "image/jp2");
        map.insert("avif", "image/avif");
        map.insert("heif", "image/heif");
        map.insert("heic", "image/heic");
        map.insert("jxl", "image/jxl");
        map.insert("jxr", "image/vnd.ms-photo");
        map.insert("wdp", "image/vnd.ms-photo");
        map.insert("hdp", "image/vnd.ms-photo");
        // Document formats
        map.insert("pdf", "application/pdf");
        map.insert("doc", "application/msword");
        map.insert("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        map.insert("xls", "application/vnd.ms-excel");
        map.insert("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        map.insert("ppt", "application/vnd.ms-powerpoint");
        map.insert("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation");
        map.insert("txt", "text/plain");
        map.insert("rtf", "application/rtf");
        // Archive formats
        map.insert("zip", "application/zip");
        map.insert("rar", "application/x-rar-compressed");
        map.insert("7z", "application/x-7z-compressed");
        map.insert("tar", "application/x-tar");
        map.insert("tgz", "application/application/x-gzip");
        map.insert("bz2", "application/application/x-bzip2");
        map.insert("dmg", "application/x-apple-diskimage");
        map.insert("jar", "application/java-archive");
        map.insert("zst", "application/zstd");
        map.insert("gz", "application/gzip");
        // Other
        map.insert("json", "application/json");
        map.insert("xml", "application/xml");
        map.insert("html", "text/html");
        map.insert("css", "text/css");
        map.insert("js", "application/javascript");
        // Source code formats
        map.insert("rs", "text/x-rust");
        map.insert("py", "text/x-python");
        map.insert("java", "text/x-java-source");
        map.insert("cpp", "text/x-c++src");
        map.insert("c", "text/x-csrc");
        map.insert("h", "text/x-chdr");
        map.insert("hpp", "text/x-c++hdr");
        map.insert("go", "text/x-go");
        map.insert("php", "text/x-php");
        map.insert("rb", "text/x-ruby");
        map.insert("swift", "text/x-swift");
        map.insert("kt", "text/x-kotlin");
        map.insert("scala", "text/x-scala");
        map.insert("sh", "text/x-shellscript");
        map.insert("bash", "text/x-shellscript");
        map.insert("zsh", "text/x-shellscript");
        map.insert("fish", "text/x-shellscript");
        map.insert("ps1", "text/x-powershell");
        map.insert("bat", "text/x-msdos-batch");
        map.insert("cmd", "text/x-msdos-batch");
        map.insert("sql", "text/x-sql");
        map.insert("r", "text/x-r");
        map.insert("m", "text/x-matlab");
        map.insert("pl", "text/x-perl");
        map.insert("lua", "text/x-lua");
        map.insert("dart", "text/x-dart");
        map.insert("ts", "text/x-typescript");
        map.insert("tsx", "text/x-typescript");
        map.insert("jsx", "text/x-javascript");
        map.insert("vue", "text/x-vue");
        map.insert("svelte", "text/x-svelte");
        map.insert("elm", "text/x-elm");
        map.insert("clj", "text/x-clojure");
        map.insert("cljs", "text/x-clojure");
        map.insert("hs", "text/x-haskell");
        map.insert("ml", "text/x-ocaml");
        map.insert("fs", "text/x-fsharp");
        map.insert("ex", "text/x-elixir");
        map.insert("exs", "text/x-elixir");
        map.insert("erl", "text/x-erlang");
        map.insert("nim", "text/x-nim");
        map.insert("cr", "text/x-crystal");
        map.insert("zig", "text/x-zig");
        map.insert("d", "text/x-d");
        map.insert("pas", "text/x-pascal");
        map.insert("ada", "text/x-ada");
        map.insert("f90", "text/x-fortran");
        map.insert("f95", "text/x-fortran");
        map.insert("cob", "text/x-cobol");
        map.insert("asm", "text/x-asm");
        map.insert("s", "text/x-asm");
        map.insert("vb", "text/x-vb");
        map.insert("vbs", "text/x-vbscript");
        map.insert("cs", "text/x-csharp");
        map.insert("fs", "text/x-fsharp");
        map.insert("vhd", "text/x-vhdl");
        map.insert("vhdl", "text/x-vhdl");
        map.insert("v", "text/x-verilog");
        map.insert("sv", "text/x-systemverilog");
        map.insert("tcl", "text/x-tcl");
        map.insert("groovy", "text/x-groovy");
        map.insert("gradle", "text/x-gradle");
        map.insert("makefile", "text/x-makefile");
        map.insert("mk", "text/x-makefile");
        map.insert("cmake", "text/x-cmake");
        map.insert("dockerfile", "text/x-dockerfile");
        map.insert("yaml", "text/x-yaml");
        map.insert("yml", "text/x-yaml");
        map.insert("toml", "text/x-toml");
        map.insert("ini", "text/x-ini");
        map.insert("cfg", "text/x-config");
        map.insert("conf", "text/x-config");
        map.insert("properties", "text/x-properties");
        map.insert("gitignore", "text/x-gitignore");
        map.insert("gitattributes", "text/x-gitattributes");
        map.insert("editorconfig", "text/x-editorconfig");
        map.insert("md", "text/x-markdown");
        map.insert("markdown", "text/x-markdown");
        map.insert("rst", "text/x-rst");
        map.insert("tex", "text/x-tex");
        map.insert("latex", "text/x-latex");
        map.insert("bib", "text/x-bibtex");
        map
    };

    pub static ref MEDIA_TYPE_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        
        // Video MIME types
        map.insert("video/mp4", "video");
        map.insert("video/x-msvideo", "video");
        map.insert("video/x-ms-wmv", "video");
        map.insert("video/x-matroska", "video");
        map.insert("video/quicktime", "video");
        map.insert("video/x-flv", "video");
        map.insert("video/webm", "video");
        map.insert("video/x-m4v", "video");
        map.insert("video/3gpp", "video");
        map.insert("video/3gpp2", "video");
        map.insert("video/mpeg", "video");
        map.insert("video/mp2t", "video");
        map.insert("video/dvd", "video");
        map.insert("video/x-ms-asf", "video");
        map.insert("video/x-pn-realvideo", "video");
        map.insert("video/ogg", "video");
        map.insert("video/x-divx", "video");
        map.insert("video/x-xvid", "video");
        map.insert("video/x-f4v", "video");
        map.insert("application/mxf", "video");
        map.insert("video/x-dv", "video");
        map.insert("video/x-raw-yuv", "video");
        map.insert("video/x-yuv4mpeg", "video");
        map.insert("video/h264", "video");
        map.insert("video/h265", "video");
        map.insert("video/av01", "video");
        map.insert("video/x-ivf", "video");
        
        // Audio MIME types
        map.insert("audio/mpeg", "audio");
        map.insert("audio/wav", "audio");
        map.insert("audio/flac", "audio");
        map.insert("audio/aac", "audio");
        map.insert("audio/ogg", "audio");
        map.insert("audio/x-ms-wma", "audio");
        map.insert("audio/mp4", "audio");
        map.insert("audio/opus", "audio");
        map.insert("audio/webm", "audio");
        map.insert("audio/3gpp", "audio");
        map.insert("audio/amr", "audio");
        map.insert("audio/amr-wb", "audio");
        map.insert("audio/basic", "audio");
        map.insert("audio/midi", "audio");
        map.insert("audio/x-pn-realaudio", "audio");
        map.insert("audio/x-aiff", "audio");
        map.insert("audio/gsm", "audio");
        map.insert("audio/x-wavpack", "audio");
        map.insert("audio/x-ape", "audio");
        map.insert("audio/x-tak", "audio");
        map.insert("audio/x-tta", "audio");
        map.insert("audio/vnd.dts", "audio");
        map.insert("audio/vnd.dts.hd", "audio");
        map.insert("audio/ac3", "audio");
        map.insert("audio/eac3", "audio");
        map.insert("audio/x-mlp", "audio");
        map.insert("audio/x-truehd", "audio");
        map.insert("audio/pcm", "audio");
        map.insert("audio/adpcm", "audio");
        map.insert("audio/s3m", "audio");
        map.insert("audio/xm", "audio");
        map.insert("audio/it", "audio");
        map.insert("audio/mod", "audio");
        map.insert("audio/669", "audio");
        map.insert("audio/amf", "audio");
        map.insert("audio/ams", "audio");
        map.insert("audio/dbm", "audio");
        map.insert("audio/dmf", "audio");
        map.insert("audio/dsm", "audio");
        map.insert("audio/far", "audio");
        map.insert("audio/mdl", "audio");
        map.insert("audio/med", "audio");
        map.insert("audio/mtm", "audio");
        map.insert("audio/okt", "audio");
        map.insert("audio/ptm", "audio");
        map.insert("audio/stm", "audio");
        map.insert("audio/ult", "audio");
        map.insert("audio/umx", "audio");
        map.insert("audio/mt2", "audio");
        map.insert("audio/psm", "audio");
        
        // Image MIME types
        map.insert("image/jpeg", "image");
        map.insert("image/png", "image");
        map.insert("image/gif", "image");
        map.insert("image/bmp", "image");
        map.insert("image/tiff", "image");
        map.insert("image/svg+xml", "image");
        map.insert("image/webp", "image");
        map.insert("image/x-icon", "image");
        map.insert("image/x-portable-bitmap", "image");
        map.insert("image/x-portable-graymap", "image");
        map.insert("image/x-portable-pixmap", "image");
        map.insert("image/x-portable-anymap", "image");
        map.insert("image/x-xbitmap", "image");
        map.insert("image/x-xpixmap", "image");
        map.insert("image/x-pcx", "image");
        map.insert("image/x-tga", "image");
        map.insert("image/x-cmu-raster", "image");
        map.insert("image/vnd.adobe.photoshop", "image");
        map.insert("image/x-xcf", "image");
        map.insert("image/x-gimp-pat", "image");
        map.insert("image/x-gimp-gbr", "image");
        map.insert("image/x-xwindowdump", "image");
        map.insert("image/x-rgb", "image");
        map.insert("image/x-sgi", "image");
        map.insert("image/x-pict", "image");
        map.insert("image/x-sun-raster", "image");
        map.insert("image/fits", "image");
        map.insert("image/vnd.radiance", "image");
        map.insert("image/x-exr", "image");
        map.insert("image/x-dpx", "image");
        map.insert("image/x-cineon", "image");
        map.insert("image/jp2", "image");
        map.insert("image/avif", "image");
        map.insert("image/heif", "image");
        map.insert("image/heic", "image");
        map.insert("image/jxl", "image");
        map.insert("image/vnd.ms-photo", "image");
        
        // Document MIME types
        map.insert("application/pdf", "document");
        map.insert("application/msword", "document");
        map.insert("application/vnd.openxmlformats-officedocument.wordprocessingml.document", "document");
        map.insert("application/vnd.ms-excel", "document");
        map.insert("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "document");
        map.insert("application/vnd.ms-powerpoint", "document");
        map.insert("application/vnd.openxmlformats-officedocument.presentationml.presentation", "document");
        map.insert("text/plain", "document");
        map.insert("application/rtf", "document");
        map.insert("text/x-markdown", "document");
        map.insert("text/x-rst", "document");
        map.insert("text/x-tex", "document");
        map.insert("text/x-latex", "document");
        map.insert("text/x-bibtex", "document");
        
        // Archive MIME types
        map.insert("application/zip", "archive");
        map.insert("application/x-rar-compressed", "archive");
        map.insert("application/x-7z-compressed", "archive");
        map.insert("application/x-tar", "archive");
        map.insert("application/application/x-gzip", "archive");
        map.insert("application/application/x-bzip2", "archive");
        map.insert("application/x-apple-diskimage", "archive");
        map.insert("application/java-archive", "archive");
        map.insert("application/zstd", "archive");
        map.insert("application/gzip", "archive");
        
        // JSON and XML MIME types
        map.insert("application/json", "json");
        map.insert("application/xml", "xml");
        map.insert("text/xml", "xml");
        
        // Source code MIME types
        map.insert("text/html", "source code");
        map.insert("text/css", "source code");
        map.insert("application/javascript", "source code");
        map.insert("text/x-rust", "source code");
        map.insert("text/x-python", "source code");
        map.insert("text/x-java-source", "source code");
        map.insert("text/x-c++src", "source code");
        map.insert("text/x-csrc", "source code");
        map.insert("text/x-chdr", "source code");
        map.insert("text/x-c++hdr", "source code");
        map.insert("text/x-go", "source code");
        map.insert("text/x-php", "source code");
        map.insert("text/x-ruby", "source code");
        map.insert("text/x-swift", "source code");
        map.insert("text/x-kotlin", "source code");
        map.insert("text/x-scala", "source code");
        map.insert("text/x-shellscript", "source code");
        map.insert("text/x-powershell", "source code");
        map.insert("text/x-msdos-batch", "source code");
        map.insert("text/x-sql", "source code");
        map.insert("text/x-r", "source code");
        map.insert("text/x-matlab", "source code");
        map.insert("text/x-perl", "source code");
        map.insert("text/x-lua", "source code");
        map.insert("text/x-dart", "source code");
        map.insert("text/x-typescript", "source code");
        map.insert("text/x-javascript", "source code");
        map.insert("text/x-vue", "source code");
        map.insert("text/x-svelte", "source code");
        map.insert("text/x-elm", "source code");
        map.insert("text/x-clojure", "source code");
        map.insert("text/x-haskell", "source code");
        map.insert("text/x-ocaml", "source code");
        map.insert("text/x-fsharp", "source code");
        map.insert("text/x-elixir", "source code");
        map.insert("text/x-erlang", "source code");
        map.insert("text/x-nim", "source code");
        map.insert("text/x-crystal", "source code");
        map.insert("text/x-zig", "source code");
        map.insert("text/x-d", "source code");
        map.insert("text/x-pascal", "source code");
        map.insert("text/x-ada", "source code");
        map.insert("text/x-fortran", "source code");
        map.insert("text/x-cobol", "source code");
        map.insert("text/x-asm", "source code");
        map.insert("text/x-vb", "source code");
        map.insert("text/x-vbscript", "source code");
        map.insert("text/x-csharp", "source code");
        map.insert("text/x-vhdl", "source code");
        map.insert("text/x-verilog", "source code");
        map.insert("text/x-systemverilog", "source code");
        map.insert("text/x-tcl", "source code");
        map.insert("text/x-groovy", "source code");
        map.insert("text/x-gradle", "source code");
        map.insert("text/x-makefile", "source code");
        map.insert("text/x-cmake", "source code");
        map.insert("text/x-dockerfile", "source code");
        map.insert("text/x-yaml", "source code");
        map.insert("text/x-toml", "source code");
        map.insert("text/x-ini", "source code");
        map.insert("text/x-config", "source code");
        map.insert("text/x-properties", "source code");
        map.insert("text/x-gitignore", "source code");
        map.insert("text/x-gitattributes", "source code");
        map.insert("text/x-editorconfig", "source code");
        
        map
    };
}
