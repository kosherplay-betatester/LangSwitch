/// Direction of the conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionDirection {
    EnglishToHebrew,
    HebrewToEnglish,
}

/// Convert input text between Hebrew and English layouts.
///
/// Rules:
/// - If English -> Hebrew: All uppercase is converted to lowercase first (e.g., YexY -> yexy -> טקסט).
/// - If Hebrew -> English: Result is all lowercase English (e.g., אקסא -> text).
/// - Non-alphabetic characters like spaces, numbers, and unaffected symbols are preserved.
pub fn convert_text(input: &str) -> (String, ConversionDirection) {
    let hebrew_count = input.chars().filter(|c| is_hebrew_letter(*c)).count();
    let latin_count = input.chars().filter(|c| c.is_ascii_alphabetic()).count();

    let direction = if hebrew_count > latin_count {
        ConversionDirection::HebrewToEnglish
    } else if latin_count > 0 {
        ConversionDirection::EnglishToHebrew
    } else {
        // Fallback check for layout-specific punctuation
        let has_hebrew_punct = input.chars().any(|c| matches!(c, 'ת' | 'ץ' | 'ף'));
        if has_hebrew_punct {
            ConversionDirection::HebrewToEnglish
        } else {
            ConversionDirection::EnglishToHebrew
        }
    };

    let result = match direction {
        ConversionDirection::EnglishToHebrew => {
            // First convert all English uppercase to lowercase as required
            let lower = input.to_lowercase();
            lower.chars().map(en_char_to_he).collect()
        }
        ConversionDirection::HebrewToEnglish => {
            // Map Hebrew chars to lowercase English
            input.chars().map(he_char_to_en).collect()
        }
    };

    (result, direction)
}

fn is_hebrew_letter(c: char) -> bool {
    ('\u{05D0}'..='\u{05EA}').contains(&c)
}

fn en_char_to_he(c: char) -> char {
    match c {
        'q' => '/',
        'w' => '\'',
        'e' => 'ק',
        'r' => 'ר',
        't' => 'א',
        'y' => 'ט',
        'u' => 'ו',
        'i' => 'ן',
        'o' => 'ם',
        'p' => 'פ',
        '[' => ']',
        ']' => '[',
        'a' => 'ש',
        's' => 'ד',
        'd' => 'ג',
        'f' => 'כ',
        'g' => 'ע',
        'h' => 'י',
        'j' => 'ח',
        'k' => 'ל',
        'l' => 'ך',
        ';' => 'ף',
        '\'' => ',',
        'z' => 'ז',
        'x' => 'ס',
        'c' => 'ב',
        'v' => 'ה',
        'b' => 'נ',
        'n' => 'מ',
        'm' => 'צ',
        ',' => 'ת',
        '.' => 'ץ',
        '/' => '.',
        '`' => ';',
        other => other,
    }
}

fn he_char_to_en(c: char) -> char {
    match c {
        '/' => 'q',
        '\'' => 'w',
        'ק' => 'e',
        'ר' => 'r',
        'א' => 't',
        'ט' => 'y',
        'ו' => 'u',
        'ן' => 'i',
        'ם' => 'o',
        'פ' => 'p',
        ']' => '[',
        '[' => ']',
        'ש' => 'a',
        'ד' => 's',
        'ג' => 'd',
        'כ' => 'f',
        'ע' => 'g',
        'י' => 'h',
        'ח' => 'j',
        'ל' => 'k',
        'ך' => 'l',
        'ף' => ';',
        ',' => '\'',
        'ז' => 'z',
        'ס' => 'x',
        'ב' => 'c',
        'ה' => 'v',
        'נ' => 'b',
        'מ' => 'n',
        'צ' => 'm',
        'ת' => ',',
        'ץ' => '.',
        '.' => '/',
        ';' => '`',
        // In case an English uppercase/lowercase is in the mixed text, ensure lowercase
        other if other.is_ascii_uppercase() => other.to_ascii_lowercase(),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_to_hebrew_lowercase() {
        let (res, dir) = convert_text("yexy");
        assert_eq!(res, "טקסט");
        assert_eq!(dir, ConversionDirection::EnglishToHebrew);
    }

    #[test]
    fn test_english_to_hebrew_uppercase() {
        let (res, dir) = convert_text("YEXY");
        assert_eq!(res, "טקסט");
        assert_eq!(dir, ConversionDirection::EnglishToHebrew);

        let (res2, _) = convert_text("YexY");
        assert_eq!(res2, "טקסט");
    }

    #[test]
    fn test_hebrew_to_english() {
        let (res, dir) = convert_text("אקסא");
        assert_eq!(res, "text");
        assert_eq!(dir, ConversionDirection::HebrewToEnglish);
    }

    #[test]
    fn test_sentence_conversion() {
        let (res, _) = convert_text("akuo jcrhh");
        assert_eq!(res, "שלום חבריי");
    }
}
