/// A module to handle parsing of color formats into the `Color` enum.
///
/// This module includes parsers for the following color formats:
/// - **RGB**: A color defined by three numeric values (e.g., "rgb(255, 0, 0)") or in "rgb(x, y, z)" format where `x`, `y`, and `z` are integer values (either decimal or hexadecimal).
/// - **Hex**: A color defined in hexadecimal format (e.g., "#FF0000").
/// - **Ansi256**: A 256-color ANSI code (e.g., "137" or "0x89" for hexadecimal).
/// - **Other formats**: This can include named colors or additional color formats defined by specific comma-separated values.
use crate::{Color, ParseColorError, ParseColorErrorKind};

/// Attempts to parse a single number from a string, either in decimal or hexadecimal format.
///
/// - If the string starts with "0x", it will be parsed as hexadecimal.
/// - Otherwise, it will be parsed as a decimal number.
///
/// # Parameters:
/// - `s`: The string slice containing the number to parse.
///
/// # Returns:
/// Returns an `Option<u8>` which is the parsed number as an unsigned byte.
/// If parsing fails, it returns `None`.
#[inline]
fn parse_number(s: &str) -> Option<u8> {
    if let Some(stripped) = s.strip_prefix("0x") {
        u8::from_str_radix(stripped, 16).ok()
    } else {
        s.parse::<u8>().ok()
    }
}

#[inline]
fn parse_percent_or_255(s: &str) -> Option<(u8, bool)> {
    s.strip_suffix('%')
        .and_then(|s| {
            s.parse::<f32>()
                .ok()
                .map(|t| ((t * 255.0 / 100.0).round() as u8, true))
        })
        .or_else(|| parse_number(s).map(|t| (t, false)))
}

/// Parses a string in the "rgb(x, y, z)" format, where x, y, and z are numbers in decimal or hexadecimal.
///
/// # Parameters:
/// - `s`: A string slice containing the RGB color in the format "rgb(x, y, z)", where `x`, `y`, and `z` are integers.
///
/// # Returns:
/// A `Result<Color, ParseColorError>`. On success, it returns `Color::Rgb(r, g, b)`, where `r`, `g`, and `b` are the parsed color values.
/// On failure, it returns an error of type `ParseColorError` indicating why the format is invalid.
pub fn parse_rgb(s: &str) -> Result<Color, ParseColorError> {
    let trimmed = if s.starts_with("rgb(") && s.ends_with(")") {
        // If it starts with "rgb(" and ends with ")", remove those parts
        s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(")")).ok_or_else(
            || ParseColorError {
                kind: ParseColorErrorKind::InvalidRgb,
                given: s.to_string(),
            },
        )?
    } else {
        s
    };

    let normalized = trimmed.replace([',', '/'], " ");
    let components: Vec<&str> = normalized.split_whitespace().collect();

    // Ensure exactly three components exist
    if components.len() != 3 {
        return Err(ParseColorError {
            kind: ParseColorErrorKind::InvalidRgb,
            given: s.to_string(),
        });
    }

    let colors: Result<Vec<u8>, ParseColorError> = components
        .iter()
        .map(|&component| {
            parse_percent_or_255(component).map(|(value, _)| value).ok_or_else(
                || ParseColorError {
                    kind: ParseColorErrorKind::InvalidRgb,
                    given: s.to_string(),
                },
            )
        })
        .collect();

    let colors = colors?;

    if colors.iter().all(|&x| (0..=255).contains(&x)) {
        Ok(Color::Rgb(colors[0], colors[1], colors[2]))
    } else {
        Err(ParseColorError {
            kind: ParseColorErrorKind::InvalidRgb,
            given: s.to_string(),
        })
    }
}

/// Parses a string in hex format (e.g., "#FF0000") into a `Color::Hex`.
///
/// # Parameters:
/// - `s`: A string slice containing the hexadecimal color.
///
/// # Returns:
/// A `Result<Color, ParseColorError>`. On success, it returns `Color::Hex(s)`, where `s` is the hexadecimal string.
/// On failure, it returns an error of type `ParseColorError` if the string is not a valid hex color.
pub fn parse_hex(s: &str) -> Result<Color, ParseColorError> {
    if !s.starts_with('#') || !s[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ParseColorError {
            kind: ParseColorErrorKind::InvalidHex,
            given: s.to_string(),
        });
    }

    if s.len() != 4 && s.len() != 7 {
        return Err(ParseColorError {
            kind: ParseColorErrorKind::InvalidHex,
            given: s.to_string(),
        });
    }

    let upper = s.to_ascii_uppercase();
    Ok(Color::Hex(Box::leak(upper.into_boxed_str())))
}

/// A more flexible parser that can handle "ansi256" or "rgb".
///
/// # Parameters:
/// - `s`: A string slice containing the color to parse, either in "rgb", "ansi256", or another supported format.
///
/// # Returns:
/// A `Result<Color, ParseColorError>`. On success, it returns a valid `Color` variant.
/// On failure, it returns a `ParseColorError` describing the issue with the input.
pub fn parse_other(s: &str) -> Result<Color, ParseColorError> {
    let s = s.replace([',', '/'], " ");
    let codes = s.split_whitespace().collect::<Vec<&str>>();
    if codes.len() == 1 {
        if let Some(n) = parse_number(codes[0]) {
            Ok(Color::Ansi256(n))
        } else if s.chars().all(|c| c.is_ascii_hexdigit()) {
            Err(ParseColorError {
                kind: ParseColorErrorKind::InvalidAnsi256,
                given: s.to_string(),
            })
        } else {
            Err(ParseColorError {
                kind: ParseColorErrorKind::InvalidName,
                given: s.to_string(),
            })
        }
    } else if codes.len() == 3 {
        parse_rgb(s.as_str())
    } else {
        Err(if s.contains(",") {
            ParseColorError {
                kind: ParseColorErrorKind::InvalidRgb,
                given: s.to_string(),
            }
        } else {
            ParseColorError {
                kind: ParseColorErrorKind::InvalidName,
                given: s.to_string(),
            }
        })
    }
}
