use crate::{
    files,
    metadata::{field, tag_selector, values},
    model::*,
};
use chrono::{Duration, NaiveDateTime};
use serde_json::{json, Value};
use std::{collections::HashSet, path::Path};

pub fn build_file_plan(
    document: &Document,
    request: &ChangeRequest,
) -> Result<PlannedFile, String> {
    if document.fingerprint != request.expected
        || document.sidecar_fingerprint != request.sidecar_expected
    {
        return Err("External changes detected. Reload the file and review edits again.".into());
    }
    if document.file.readonly {
        return Err("The source file is read-only".into());
    }
    let source_path = Path::new(&document.file.path);
    let sidecar = (document.capabilities.write == "sidecar" || request.force_sidecar)
        && document.file.extension != "xmp";
    if document.capabilities.write == "readOnly" && !sidecar && !request.edits.is_empty() {
        return Err("Embedded writing is not supported for this format".into());
    }
    if request.privacy.is_some() && (sidecar || document.capabilities.write != "embedded") {
        return Err("Embedded privacy cleanup is not supported for this format. A sidecar cannot remove data from the original.".into());
    }
    let target = if sidecar {
        document
            .sidecar
            .clone()
            .unwrap_or(files::path_string(&files::sidecar_path(source_path))?)
    } else {
        document.file.path.clone()
    };
    let source = if sidecar { "sidecar" } else { "embedded" };
    let target_expected = if sidecar {
        document.sidecar_fingerprint.clone()
    } else {
        Some(document.fingerprint.clone())
    };
    let mut plan = PlannedFile {
        file: document.file.clone(),
        expected: document.fingerprint.clone(),
        sidecar_expected: document.sidecar_fingerprint.clone(),
        target,
        target_expected,
        operations: vec![],
        differences: vec![],
        warnings: vec![],
        error: None,
        rename_to: None,
        privacy: request.privacy.clone(),
    };
    files::ensure_writable(Path::new(&plan.target))?;
    let mut seen = HashSet::new();
    for edit in &request.edits {
        if edit.value.to_string().len() > 32768 || value_text(&edit.value).contains('\0') {
            return Err("Value exceeds 32 KiB or contains NUL".into());
        }
        if !seen.insert(&edit.field) {
            return Err(format!("Duplicate edit for {}", edit.field));
        }
        let (selector, kind) = field(&edit.field)
            .ok_or_else(|| format!("Unsupported writable field: {}", edit.field))?;
        let current = effective_value(document, &edit.field, source);
        let value = if edit.mode == "shift" {
            if kind != "date" {
                return Err("Time shifts only apply to dates".into());
            }
            let seconds = edit
                .value
                .as_i64()
                .ok_or("Time shift must be a whole number of seconds")?;
            if seconds.unsigned_abs() > 315576000 {
                return Err("Time shift exceeds ten years".into());
            }
            Value::String(shift_date(&value_text(&current), seconds)?)
        } else if kind == "list" {
            let requested = list_value(&edit.value)?;
            let existing = list_value(&current)?;
            let output = match edit.mode.as_str() {
                "replace" => requested,
                "add" => {
                    let mut all = existing;
                    for item in requested {
                        if !all.contains(&item) {
                            all.push(item);
                        }
                    }
                    all
                }
                "remove" => existing
                    .into_iter()
                    .filter(|v| !requested.contains(v))
                    .collect(),
                _ => return Err("Unsupported list mode".into()),
            };
            if output.is_empty() {
                Value::Null
            } else {
                json!(output)
            }
        } else {
            if edit.mode != "replace" {
                return Err("Only list fields support add/remove".into());
            }
            validate_value(kind, &edit.value)?
        };
        add_operation(&mut plan, document, source, selector, value.clone());
        if request.sync_legacy && !sidecar {
            if !["JPEG", "TIFF"].contains(&document.capabilities.format.as_str()) {
                plan.warnings.push(
                    "Legacy synchronization is limited to JPEG/TIFF; XMP was planned only.".into(),
                );
            } else {
                synchronize(&mut plan, document, &edit.field, &value)?;
            }
        }
    }
    if let Some(preset) = &request.privacy {
        privacy(&mut plan, document, preset)?;
    }
    if let Some(name) = &request.rename {
        files::validate_filename(name)?;
        if files::extension(Path::new(name)) != document.file.extension {
            return Err("Renaming must preserve the file extension".into());
        }
        if document.sidecar.is_some() {
            return Err("Rename of a file with a companion sidecar is not enabled; move both together outside Tagryn.".into());
        }
        if sidecar && !request.edits.is_empty() {
            return Err("Apply sidecar edits before renaming".into());
        }
        let destination = source_path.with_file_name(name);
        if destination != source_path {
            if destination.exists() {
                return Err(format!("Rename collision: {name}"));
            }
            plan.rename_to = Some(files::path_string(&destination)?);
            plan.target = document.file.path.clone();
            plan.target_expected = Some(document.fingerprint.clone());
            plan.differences.push(Difference {
                tag: "File:FileName".into(),
                before: json!(document.file.name),
                after: json!(name),
                source: "filesystem".into(),
            });
        }
    }
    if sidecar {
        plan.warnings.push(
            "Writes target the XMP sidecar. Embedded metadata remains in the source file.".into(),
        );
    }
    if document.sidecar.is_some()
        && !sidecar
        && (!request.edits.is_empty() || request.privacy.is_some())
    {
        plan.warnings.push("A companion XMP sidecar exists and is not changed by this embedded-file operation. Review its values separately.".into());
    }
    if !request.sync_legacy && !request.edits.is_empty() {
        plan.warnings
            .push("XMP only: existing EXIF and IPTC fields are not synchronized.".into());
    }
    for operation in &plan.operations {
        if !operation.value.is_null()
            && document
                .tags
                .iter()
                .filter(|tag| tag.source == source && tag_selector(tag) == operation.tag)
                .count()
                > 1
        {
            return Err(format!("Ambiguous tag instances: {}. Instance-specific writing is not supported; no file was changed.", operation.tag));
        }
    }
    Ok(plan)
}

pub fn effective_value(document: &Document, id: &str, source: &str) -> Value {
    let Some((selector, _)) = field(id) else {
        return Value::Null;
    };
    let value = values(&document.tags, selector, source);
    if !value.is_null() {
        return value;
    }
    if source == "sidecar" {
        let embedded = values(&document.tags, selector, "embedded");
        if !embedded.is_null() {
            return embedded;
        }
    }
    let names: &[&str] = match id {
        "dateTaken" => &["DateTimeOriginal", "CreateDate"],
        "creator" => &["Artist", "By-line"],
        "copyright" => &["Copyright", "CopyrightNotice"],
        "description" => &["ImageDescription", "Caption-Abstract"],
        "title" => &["ObjectName"],
        "keywords" => &["Keywords"],
        "latitude" => &["GPSLatitude"],
        "longitude" => &["GPSLongitude"],
        _ => &[],
    };
    let fallback = document
        .tags
        .iter()
        .find(|t| names.contains(&t.name.as_str()) && !t.derived && t.source == "embedded");
    if id == "dateTaken" {
        if let Some(tag) = fallback {
            let mut value = value_text(&tag.raw);
            if value.len() == 19 {
                if let Some(offset) = document
                    .tags
                    .iter()
                    .find(|t| t.name == "OffsetTimeOriginal" && t.source == "embedded")
                {
                    value.push_str(&value_text(&offset.raw));
                }
            }
            return json!(value);
        }
    }
    fallback.map(|t| t.raw.clone()).unwrap_or(Value::Null)
}

fn add_operation(
    plan: &mut PlannedFile,
    document: &Document,
    source: &str,
    selector: &str,
    value: Value,
) {
    let before = values(&document.tags, selector, source);
    if equivalent(&before, &value) {
        return;
    }
    plan.operations.retain(|op| op.tag != selector);
    plan.differences
        .retain(|d| !(d.tag == selector && d.source == source));
    plan.operations.push(WriteOp {
        tag: selector.into(),
        value: value.clone(),
    });
    plan.differences.push(Difference {
        tag: selector.into(),
        before,
        after: value,
        source: source.into(),
    });
}

fn synchronize(
    plan: &mut PlannedFile,
    document: &Document,
    id: &str,
    value: &Value,
) -> Result<(), String> {
    let selectors: &[(&str, usize)] = match id {
        "title" => &[("IPTC:ObjectName", 64)],
        "description" => &[
            ("IFD0:ImageDescription", 65535),
            ("IPTC:Caption-Abstract", 2000),
        ],
        "creator" => &[("IFD0:Artist", 65535), ("IPTC:By-line", 32)],
        "copyright" => &[("IFD0:Copyright", 65535), ("IPTC:CopyrightNotice", 128)],
        "keywords" => &[("IPTC:Keywords", 64)],
        "dateTaken" => &[("ExifIFD:DateTimeOriginal", 64)],
        _ => &[],
    };
    for (tag, limit) in selectors {
        let val = if tag.starts_with("IFD0:") && value.is_array() {
            json!(list_value(value)?.join("; "))
        } else {
            value.clone()
        };
        for text in list_value(&val)? {
            if text.len() > *limit {
                return Err(format!("{tag} exceeds {limit} UTF-8 bytes; disable legacy synchronization or shorten the value"));
            }
        }
        if id == "dateTaken" && !value.is_null() {
            let text = value_text(value);
            add_operation(plan, document, "embedded", tag, json!(&text[..19]));
            if text.len() > 19 {
                let suffix = &text[19..];
                add_operation(
                    plan,
                    document,
                    "embedded",
                    "ExifIFD:OffsetTimeOriginal",
                    json!(if suffix == "Z" { "+00:00" } else { suffix }),
                );
            }
        } else {
            add_operation(plan, document, "embedded", tag, val);
        }
    }
    if id == "keywords" || selectors.iter().any(|(t, _)| t.starts_with("IPTC:")) {
        add_operation(
            plan,
            document,
            "embedded",
            "IPTC:CodedCharacterSet",
            json!("UTF8"),
        );
    }
    Ok(())
}

fn privacy(plan: &mut PlannedFile, document: &Document, preset: &str) -> Result<(), String> {
    if !["location", "personal", "publication"].contains(&preset) {
        return Err("Unknown privacy preset".into());
    }
    let mut groups = HashSet::new();
    for tag in &document.tags {
        if tag.source != "embedded" || tag.derived {
            continue;
        }
        let name = tag.name.to_lowercase();
        let location = name.contains("gps")
            || [
                "city",
                "country",
                "countrycode",
                "state",
                "location",
                "sublocation",
                "locationcreated",
                "locationshown",
            ]
            .contains(&name.as_str());
        let personal = [
            "artist",
            "author",
            "ownername",
            "cameraownername",
            "creator",
            "by-line",
            "serialnumber",
            "internalserialnumber",
            "bodyserialnumber",
            "lensserialnumber",
            "creatorcontactinfo",
            "personinimage",
        ]
        .contains(&name.as_str())
            || name.starts_with("creatorwork");
        let publication = [
            "title",
            "description",
            "imagedescription",
            "caption-abstract",
            "subject",
            "keywords",
            "comment",
            "usercomment",
            "history",
            "documentid",
            "instanceid",
            "originaldocumentid",
        ]
        .contains(&name.as_str());
        if tag.group == "MakerNotes" {
            groups.insert("MakerNotes".to_owned());
            continue;
        }
        if ["ThumbnailImage", "PreviewImage", "JpgFromRaw"].contains(&tag.name.as_str()) {
            groups.insert(tag.name.clone());
            continue;
        }
        if location
            || (preset != "location" && personal)
            || (preset == "publication" && publication)
        {
            if ["EXIF", "IPTC", "XMP"].contains(&tag.group.as_str()) {
                add_operation(plan, document, "embedded", &tag_selector(tag), Value::Null);
            }
        }
    }
    for group in groups {
        let selector = if group == "MakerNotes" {
            "MakerNotes:all".into()
        } else {
            group.clone()
        };
        plan.operations.push(WriteOp {
            tag: selector,
            value: Value::Null,
        });
        for tag in document
            .tags
            .iter()
            .filter(|t| t.source == "embedded" && (t.group == group || t.name == group))
        {
            plan.differences.push(Difference {
                tag: tag.key.clone(),
                before: tag.raw.clone(),
                after: Value::Null,
                source: "embedded".into(),
            });
        }
    }
    plan.warnings.push("MakerNotes and embedded previews are removed when present because they may retain private data. Orientation and ICC profiles are preserved. Rereading checks known sensitive tags; unknown/private encodings and visible image content are not proof of anonymization.".into());
    Ok(())
}

pub fn sensitive_residuals(tags: &[Tag]) -> Vec<String> {
    tags.iter()
        .filter(|t| {
            !t.derived
                && t.source == "embedded"
                && (t.name.to_lowercase().contains("gps")
                    || t.name.to_lowercase().contains("serial")
                    || t.group == "MakerNotes"
                    || ["ThumbnailImage", "PreviewImage", "JpgFromRaw"].contains(&t.name.as_str()))
        })
        .map(|t| t.key.clone())
        .collect()
}

pub fn list_value(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::Null => Ok(vec![]),
        Value::String(s) => {
            if s.is_empty() {
                Ok(vec![])
            } else {
                Ok(vec![s.clone()])
            }
        }
        Value::Array(a) => a
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or("List values must be strings".into())
            })
            .collect(),
        Value::Number(n) => Ok(vec![n.to_string()]),
        _ => Err("Expected text, number or list".into()),
    }
}
pub fn validate_value(kind: &str, value: &Value) -> Result<Value, String> {
    if value.is_null() || value.as_str() == Some("") {
        return Ok(Value::Null);
    }
    if value.to_string().len() > 32768 || value_text(value).contains('\0') {
        return Err("Value too long or contains NUL".into());
    }
    match kind {
        "text" => value
            .as_str()
            .map(|s| json!(s))
            .ok_or("Expected text".into()),
        "rating" | "latitude" | "longitude" | "number" => {
            let n = value
                .as_f64()
                .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
                .ok_or("Expected a number")?;
            if !n.is_finite()
                || (kind == "rating" && (n.fract() != 0.0 || !(-1.0..=5.0).contains(&n)))
                || (kind == "latitude" && !(-90.0..=90.0).contains(&n))
                || (kind == "longitude" && !(-180.0..=180.0).contains(&n))
            {
                return Err("Number outside supported range".into());
            }
            Ok(json!(n))
        }
        "date" => {
            let text = value
                .as_str()
                .ok_or("Expected date text")?
                .replace('T', " ");
            let mut bytes = text.into_bytes();
            if bytes.len() >= 10 {
                bytes[4] = b':';
                bytes[7] = b':';
            }
            let date = String::from_utf8(bytes).map_err(|e| e.to_string())?;
            shift_date(&date, 0).map(Value::String)
        }
        _ => Err("Unsupported field type".into()),
    }
}
pub fn shift_date(text: &str, seconds: i64) -> Result<String, String> {
    let base = text
        .get(..19)
        .ok_or("Date must include year, month, day and time")?;
    let date = NaiveDateTime::parse_from_str(base, "%Y:%m:%d %H:%M:%S")
        .map_err(|_| "Use YYYY:MM:DD HH:mm:ss with optional Z or ±HH:mm")?;
    let suffix = &text[19..];
    if !suffix.is_empty() && suffix != "Z" {
        let valid = suffix.len() == 6
            && matches!(suffix.as_bytes()[0], b'+' | b'-')
            && suffix.as_bytes()[3] == b':'
            && suffix
                .get(1..3)
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|v| v <= 14)
            && suffix
                .get(4..6)
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|v| v < 60);
        if !valid {
            return Err("Invalid timezone offset. Missing timezones are left unspecified.".into());
        }
    }
    let shifted = date
        .checked_add_signed(Duration::seconds(seconds))
        .ok_or("Date overflow")?;
    Ok(format!("{}{suffix}", shifted.format("%Y:%m:%d %H:%M:%S")))
}
pub fn equivalent(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    if (a.is_null() && (b.as_str() == Some("") || b.as_array().is_some_and(Vec::is_empty)))
        || (b.is_null() && (a.as_str() == Some("") || a.as_array().is_some_and(Vec::is_empty)))
    {
        return true;
    }
    if let (Some(a), Some(b)) = (
        a.as_f64()
            .or_else(|| a.as_str().and_then(|s| s.parse().ok())),
        b.as_f64()
            .or_else(|| b.as_str().and_then(|s| s.parse().ok())),
    ) {
        return (a - b).abs() < 0.000001;
    }
    if let (Ok(a), Ok(b)) = (list_value(a), list_value(b)) {
        return a == b;
    }
    false
}
